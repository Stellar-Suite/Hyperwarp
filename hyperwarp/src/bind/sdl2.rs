

// this is fine to spam call since it accesses internal variables
// https://github.com/libsdl-org/SDL/blob/e264bb517827a2c9cf16570fd89385c0f1f7f344/src/video/SDL_video.c#L2623

use crate::utils::manual_types::sdl2::SDL_Window;

lazy_static::lazy_static! {
    pub static ref SDL_GetWindowSize: unsafe extern "C" fn(window: *const SDL_Window, w: *mut i32, h: *mut i32) -> libc::c_void = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetWindowSize\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_GetWindowID: unsafe extern "C" fn(window: *const SDL_Window) -> libc::c_uint = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_GetWindowID\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };
}