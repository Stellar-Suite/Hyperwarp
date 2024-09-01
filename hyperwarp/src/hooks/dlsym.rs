// ugly dlsym router

use std::{collections::HashMap, ffi::CString, sync::Mutex};

use backtrace::Backtrace;
use lazy_static::lazy_static;

use libc::{c_void, c_char};

use crate::{shim, utils::{pointer::Pointer, sdl2_dynapi::DYNAPI_FUNCS, sdl2_dynapi_helper}};

use super::{glx, sdl2, xlib};

extern "C" {
    pub fn odlsym(handle: *const c_void, symbol: *const c_char) -> *mut c_void;
}

extern "C" {
    pub fn init_if_needed();
}

lazy_static! {
    pub static ref DLSYM_CACHE: Mutex<HashMap<String, Pointer>> = Mutex::new(HashMap::new());
}

pub fn query_dlsym_cache(symbol_name: &str) -> Option<Pointer> {
    DLSYM_CACHE.lock().unwrap().get(symbol_name).copied()
}

#[cfg(feature = "log_dlsym")]
pub const LOG_DLSYM: bool = true;
#[cfg(not(feature = "log_dlsym"))]
pub const LOG_DLSYM: bool = false;

pub const USE_CACHE_WORKAROUND: bool = true;

pub fn check_cache_integrity() {
    let cache = DLSYM_CACHE.lock().unwrap();
    for (symbol_name, pointer) in cache.iter() {
        if pointer.0.is_null() {
            println!("cache integrity error: symbol {} has a null pointer", symbol_name);
        }
    }
}

// #[cfg(crate_type="dylib")]
redhook::hook! {
    unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void => dlsym_first {
        let symbol_name = std::ffi::CStr::from_ptr(symbol).to_str().unwrap();
        if LOG_DLSYM {
            println!("dlsym: symbol name {}",symbol_name);
        }
        // TODO: refactor the long if else into a map?
        let should_cache = symbol_name.starts_with("SDL_") || symbol_name.starts_with("glX") || symbol_name.starts_with("X");
        if should_cache && !symbol_name.ends_with("_hw_direct")  {
            // caching

            init_if_needed();

            let symbol_cstring = CString::new(symbol_name).unwrap();
            let symbol_pointer = odlsym(handle, symbol_cstring.as_ptr() as *const c_char);
            if !symbol_pointer.is_null() {
                if LOG_DLSYM {
                    println!("cache real {} pointer {}",symbol_name,symbol_pointer as usize);
                }
                {
                    let mut cache = DLSYM_CACHE.lock().unwrap();
                    // println!("locked cache");
                    cache.insert(symbol_name.to_string(), Pointer(symbol_pointer));
                }
                // println!("unlocked cache");
            } else {
                if LOG_DLSYM {
                    println!("caching {} pointer failed because we got a null pointer.", symbol_name);
                }
            }
        }

        if symbol_name == "SDL_Init" {
            // cache hack of all time
            for symbol in DYNAPI_FUNCS {
                let symbol_cstring = CString::new(symbol).unwrap();
                let symbol_pointer = odlsym(handle, symbol_cstring.as_ptr() as *const c_char);
                if !symbol_pointer.is_null() {
                    if LOG_DLSYM {
                        println!("sdl force cache cache real {} pointer {}",symbol,symbol_pointer as usize);
                    }
                    {
                        let mut cache = DLSYM_CACHE.lock().unwrap();
                        if !cache.contains_key(symbol) {
                            cache.insert(symbol.to_string(), Pointer(symbol_pointer));
                        }
                        cache.insert(format!("{}_hw_direct", symbol), Pointer(symbol_pointer));
                    }
                    // println!("unlocked cache");
                } else {
                    if LOG_DLSYM {
                        println!("sdl force cache caching {} pointer failed because we got a null pointer.", symbol);
                    }
                }
            }
        }

        if symbol_name.ends_with("_hw_direct") {
            init_if_needed();
            // this is only slow for the one lookup yk
            let symbol_string = CString::new(symbol_name.replace("_hw_direct","")).unwrap();
            let real_symbol_name = symbol_name.replace("_hw_direct","");
            if LOG_DLSYM {
                println!("indirect resolving {} pointer",symbol_name);
            }
            let cache_hit_dynapi = {
                DLSYM_CACHE.lock().unwrap().contains_key(&format!("{}_hw_sdl_dynapi", real_symbol_name))
            };
            if cache_hit_dynapi {
                if LOG_DLSYM {
                    println!("using dynapi bypass for {}", symbol_name);
                }
                let ptr = DLSYM_CACHE.lock().unwrap().get(&format!("{}_hw_sdl_dynapi", real_symbol_name)).unwrap().as_mut_func();
                if LOG_DLSYM {    
                    println!("dynapi bypass gave {}", ptr as usize);
                }
                return ptr;
            }
            let pointer = odlsym(handle, symbol_string.as_ptr() as *const c_char);
            if pointer.is_null() {
                if LOG_DLSYM {
                    println!("impending null pointer for {}",symbol_name);
                }
                let cache_hit = {
                    DLSYM_CACHE.lock().unwrap().contains_key(&real_symbol_name)
                };
                if cache_hit && USE_CACHE_WORKAROUND {
                    if LOG_DLSYM {
                        println!("luckily the cache contains the symbol");
                    }
                    let pointer = {
                        DLSYM_CACHE.lock().unwrap().get(&real_symbol_name).unwrap().as_mut_func()
                    };
                    // this shouldn't trigger
                    if pointer.is_null() {
                        if LOG_DLSYM {
                            println!("that pointer is also null :( it is {}", pointer as usize);
                        }
                    }
                    return pointer;
                }
            }
            println!("direct resolving {} pointer to {}",symbol_name,pointer as usize);
            pointer
        // TODO: the transmute is not actually needed you can just cast to *mut c_void
        } else if symbol_name == "_internal_rust_launch" {
            unsafe {
                std::mem::transmute(shim::launch::rust_launch_first as *const c_void) 
            }
        } else if symbol_name == "glXSwapBuffers" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers as *const c_void) }
        } else if symbol_name == "glXSwapBuffersMscOML" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers_msc_oml as *const c_void) }
        } else if symbol_name == "glXGetProcAddress" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else if symbol_name == "glXGetProcAddressARB" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else if let Some(pointer) = xlib::try_modify_symbol(symbol_name) {
            pointer
        } else if let Some(pointer) = sdl2::try_modify_symbol(symbol_name) {
            pointer
        } else if symbol_name == "SDL_DYNAPI_entry" {
            // TODO: allow this to be disabled
            if LOG_DLSYM {
                println!("sent modified SDL_DYNAPI_entry");
            }
            sdl2_dynapi_helper::SDL_DYNAPI_entry_modified as *mut c_void
        }else {
            /*if symbol_name.contains("udev") {
                let bt = Backtrace::new();
                println!("dlsym: symbol name {} backtrace {:?}", symbol_name, bt);
            }*/
            // odlsym is from preglue
            // println!("using odlsym");
            /*unsafe {
                let p = (odlsym as *const c_void);
                println!("p is {}", p as u64);
            }*/
            // println!("telling c preglue to grab the odlsym if needed");
            init_if_needed();
            // println!("brace");
            let result = odlsym(handle, symbol);
            // println!("nothing exploded looking up {}",symbol_name);
            if LOG_DLSYM {
                println!("dlsym({})",symbol_name);
            }
            result
        }
    }
}