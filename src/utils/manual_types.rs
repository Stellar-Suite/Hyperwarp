// We use enums to ref structs in C++ without actually defining them fully
// see https://users.rust-lang.org/t/idiomatic-untyped-pointer/3757/2

pub mod glfw {
    use libc::c_int;

    pub type GLFW_STATUS_BINARY = c_int;
}

pub mod sdl2 {
    use libc::c_void;

    pub type Uint32 = u32;
    pub enum SDL_Window {}

    pub type SDL_Renderer = *const c_void;
}

pub mod libc {
    use std::str::from_utf8;

    use libc::{c_char, c_void};

    pub type ENUM_TYPE = i32;
    pub type POINTER_TYPE = *const c_void;
    pub unsafe fn read_cstr(str: *const c_char) -> String {
        std::str::from_utf8(std::ffi::CStr::from_ptr(str).to_bytes())
            .unwrap_or("")
            .to_owned()
    }
}
