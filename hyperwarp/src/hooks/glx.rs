use std::collections::HashMap;

use libc::{c_char, c_void, c_long, c_int};
use stellar_protocol::protocol::GraphicsAPI;

use crate::{host::hosting::HOST, utils::pointer::Pointer};

// types
type Display = *mut c_void;
type GLXDrawable = *mut c_void;

type c_func = *const c_void;

#[no_mangle]
pub extern "C" fn glXSwapBuffersShim(name: Display, drawble: GLXDrawable) {
    
}

pub fn modify_pointers(name: &str, pointer: Pointer) -> Pointer{
    match name {
        "glXSwapBuffers" => {
            println!("overrode glxswapbuffers");
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_glx();
            }
            HOST.suggest_graphics_api(GraphicsAPI::OpenGL);
            // memorize the real glxSwapBuffers pointer and return our shim instead
            HOST.func_pointers.lock().unwrap().insert(name.to_string(), pointer);
            let func: c_func = unsafe { std::mem::transmute(glXSwapBuffersShim as *const c_void) };
            Pointer(func)
        },
        _ => pointer
    }
}

// pub const getProcAddressOverrides: HashMap<String, *const c_void> = HashMap::new();

redhook::hook! {
    unsafe fn glXSwapBuffers_hw_direct(name: Display, drawble: GLXDrawable) => gl_x_swap_buffers_hw_direct {
        // shim so I can run redhook::real on it
    }
}

// extern void glXSwapBuffers( Display *dpy, GLXDrawable drawable );
redhook::hook! {
    unsafe fn glXSwapBuffers(name: Display, drawble: GLXDrawable) => gl_x_swap_buffers {
        if HOST.config.enable_x11 && HOST.config.enable_glx {
            // HOST.test();
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_glx();
            }

            HOST.onFrameSwapBegin();            
            let result = redhook::real!(glXSwapBuffers_hw_direct)(name, drawble);
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

redhook::hook! {
    unsafe fn glXSwapBuffersMscOML(name: Display, drawble: GLXDrawable, target_msc: c_long, divisor: c_long, remainder: c_long) => gl_x_swap_buffers_msc_oml {
        if HOST.config.enable_x11 && HOST.config.enable_glx {
            // HOST.test();
            {
                let mut features = HOST.features.lock().unwrap();
                features.enable_glx();
            }

            HOST.onFrameSwapBegin();            
            let result = redhook::real!(glXSwapBuffersMscOML)(name, drawble, target_msc, divisor, remainder);
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

#[no_mangle]
pub extern "C" fn glXSwapBuffersPA(name: Display, drawble: GLXDrawable){
    // println!("Entered glXSwapBuffersPA");
    let func_pointers = HOST.func_pointers.lock().unwrap();
    // println!("Locked pointer map");
    let func = func_pointers.get("glXSwapBuffers").unwrap();
    match func {
        Pointer(func_ref) => {

            let func: extern "C" fn(name: Display, drawble: GLXDrawable) = unsafe { std::mem::transmute(*func_ref) };
            
            HOST.onFrameSwapBegin();
            func(name, drawble);
            HOST.onFrameSwapEnd();

        },
        _ => {
            // println!("glXSwapBuffersPA: func is not a pointer");
        }
    }
}

fn glxGetProcAddrShim(name: String, origPointer: Pointer) -> Pointer{
    match name.as_ref() {
        "glXSwapBuffers" => {
            HOST.suggest_graphics_api(GraphicsAPI::OpenGL);

            if HOST.config.debug_mode {
                println!("overrode glxswapbuffers");
            }
            let mut features = HOST.features.lock().unwrap();
            features.enable_glx();
            // return our above shim
            Pointer(glXSwapBuffersPA as *const c_void)
        },
        _ => origPointer
    }
}

// shim so we can call real from another

redhook::hook! {
    unsafe fn glXGetProcAddress_hw_direct(name: *const c_char) -> c_func => gl_x_get_proc_address_hw_direct {
        std::ptr::null()
    }
}

redhook::hook! {
    unsafe fn glXGetProcAddress(name: *const c_char) -> c_func => gl_x_get_proc_address {
        let func = redhook::real!(glXGetProcAddress_hw_direct)(name);
        let func_name = std::ffi::CStr::from_ptr(name).to_str().unwrap();
        // println!("glx get proc addr {}", func_name);
        let origPointer = Pointer(func);
        // insert orig pointer
        HOST.func_pointers.lock().unwrap().insert(func_name.to_owned(), Pointer(func));
        let pointer = glxGetProcAddrShim(func_name.to_owned(), origPointer);

        pointer.0
    }
}

redhook::hook! {
    unsafe fn glXGetProcAddressARB_hw_direct(name: *const c_char) -> c_func => gl_x_get_proc_address_arb_hw_direct {
        std::ptr::null() // exists so we can call real on it
    }
}

redhook::hook! {
    unsafe fn glXGetProcAddressARB(name: *const c_char) -> c_func => gl_x_get_proc_address_arb {
        let func = redhook::real!(glXGetProcAddressARB_hw_direct)(name);
        let func_name = std::ffi::CStr::from_ptr(name).to_str().unwrap();
        // println!("glx get proc addr arb {}", func_name);
        // insert orig pointer
        HOST.func_pointers.lock().unwrap().insert(func_name.to_owned(), Pointer(func));
        let origPointer = Pointer(func);
        let pointer = glxGetProcAddrShim(func_name.to_owned(), origPointer); // we use the same since this func only switches funcs for stuff we're interested in
        pointer.0
    }
}