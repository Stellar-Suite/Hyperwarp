use libc::{c_char, c_void};

use crate::host::hosting::HOST;

// types
type Display = *const c_void;
type GLXDrawable = *const c_void;

// extern void glXSwapBuffers( Display *dpy, GLXDrawable drawable );
redhook::hook! {
    unsafe fn glXSwapBuffers(name: Display, drawble: GLXDrawable) => gl_x_swap_buffers {
        if HOST.config.enable_x11 && HOST.config.enable_glx {
            // HOST.test();
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_glx();
            }

            HOST.get_behavior().onFrameSwapBegin();            
            let result = redhook::real!(glXSwapBuffers)(name, drawble);
            HOST.get_behavior().onFrameSwapEnd();
            result
        } else {
            if HOST.config.debug_mode {
                // println!("Attempted to open {}", name);
            }
            // std::ptr::null()
        }
    }
}
