use libc::{c_char, c_int, c_void};
use stellar_protocol::protocol::GraphicsAPI;

use crate::constants::sdl2::SDL_FALSE;

use crate::host::window::Window;
use crate::utils::manual_types::sdl2::{SDL_Window, Uint32, SDL_Renderer};

use crate::host::hosting::HOST;

// Many of these hooks

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
    unsafe fn SDL_CreateWindow_hw_direct(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: Uint32) -> *const SDL_Window  => sdl_createwindow_hw_direct {
        std::ptr::null() // Once again this is a shim so I can run redhook::real on it
    }
}

redhook::hook! {
    unsafe fn SDL_CreateWindow(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: Uint32) -> *const SDL_Window  => sdl_createwindow_first {
        if HOST.config.debug_mode {
            println!("SDL_CreateWindow called");
        }
        {
            let mut features = HOST.features.lock().unwrap();
            features.enable_sdl2();
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

        let mut result = if HOST.config.enable_sdl2 {
            redhook::real!(SDL_CreateWindow_hw_direct)(title, final_x, final_y, final_w, final_h, flags)
        } else {
            std::ptr::null()
        };

        // this should never panic
        // who would have a negative window size?
        let window = Window {
            id: result as usize,
            is_SDL2: true,
        };

        HOST.onWindowCreate(window, Some(final_x), Some(final_y), Some(final_w.try_into().unwrap()), Some(final_h.try_into().unwrap()));

        if HOST.config.debug_mode {
            println!("SDL_CreateWindow called with x: {}, y: {}, w: {}, h: {}", final_x, final_y, final_w, final_h);
        }

        result
    }
}

redhook::hook! {
    unsafe fn SDL_GL_SwapBuffers_hw_direct() => sdl_gl_swapbuffers_hw_direct {
        // shim so I can run redhook::real on it   
    }
}

redhook::hook! {
    unsafe fn SDL_GL_SwapBuffers() => sdl_gl_swapbuffers_first {
        if HOST.config.debug_mode {
            println!("SDL_GL_SwapBuffers called");
        }
        if HOST.config.enable_sdl2 {
            HOST.suggest_graphics_api(GraphicsAPI::OpenGL);
            HOST.onFrameSwapBegin();
            let result = redhook::real!(SDL_GL_SwapBuffers_hw_direct)();
            HOST.onFrameSwapEnd();
            result
        } else {
            // std::ptr::null()
        }
    }
}

redhook::hook! {
    unsafe fn SDL_GL_SwapWindow_hw_direct(display: *mut SDL_Window) => sdl_gl_swapwindow_hw_direct {
        // shim so I can run redhook::real on it   
    }
}

redhook::hook! {
    unsafe fn SDL_GL_SwapWindow(display: *mut SDL_Window) => sdl_gl_swapwindow_first {
        if HOST.config.tracing_mode {
            println!("SDL_GL_SwapWindow called");
        }
        if HOST.config.enable_sdl2 {
            HOST.onFrameSwapBegin();
            HOST.suggest_graphics_api(GraphicsAPI::OpenGL);
            redhook::real!(SDL_GL_SwapWindow_hw_direct)(display);
            HOST.onFrameSwapEnd();
        }
    }
}

redhook::hook! {
    unsafe fn SDL_RenderPresent(renderer: *mut SDL_Renderer) -> *const c_void => sdl_renderpresent_first {
        if HOST.config.tracing_mode {
            println!("SDL_RenderPresent called");
        }
        if HOST.config.enable_sdl2 {
            HOST.onFrameSwapBegin();
            let result = redhook::real!(SDL_RenderPresent)(renderer);
            HOST.onFrameSwapEnd();
            result
        } else {
            std::ptr::null()
        }
    }
}