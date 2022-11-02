use libc::{c_void,c_char};

use crate::host::hosting::HOST;

redhook::hook! {
    unsafe fn XOpenDisplay(name: c_char) -> *const c_void => x_open_display_first {
        if HOST.config.enable_x11 {
            redhook::real!(XOpenDisplay)(name)
        } else {
            if HOST.config.debug_mode {
                println!("Attempted to open {}", name);
            }
            std::ptr::null()
        }
    }
}