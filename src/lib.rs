use libc::c_char;
use libc::c_void;

mod utils;
mod host;
mod hooks;

fn main() {
    println!("Hello, world!");
}

redhook::hook! {
    unsafe fn premain_plugin() => premain_plugin_first {
        println!("Premain starting. Please wait. ");
    }
}