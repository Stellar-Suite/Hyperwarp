use std::borrow::BorrowMut;


use std::ffi::CStr;
use libc::{c_char, c_void};

use crate::{constants::Library, host::hosting::HOST};

// types
// TODO: convert them to without the pointer stuffs
pub type Display = *const c_void;
pub type Window = *const c_void;
pub type Visual = *const c_void;
pub type XSetWindowAttributes = *const c_void;

redhook::hook! {
    unsafe fn XOpenDisplay(name: *const c_char) -> *mut Display => x_open_display_first {
        if HOST.config.enable_x11 {
            // HOST.test();
            
            let mut features = HOST.features.lock().unwrap();
            features.enable_x11();

            redhook::real!(XOpenDisplay)(name)
        } else {
            if HOST.config.debug_mode {
                // println!("Attempted to open {}", CStr::from_ptr(name).to_str().unwrap());
            }
            std::ptr::null_mut()
        }
    }
}

redhook::hook! {
    unsafe fn XCreateWindow(
        display: Display,
        parent: Window,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        border_width: u32,
        depth: i32,
        class: u32,
        visual: Visual,
        value_mask: u64,
        attributes: XSetWindowAttributes
    ) -> Window => x_create_window_first {
        if HOST.config.enable_x11 {
            HOST.test();

            let mut features = HOST.features.lock().unwrap();
            features.enable_x11();

            let result = redhook::real!(XCreateWindow)(
                display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                depth,
                class,
                visual,
                value_mask,
                attributes,
            );
            if HOST.config.debug_mode {
                println!("XCreateWindow: {}", result as u64);
            }

            let window = crate::host::window::Window {
                id: ((result) as *const c_void) as usize,
                lib: Library::Xlib,
            };

            HOST.onWindowCreate(window, Some(x), Some(y), Some(width), Some(height));
            
            result
        } else {
            if HOST.config.debug_mode {
                println!("Attempted to create window, denied by config");
            }
            std::ptr::null()
        }
    }
}
