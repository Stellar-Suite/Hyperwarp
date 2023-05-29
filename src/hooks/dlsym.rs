// ugly dlsym router

use std::ffi::CString;

use libc::{c_void, c_char};

use super::glx;

extern "C" {
    pub fn odlsym(handle: *const c_void, symbol: *const c_char) -> *mut c_void;
}

extern "C" {
    pub fn init_if_needed();
}

redhook::hook! {
    unsafe fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void => dlsym_first {
        let symbol_name = std::ffi::CStr::from_ptr(symbol).to_str().unwrap();
        // println!("dlsym symbol name {}",symbol_name);
        if symbol_name.ends_with("_hw_direct") {
            init_if_needed();
            let symbol_string = CString::new(symbol_name.replace("_hw_direct","")).unwrap();
            odlsym(handle, symbol_string.as_ptr() as *const c_char)
        } else if symbol_name == "glXSwapBuffers" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers as *const c_void) }
        } else if symbol_name == "glXSwapBuffersMscOML" {
            unsafe { std::mem::transmute(glx::gl_x_swap_buffers_msc_oml as *const c_void) }
        } else if symbol_name == "glXGetProcAddress" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else if symbol_name == "glXGetProcAddressARB" {
            unsafe { std::mem::transmute(glx::gl_x_get_proc_address as *const c_void) }
        } else {
            // odlsym is from preglue
            // println!("using odlsym");
            /*unsafe {
                let p = (odlsym as *const c_void);
                println!("p is {}", p as u64);
            }*/
            // println!("telling c preglue to grab the odlsym if needed");
            init_if_needed();
            // println!("brace");
            let result = odlsym(handle, symbol);
            // println!("nothing exploded looking up {}",symbol_name);
            result
        }
    }
}