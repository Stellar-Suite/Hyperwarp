use human_panic::setup_panic;
use libc::c_char;
use libc::c_void;

use crate::host::hosting::HOST;

pub mod constants;
pub mod hooks;
pub mod host;
pub mod utils;
pub mod bind;
pub mod standalone;
pub mod shim;
pub mod platform;

// I forgot what this is for. 
fn main() {
    println!("Hello, world!");
}

redhook::hook! {
    unsafe fn premain_plugin() => premain_plugin_first {
        setup_panic!();
        if HOST.config.debug_mode { // this will trigger initalization of host
            println!("Premain starting. Please wait. ");
        }
    }
}