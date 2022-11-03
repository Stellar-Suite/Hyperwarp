use libc::{c_char, c_int, c_void};

use crate::host::hosting::HOST;

redhook::hook! {
    unsafe fn glfwInit() -> c_int => glfw_init_first {
        if HOST.config.debug_mode {
            println!("glfwInit called");
        }
        redhook::real!(glfwInit)()
    }
}
