use std::borrow::BorrowMut;


use std::ffi::CStr;
use libc::{c_char, c_void};

use crate::{constants::{xlib::XWindowChanges, Library}, host::hosting::HOST};

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
        x: libc::c_int,
        y: libc::c_int,
        width: libc::c_uint,
        height: libc::c_uint,
        border_width: libc::c_uint,
        depth: libc::c_int,
        class: libc::c_uint,
        visual: Visual,
        value_mask: libc::c_ulong,
        attributes: XSetWindowAttributes
    ) -> Window => x_create_window_first {
        if HOST.config.enable_x11 {
            // HOST.test();
            
            // TODO: various feature flags locks are held longer than they should be, I should be adding more blocks to limit the lock time
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_x11();
            }

            let result = redhook::real!(XCreateWindow_hw_direct)(
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

            let window = crate::host::window::Window::new(((result) as *const c_void) as usize, Library::Xlib);

            // WARNING: implicit tick
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

redhook::hook! {
    unsafe fn XCreateWindow_hw_direct(
        display: Display,
        parent: Window,
        x: libc::c_int,
        y: libc::c_int,
        width: libc::c_uint,
        height: libc::c_uint,
        border_width: libc::c_uint,
        depth: libc::c_int,
        class: libc::c_uint,
        visual: Visual,
        value_mask: libc::c_ulong,
        attributes: XSetWindowAttributes
    ) -> Window => x_create_window_hw_direct {
        std::ptr::null()
    }
}


redhook::hook! {
    unsafe fn XCreateSimpleWindow(
        display: Display,
        parent: Window,
        x: libc::c_int,
        y: libc::c_int,
        width: libc::c_uint,
        height: libc::c_uint,
        border_width: libc::c_uint,
        border: libc::c_ulong,
        background: libc::c_ulong
    ) -> Window => x_create_simple_window_first {
        if HOST.config.enable_x11 {
            // HOST.test();

            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_x11();
            }

            let result = redhook::real!(XCreateSimpleWindow_hw_direct)(
                display,
                parent,
                x,
                y,
                width,
                height,
                border_width,
                border,
                background,
            );
            if HOST.config.debug_mode {
                println!("XCreateSimpleWindow: {}", result as u64);
            }

            let window = crate::host::window::Window::new(((result) as *const c_void) as usize, Library::Xlib);

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

redhook::hook! {
    unsafe fn XCreateSimpleWindow_hw_direct(
        display: Display,
        parent: Window,
        x: libc::c_int,
        y: libc::c_int,
        width: libc::c_uint,
        height: libc::c_uint,
        border_width: libc::c_uint,
        border: libc::c_ulong,
        background: libc::c_ulong
    ) -> Window => x_create_simple_window_hw_direct {
        std::ptr::null()
    }
}

redhook::hook! {
    unsafe fn XConfigureWindow(display: Display, window: Window, value_mask: libc::c_ulong, values: *const XWindowChanges) => x_configure_window_first {
        if HOST.config.enable_x11 {
            // HOST.test();
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_x11();
            }

            redhook::real!(XConfigureWindow_hw_direct)(
                display,
                window,
                value_mask,
                values,
            );

            let configuration = values.as_ref().unwrap();
            println!("reconfigure: {:#?}", configuration);

            // HOST.onWindowConfigure(window, values);
        } else {
            if HOST.config.debug_mode {
                println!("Attempted to configure window, denied by config");
            }
        }
    }
}

redhook::hook! {
    unsafe fn XConfigureWindow_hw_direct(display: Display, window: Window, value_mask: libc::c_ulong, values: *const XWindowChanges) => x_configure_window_hw_direct {
    }
}

// https://tronche.com/gui/x/xlib/window/XResizeWindow.html
redhook::hook! {
    unsafe fn XResizeWindow(display: Display, window: Window, width: libc::c_uint, height: libc::c_uint) => x_resize_window_first {
        println!("XResizeWindow called");
        if HOST.config.enable_x11 {
            // HOST.test();

            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_x11();
            }

            println!("XResizeWindow called {}x{}", width, height);

            redhook::real!(XResizeWindow_hw_direct)(
                display,
                window,
                width,
                height,
            );

            HOST.onWindowResize(window as usize, width, height);

        } else {
            if HOST.config.debug_mode {
                println!("Attempted to resize window, denied by config");
            }
        }
    }
}

redhook::hook! {
    unsafe fn XResizeWindow_hw_direct(display: Display, window: Window, width: libc::c_uint, height: libc::c_uint) => x_resize_window_hw_direct {
    }
}

pub fn try_modify_symbol(symbol_name: &str) -> Option<*mut c_void> {
    match symbol_name {
        "XOpenDisplay" => Some(x_open_display_first as *mut c_void),
        "XCreateWindow" => Some(x_create_window_first as *mut c_void),
        "XCreateSimpleWindow" => Some(x_create_simple_window_first as *mut c_void),
        "XConfigureWindow" => Some(x_configure_window_first as *mut c_void),
        "XResizeWindow" => Some(x_resize_window_first as *mut c_void),
        _ => None
    }
}