use gl::types::*;

/*extern "C" {
    pub fn glReadPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, type_: GLenum, pixels: *mut u8);
}*/
// reimpl as used from
// https://github.com/madsim-rs/madsim/blob/main/madsim/src/sim/time/system_time.rs

lazy_static::lazy_static! {

    // not sure if these are the correct types yet
    pub static ref XGetGeometry: unsafe extern "C" fn(
        display: *mut libc::c_void, // Display
        window: libc::c_ulong, // Window?
        root_return: *mut libc::c_ulong,
        x_return: *mut libc::c_int,
        y_return: *mut libc::c_int,
        width_return: *mut libc::c_uint,
        height_return: *mut libc::c_uint,
        border_width_return: *mut libc::c_uint,
        depth_return: *mut libc::c_int,
    ) -> libc::c_int = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"XGetGeometry\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };
}