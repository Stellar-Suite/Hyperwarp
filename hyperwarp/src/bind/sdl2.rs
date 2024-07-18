

// this is fine to spam call since it accesses internal variables
// https://github.com/libsdl-org/SDL/blob/e264bb517827a2c9cf16570fd89385c0f1f7f344/src/video/SDL_video.c#L2623

use sdl2_sys_lite::bindings::{SDL_Event, SDL_Joystick, SDL_JoystickType};
use stellar_shared::vendor::sdl_bindings::{SDL_KeyCode, SDL_Scancode};

use crate::utils::manual_types::sdl2::SDL_Window;

// *const libc::c_char is a String

// SDL2 now needs to use the _hw_direct suffix to work with dynapi to force use of cache.
// some of these are written by Supermaven but I usuually check if they are correct

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

    // joystick
    pub static ref SDL_JoystickAttachVirtual: unsafe extern "C" fn(type_: SDL_JoystickType, naxes: libc::c_int, nbuttons: libc::c_int, nhats: libc::c_int) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickAttachVirtual_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_JoystickDetachVirtual: unsafe extern "C" fn(index: libc::c_int) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickDetachVirtual_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_JoystickIsVirtual: unsafe extern "C" fn(joystick: *mut SDL_Joystick) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickIsVirtual_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    /*SDL_JoystickSetVirtualAxis
    SDL_JoystickSetVirtualButton
    SDL_JoystickSetVirtualHat */
    pub static ref SDL_JoystickSetVirtualAxis: unsafe extern "C" fn(joystick: *mut SDL_Joystick, axis: libc::c_int, value: libc::c_short) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickSetVirtualAxis_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_JoystickSetVirtualButton: unsafe extern "C" fn(joystick: *mut SDL_Joystick, button: libc::c_int, value: libc::c_char) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickSetVirtualButton_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };

    pub static ref SDL_JoystickSetVirtualHat: unsafe extern "C" fn(joystick: *mut SDL_Joystick, hat: libc::c_int, value: libc::c_char) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"SDL_JoystickSetVirtualHat_hw_direct\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };
}