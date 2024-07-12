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

pub fn SDL_DYNAPI_entry_modified(apiver: u32, jump_table: *mut libc::c_void, tablesize: u32) -> i32 {
    if LOG_DLSYM {
        println!("modded SDL_DYNAPI_entry called, api ver: {}, table size: {}", apiver, tablesize);
    }
    let orig_func_ptr = query_dlsym_cache("SDL_DYNAPI_entry").expect("Grabbing original dlsym failed.").as_func();
    let orig_func: extern "C" fn(u32, *mut libc::c_void, u32) -> i32 = unsafe { std::mem::transmute(orig_func_ptr) };
    let result = orig_func(apiver, jump_table, tablesize);

    let mut dlsym_cache_locked = DLSYM_CACHE.lock().unwrap();

    let bytes_per_pointer = std::mem::size_of::<*mut libc::c_void>();

    for (i, func) in DYNAPI_FUNCS.iter().enumerate() {
        if i > (tablesize as usize) {
            if LOG_DLSYM {
                println!("skipping table index {} which contains {}", i, func);
            }
            continue;
        }
        dlsym_cache_locked.insert(format!("{}_hw_sdl_dynapi", func), Pointer(unsafe {
            jump_table.offset(i as isize)
        }));
        if let Some(alt_ptr) = crate::hooks::sdl2::try_modify_symbol(func){
            // set offset to our new function pointer
            unsafe {
                (jump_table.byte_offset((bytes_per_pointer * i) as isize) as *mut usize).write(alt_ptr as usize);
            }
            if LOG_DLSYM {
                println!("SDL_DYNAPI_entry_hw_direct modified {} to {}", func, alt_ptr as usize);
            }
        }
    }

    if result != 0 {
        println!("SDL_DYNAPI_entry_hw_direct returned {}, which is not ok", result);
    }
    0 // ok
}