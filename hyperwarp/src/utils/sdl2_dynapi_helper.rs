use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::hooks::dlsym::{query_dlsym_cache, DLSYM_CACHE, LOG_DLSYM};

use super::{pointer::Pointer, sdl2_dynapi::DYNAPI_FUNCS};

lazy_static! {
    pub static ref DYNAPI_FUNCS_INDEX: HashMap<String, usize> = {
        let mut map = HashMap::new();
        for (i, func) in DYNAPI_FUNCS.iter().enumerate() {
            map.insert(func.to_string(), i);
        }
        map
    };
}

pub static mut INTERNAL_JUMP_TABLE: [usize; 601] = [0; 601];

pub fn SDL_DYNAPI_entry_modified(apiver: u32, jump_table: *mut libc::c_void, tablesize: u32) -> i32 {
    if LOG_DLSYM {
        println!("modded SDL_DYNAPI_entry called, api ver: {}, table size: {}", apiver, tablesize);
    }
    let orig_func_ptr = query_dlsym_cache("SDL_DYNAPI_entry").expect("Grabbing original dlsym failed.").as_func();
    let orig_func: extern "C" fn(u32, *mut libc::c_void, u32) -> i32 = unsafe { std::mem::transmute(orig_func_ptr) };
    let result = orig_func(apiver, jump_table, tablesize);
    let result_extension = orig_func(apiver, jump_table, tablesize);
    

    let bytes_per_pointer = std::mem::size_of::<*mut libc::c_void>();
    if LOG_DLSYM {
        println!("SDL_DYNAPI_entry modified, tablesize: {}, pointer size in bytes: {}", tablesize, bytes_per_pointer);
    }

    if result < 0 {
        println!("orig SDL_DYNAPI_entry returned {}, which is not ok", result);
        return result;
    }

    let modern_ok = result_extension >= 0;
    if result_extension < 0 {
        println!("requesting new funcs with orig SDL_DYNAPI_entry returned {}, may affect avalibility of new functions like joystick emu", result_extension);
    }

    {
        let mut dlsym_cache_locked = DLSYM_CACHE.lock().unwrap();

        let jump_table_usized: *mut usize = jump_table as *mut usize;

        let internal_len = unsafe {
            INTERNAL_JUMP_TABLE.len()
        };

        for (i, func) in DYNAPI_FUNCS.iter().enumerate() {
            if i > (tablesize as usize) && i > internal_len {
                if LOG_DLSYM {
                    println!("skipping table index {} which contains {}", i, func);
                }
                continue;
            }
            let ptr_to_orig_ptr = unsafe {
                if i > (tablesize as usize) {
                    // use internal jump table
                    if !modern_ok {
                        continue;
                    }
                    INTERNAL_JUMP_TABLE.get_mut(i).unwrap() as *mut usize
                } else {
                    jump_table_usized.offset(i as isize)
                }
            };
            let orig_ptr = unsafe {
                *ptr_to_orig_ptr
            };
            println!("SDL_dynapi helper: read the orig ptr as {} for {}", orig_ptr, func);
            if !ptr_to_orig_ptr.is_null() {
                dlsym_cache_locked.insert(format!("{}_hw_sdl_dynapi", func), Pointer(orig_ptr as *const libc::c_void));
                dlsym_cache_locked.insert(format!("{}", func), Pointer(orig_ptr as *const libc::c_void));
            } else {
                println!("{}'s pointer is null {}", func, ptr_to_orig_ptr as usize);
            }
            if let Some(alt_ptr) = crate::hooks::sdl2::try_modify_symbol(func){
                // set offset to our new function pointer
                unsafe {
                    (jump_table.byte_offset((bytes_per_pointer * i) as isize) as *mut usize).write(alt_ptr as usize);
                }
                if LOG_DLSYM {
                    println!("SDL_DYNAPI_entry: modified {} to {}", func, alt_ptr as usize);
                }
            }
        }
    }

    if result != 0 {
        println!("orig SDL_DYNAPI_entry returned {}, which is not ok", result);
        return result;
    }
    0 // ok
}