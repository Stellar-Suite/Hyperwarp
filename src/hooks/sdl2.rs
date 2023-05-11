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
    unsafe fn SDL_CreateWindow(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: Uint32) -> *const SDL_Window  => sdl_createwindow_first {
        if HOST.config.debug_mode {
            println!("SDL_CreateWindow called");
        }
        let mut final_x: c_int = x;
        let mut final_y: c_int = y;
        let mut final_w: c_int = w;
        let mut final_h: c_int = h;

        if let Some(new_w) = HOST.config.window_width_override {
            final_w = new_w as c_int;
        }
        if let Some(new_h) = HOST.config.window_height_override {
            final_h = new_h as c_int;
        }

        if HOST.config.enable_sdl2 {
            redhook::real!(SDL_CreateWindow)(title, final_x, final_y, final_w, final_h, flags)
        } else {
            std::ptr::null()
        }
    }
}