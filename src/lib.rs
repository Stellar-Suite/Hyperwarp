use libc::c_char;
use libc::c_void;

use crate::host::hosting::HOST;

mod utils;
mod host;
mod hooks;

fn main() {
    println!("Hello, world!");
}

redhook::hook! {
    unsafe fn premain_plugin() => premain_plugin_first {
        if HOST.config.debug_mode {
            println!("Premain starting. Please wait. ");
        }
    }
}