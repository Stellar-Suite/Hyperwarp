// ugly dlsym router

use std::{collections::HashMap, ffi::CString, sync::Mutex};

use backtrace::Backtrace;
use lazy_static::lazy_static;

use libc::{c_void, c_char};

use crate::{shim, utils::pointer::Pointer};

use super::{glx, sdl2, xlib};

extern "C" {
    pub fn odlsym(handle: *const c_void, symbol: *const c_char) -> *mut c_void;
}

extern "C" {
    pub fn init_if_needed();
}

lazy_static! {
    static ref DLSYM_CACHE: Mutex<HashMap<String, Pointer>> = Mutex::new(HashMap::new());
}

#[cfg(feature = "log_dlsym")]
pub const LOG_DLSYM: bool = true;
#[cfg(not(feature = "log_dlsym"))]
pub const LOG_DLSYM: bool = false;

// #[cfg(crate_type="dylib")]
redhook::hook! {
    unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void => dlsym_first {
        let symbol_name = std::ffi::CStr::from_ptr(symbol).to_str().unwrap();
        // println!("dlsym: symbol name {}",symbol_name);
        // TODO: refactor the long if else into a map?
        if symbol_name.starts_with("SDL_") || symbol_name.starts_with("glX") || symbol_name.starts_with("X") {
            // caching
            let symbol_cstring = CString::new(symbol_name.replace("SDL_","")).unwrap();
            let symbol_pointer = odlsym(handle, symbol_cstring.as_ptr() as *const c_char);
            if !symbol_pointer.is_null() {
                println!("cache {} pointer {}",symbol_name,symbol_pointer as usize);
                let mut cache = DLSYM_CACHE.lock().unwrap();
                cache.insert(symbol_name.to_string(), Pointer(symbol_pointer));
            }
        }
        if symbol_name.ends_with("_hw_direct") {
            init_if_needed();
            // this is only slow for the one lookup yk
            let symbol_string = CString::new(symbol_name.replace("_hw_direct","")).unwrap();
            let real_symbol_name = symbol_name.replace("_hw_direct","");
            let pointer = odlsym(handle, symbol_string.as_ptr() as *const c_char);
            if pointer.is_null() {
                println!("impending null pointer for {}",symbol_name);
                let cache_hit = {
                    DLSYM_CACHE.lock().unwrap().contains_key(&real_symbol_name)
                };
                if cache_hit {
                    println!("luckily the cache contains the symbol");
                    let pointer = DLSYM_CACHE.lock().unwrap().get(&real_symbol_name).unwrap().as_mut_func();
                    // this shouldn't trigger
                    if pointer.is_null() {
                        println!("that pointer is also null :( it is {}", pointer as usize);
                    }
                    return pointer;
                }
            }
            println!("direct resolving {} pointer {}",symbol_name,pointer as usize);
            pointer
        }  else if symbol_name == "_internal_rust_launch" {
            unsafe {
                std::mem::transmute(shim::launch::rust_launch_first as *const c_void) 
            }
        } else if symbol_name == "XResizeWindow" {
            if LOG_DLSYM { println!("modified XResizeWindow"); }
            unsafe { std::mem::transmute(xlib::x_resize_window_first as *const c_void) }
        } else if symbol_name == "XConfigureWindow" {
            if LOG_DLSYM { println!("modified XConfigureWindow"); }
            unsafe { std::mem::transmute(xlib::x_configure_window_first as *const c_void) }
        }else if symbol_name == "XCreateWindow" {
            if LOG_DLSYM { println!("modified XCreateWindow"); }
            unsafe { std::mem::transmute(xlib::x_create_window_first as *const c_void) }
        } else if symbol_name == "XCreateSimpleWindow" {
            if LOG_DLSYM { println!("modified XCreateSimpleWindow"); }
            unsafe { std::mem::transmute(xlib::x_create_simple_window_first as *const c_void) }
        } else if symbol_name == "SDL_CreateWindow" {
            if LOG_DLSYM { println!("modified SDL_CreateWindow"); }
            unsafe { std::mem::transmute(sdl2::sdl_createwindow_first as *const c_void) }
        } else if symbol_name == "SDL_GL_SwapBuffers" {
            if LOG_DLSYM { println!("modified SDL_GL_SwapBuffers"); }
            unsafe { std::mem::transmute(sdl2::sdl_gl_swapbuffers_first as *const c_void) }
        } else if symbol_name == "SDL_GL_SwapWindow" {
            if LOG_DLSYM { println!("modified SDL_GL_SwapWindow"); }
            unsafe { std::mem::transmute(sdl2::sdl_gl_swapwindow_first as *const c_void) }
        } else if symbol_name == "SDL_GetKeyboardState" {
            if LOG_DLSYM { println!("modified SDL_GetKeyboardState"); }
            unsafe { std::mem::transmute(sdl2::sdl_getkeyboardstate_first as *const c_void) }
        } else if symbol_name == "SDL_SetWindowTitle" {
            if LOG_DLSYM { println!("modified SDL_SetWindowTitle"); }
            unsafe { std::mem::transmute(sdl2::sdl_setwindowtitle_first as *const c_void) }
        } else if symbol_name == "glXSwapBuffers" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers as *const c_void) }
        } else if symbol_name == "glXSwapBuffersMscOML" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers_msc_oml as *const c_void) }
        } else if symbol_name == "glXGetProcAddress" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else if symbol_name == "glXGetProcAddressARB" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else {
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