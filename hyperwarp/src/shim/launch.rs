use crate::host::hosting::HOST;

redhook::hook! {
    unsafe fn rust_launch() -> i32 => rust_launch_first {
        println!("Test success");
        // we are still using the host lazy static since it simplifies config loading
        // sanity!
        if HOST.config.capture_mode {
            println!("Can't capture when being run through shim. ");
            return 1;
        }
        // do stuffs

        // Exit
        0
    }
}