

// this is fine to spam call since it accesses internal variables
// https://github.com/libsdl-org/SDL/blob/e264bb517827a2c9cf16570fd89385c0f1f7f344/src/video/SDL_video.c#L2623

use sdl2_sys_lite::bindings::SDL_Event;
use stellar_shared::vendor::sdl_bindings::{SDL_KeyCode, SDL_Scancode};

use crate::utils::manual_types::sdl2::SDL_Window;

// *const libc::c_char is a String

// SDL2 now needs to use the _hw_direct suffix to work with dynapi to force use of cache.

lazy_static::lazy_static! {
    pub static ref SDL_GetWindowSize: unsafe extern "C" fn(window: *const SDL_Window, w: *mut i32, h: *mut i32) -> libc::c_void = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetWindowSize_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_GetWindowID: unsafe extern "C" fn(window: *const SDL_Window) -> libc::c_uint = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetWindowID_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_GetError: unsafe extern "C" fn() -> *const libc::c_char = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetError_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_PushEvent: unsafe extern "C" fn(event: *const SDL_Event) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_PushEvent_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_GetTicks: unsafe extern "C" fn() -> libc::c_uint = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetTicks_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_GetScancodeFromKey: unsafe extern "C" fn(key: SDL_KeyCode) -> SDL_Scancode = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetScancodeFromKey_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };
}