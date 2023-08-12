redhook::hook! {
    unsafe fn rust_launch() -> i32 => rust_launch_first {
        println!("Test success");
        0
    }
}