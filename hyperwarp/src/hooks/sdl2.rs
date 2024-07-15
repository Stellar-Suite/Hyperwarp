use std::ffi::c_short;

use libc::{c_char, c_int, c_void};
use sdl2_sys_lite::bindings::{SDL_Event, SDL_EventType, SDL_WindowEventID};
use stellar_protocol::protocol::GraphicsAPI;

use crate::constants::sdl2::SDL_FALSE;

use crate::constants::Library;
use crate::host::window::Window;
use crate::utils::{self, format_window_title_prefix_cstr};
use crate::utils::manual_types::sdl2::{SDL_Window, Uint32, SDL_Renderer};

use crate::host::hosting::HOST;

// Many of these hooks

pub const SDL_DYNAPI_TABLE_MAX_SIZE: usize = 1024;

redhook::hook! {
    unsafe fn SDL_Init(flags: Uint32) -> c_int => sdl_init_first {
        if HOST.config.debug_mode {
            println!("SDL_Init called");
        }
        if HOST.config.enable_sdl2 {
            redhook::real!(SDL_Init_hw_direct)(flags)
        } else {
            SDL_FALSE
        }
    }
}

redhook::hook! {
    unsafe fn SDL_Init_hw_direct(flags: Uint32) -> c_int => sdl_init_hw_direct {
        // shim so I can run redhook::real on it
        0
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

        let final_title = utils::format_window_title_prefix_cstr(title);

        let mut result = if HOST.config.enable_sdl2 {
            redhook::real!(SDL_CreateWindow_hw_direct)(final_title, final_x, final_y, final_w, final_h, flags)
        } else {
            std::ptr::null()
        };

        // this should never panic
        // who would have a negative window size?
        let window = Window::new(result as usize, Library::SDL2);

        HOST.onWindowCreate(window, Some(final_x), Some(final_y), Some(final_w.try_into().unwrap()), Some(final_h.try_into().unwrap()));

        if HOST.config.debug_mode {
            println!("SDL_CreateWindow called with x: {}, y: {}, w: {}, h: {}", final_x, final_y, final_w, final_h);
        }

        // TODO: maybe we should send a fake window focus here.

        result
    }
}

