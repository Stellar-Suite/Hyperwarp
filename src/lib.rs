use libc::c_char;
use libc::c_void;

mod utils;

fn main() {
    println!("Hello, world!");
}

redhook::hook! {
    unsafe fn premain_plugin() => premain_plugin_first {
        println!("Premain starting. Please wait. ");
    }
    
}

redhook::hook! {
    unsafe fn XOpenDisplay(name: c_char) -> *const c_void => XOpenDisplay_first {
        println!("Attempted to open {}", name);
        return std::ptr::null();
    }
}