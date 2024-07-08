use std::ffi::CStr;

use crate::utils::manual_types::sdl2::SDL_Window;


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
        let err_ptr = sdl2_sys::SDL_GetError();
          //super::sdl2::SDL_GetError(); 
        CStr::from_ptr(err_ptr).to_str().unwrap().to_owned()
    }
}