redhook::hook! {
    unsafe fn SDL_CreateWindow_hw_direct(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: Uint32) -> *const SDL_Window  => sdl_createwindow_hw_direct {
        std::ptr::null() // Once again this is a shim so I can run redhook::real on it
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
    unsafe fn SDL_GL_SwapBuffers_hw_direct() => sdl_gl_swapbuffers_hw_direct {
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
    unsafe fn SDL_GL_SwapWindow_hw_direct(display: *mut SDL_Window) => sdl_gl_swapwindow_hw_direct {
        // shim so I can run redhook::real on it   
    }
}

redhook::hook! {
    unsafe fn SDL_RenderPresent(renderer: *mut SDL_Renderer) -> *const c_void => sdl_renderpresent_first {
        if HOST.config.tracing_mode {
            println!("SDL_RenderPresent called");
        }
        if HOST.config.enable_sdl2 {
            HOST.onFrameSwapBegin();
            let result = redhook::real!(SDL_RenderPresent_hw_direct)(renderer);
            HOST.onFrameSwapEnd();
            result
        } else {
            std::ptr::null()
        }
    }
}

redhook::hook! {
    unsafe fn SDL_RenderPresent_hw_direct(renderer: *mut SDL_Renderer) -> *const c_void => sdl_renderpresent_hw_direct {
        std::ptr::null()
    }
}

redhook::hook! {
    unsafe fn SDL_SetWindowTitle(display: *mut SDL_Window, title: *const c_char) => sdl_setwindowtitle_first {
        if HOST.config.debug_mode {
            println!("SDL_SetWindowTitle called");
        }

        if HOST.config.enable_sdl2 {
            let result = redhook::real!(SDL_SetWindowTitle_hw_direct)(display, format_window_title_prefix_cstr(title));
            result
        }
    }
}

redhook::hook! {
    unsafe fn SDL_SetWindowTitle_hw_direct(display: *mut SDL_Window, title: *const c_char) => sdl_setwindowtitle_hw_direct {
        // shim so I can run redhook::real on it   
    }
}

// c_char is one way of getting a u8 array in c
redhook::hook! {
    unsafe fn SDL_GetKeyboardState(count: *const c_int)-> *const c_char => sdl_getkeyboardstate_first {
        if HOST.config.debug_mode {

        }

        if HOST.config.enable_sdl2 {
            let result = redhook::real!(SDL_GetKeyboardState_hw_direct)(count);
            std::mem::transmute(HOST.input_manager.lock().unwrap().keyboard.get_virt_array_ptr())
        } else {
            std::ptr::null()
        }
    }
}

redhook::hook! {
    unsafe fn SDL_GetKeyboardState_hw_direct(count: *const c_int) -> *const c_char => sdl_getkeyboardstate_hw_direct {
        // shim so I can run redhook::real on it   
        std::ptr::null()
    }
}

redhook::hook! {
    unsafe fn SDL_DestroyWindow(display: *mut SDL_Window) => sdl_destroywindow_first {
        if HOST.config.debug_mode {
            println!("SDL_DestroyWindow called");
        }

        if HOST.config.enable_sdl2 {
            HOST.onWindowDestroy(display as usize);
            let result = redhook::real!(SDL_DestroyWindow_hw_direct)(display);
            result
        }
    }
}

redhook::hook! {
    unsafe fn SDL_DestroyWindow_hw_direct(display: *mut SDL_Window) => sdl_destroywindow_hw_direct {
        // shim so I can run redhook::real on it   
    }
}

pub fn SDL_should_allow_event(event: &SDL_Event) -> bool {
    unsafe {
        if event.type_ == SDL_EventType::SDL_WINDOWEVENT as u32 {
            let event_type_id = event.window.event;
            if event_type_id == SDL_WindowEventID::SDL_WINDOWEVENT_MINIMIZED as u8 {
                return false;
            } else if event_type_id == SDL_WindowEventID::SDL_WINDOWEVENT_FOCUS_LOST as u8 {
                return false;
            } else if event_type_id == SDL_WindowEventID::SDL_WINDOWEVENT_SHOWN as u8 {
                return false;
            }
        }
    }
    true
}

redhook::hook! {
    unsafe fn SDL_PollEvent(event: *mut SDL_Event) -> c_int => sdl_pollevent_first {
        if HOST.config.debug_mode {
            // println!("SDL_PollEvent called");
        }

        if HOST.config.enable_sdl2 {
            // flush inputs here as well
            HOST.input_manager.lock().unwrap().flush_queue();
            
            let result = redhook::real!(SDL_PollEvent_hw_direct)(event);
            if result != 0 {
                let event_ref = event.as_ref().unwrap();
                if !SDL_should_allow_event(event_ref) {
                    // hopefully nothing notices the event changed even if we return 0
                    // TODO: remove this print when stabilized.
                    println!("canceled event hack");
                    return 0;
                }
            }
            result
        } else {
            0
        }
    }
}

redhook::hook! {
    unsafe fn SDL_PollEvent_hw_direct(event: *mut SDL_Event) -> c_int => sdl_pollevent_hw_direct {
        // shim so I can run redhook::real on it   
        0
    }
}

pub fn try_modify_symbol(symbol_name: &str) -> Option<*mut c_void> {
    match symbol_name {
        "SDL_Init" => Some(sdl_init_first as *mut c_void),
        "SDL_CreateWindow" => Some(sdl_createwindow_first as *mut c_void),
        "SDL_GL_SwapBuffers" => Some(sdl_gl_swapbuffers_first as *mut c_void),
        "SDL_GL_SwapWindow" => Some(sdl_gl_swapwindow_first as *mut c_void),
        "SDL_RenderPresent" => Some(sdl_renderpresent_first as *mut c_void),
        "SDL_SetWindowTitle" => Some(sdl_setwindowtitle_first as *mut c_void),
        "SDL_GetKeyboardState" => Some(sdl_getkeyboardstate_first as *mut c_void),
        "SDL_DestroyWindow" => Some(sdl_destroywindow_first as *mut c_void),
        "SDL_PollEvent" => Some(sdl_pollevent_first as *mut c_void),
        _ => None
    }
}