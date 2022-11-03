use libc::{c_char, c_int, c_void};

use crate::constants::sdl2::SDL_FALSE;

use crate::utils::manual_types::sdl2::{SDL_Window, Uint32};

use crate::host::hosting::HOST;

redhook::hook! {
    unsafe fn SDL_Init(flags: Uint32) -> c_int => sdl_init_first {
        if HOST.config.debug_mode {
            println!("SDL_Init called");
        }
        if HOST.config.enable_sdl2 {
            redhook::real!(SDL_Init)(flags)
        } else {
            SDL_FALSE
        }
    }
}

redhook::hook! {
    unsafe fn SDL_CreateWindow(title: c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: Uint32) -> *const SDL_Window  => sdl_createwindow_first {
        if HOST.config.debug_mode {
            println!("SDL_CreateWindow called");
        }
        if HOST.config.enable_sdl2 {
            redhook::real!(SDL_CreateWindow)(title, x, y, w, h, flags)
        } else {
            std::ptr::null()
        }
    }
}
