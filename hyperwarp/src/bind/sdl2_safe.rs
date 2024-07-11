use std::ffi::CStr;

use sdl2_sys_lite::bindings::SDL_Event;
use stellar_shared::vendor::sdl_bindings::{SDL_KeyCode, SDL_Scancode};

use crate::utils::manual_types::sdl2::SDL_Window;

use super::sdl2::SDL_GetError;


pub fn SDL_GetWindowSize_safe(window: *const SDL_Window) -> (i32, i32)  {
    let mut outputX: i32 = 0;
    let mut outputY: i32 = 0;
    unsafe {
        super::sdl2::SDL_GetWindowSize(window, (&mut outputX), (&mut outputY));
    }
    (outputX, outputY)
}

pub fn SDL_GetWindowID_safe(window: *const SDL_Window) -> u32 {
    unsafe {
        super::sdl2::SDL_GetWindowID(window)
    }
}

// https://github.com/Rust-SDL2/rust-sdl2/blob/dba66e80b14e16de309df49df0c20fdaf35b8c67/src/sdl2/sdl.rs#L378
pub fn SDL_GetError_safe() -> String {
    unsafe {
        let err_ptr = SDL_GetError();
          //super::sdl2::SDL_GetError(); 
        CStr::from_ptr(err_ptr).to_str().unwrap().to_owned()
    }
}

pub fn SDL_PushEvent_safe(event: *const SDL_Event) -> i32 {
    unsafe {
        super::sdl2::SDL_PushEvent(event)
    }
}

pub fn SDL_GetTicks_safe() -> u32 {
    unsafe {
        super::sdl2::SDL_GetTicks()
    }
}

pub fn SDL_GetScancodeFromKey_safe(key: SDL_KeyCode) -> SDL_Scancode {
    unsafe {
        super::sdl2::SDL_GetScancodeFromKey(key)
    }
}