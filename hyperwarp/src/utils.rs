use std::ffi::{CStr, CString};

use libc::c_char;

use crate::{constants::LIBRARY_NAME, host::hosting::HOST};



pub mod config;
// TODO: autogen types
pub mod manual_types;

pub mod utils;

pub mod pointer;

pub mod redhook_modif;

pub fn format_window_title_prefix_cstr(c_string: *const c_char) -> *const c_char {
    if !HOST.config.retitle_windows {
        return c_string;
    }
    
    let c_str = unsafe { CStr::from_ptr(c_string) };
    let rust_str = c_str.to_str().expect("Bad C String");
    let formatted = format!("{} ({})", rust_str, LIBRARY_NAME);
    // let c_str = CString::new(c_string).unwrap();
    let formatted_c_str = CString::new(formatted).unwrap();
    formatted_c_str.as_ptr() as *const c_char
    // formatted_c_str.into_raw()
}