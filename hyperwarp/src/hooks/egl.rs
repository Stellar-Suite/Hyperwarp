use libc::{c_char, c_void};

use crate::host::hosting::HOST;

// types
type EGLDisplay = *const c_void;
type EGLSurface = *const c_void;

// extern void glXSwapBuffers( Display *dpy, GLXDrawable drawable );
redhook::hook! {
    unsafe fn eglSwapBuffers(name: EGLDisplay, surface: EGLSurface) => gl_x_swap_buffers {
        if HOST.config.enable_x11 && HOST.config.enable_glx {
            // HOST.test();
            {
                let mut features = HOST.features.lock().unwrap();
            }

            HOST.onFrameSwapBegin();            
            let result = redhook::real!(eglSwapBuffers)(name, surface);
            HOST.onFrameSwapEnd();
            result
        } else {
            if HOST.config.debug_mode {
                // println!("Attempted to open {}", name);
            }
            // std::ptr::null()
        }
    }
}
