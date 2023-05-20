use libc::{c_char, c_void};

use crate::host::hosting::HOST;

// types
pub type Display = *const c_void;

redhook::hook! {
    unsafe fn XOpenDisplay(name: c_char) -> Display => x_open_display_first {
        if HOST.config.enable_x11 {
            HOST.test();
            redhook::real!(XOpenDisplay)(name)
        } else {
            if HOST.config.debug_mode {
                println!("Attempted to open {}", name);
            }
            std::ptr::null()
        }
    }
}
