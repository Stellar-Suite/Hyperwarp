mod utils;

fn main() {
    println!("Hello, world!");
}

redhook::hook! {
    unsafe fn premain_plugin() => premain_plugin_first {
        println!("Premain starting. Please wait. ");
    }

    unsafe fn XOpenDisplay(c_char) => xopen_display_hook {
        println!("Display open requestedc. ");
    }
}