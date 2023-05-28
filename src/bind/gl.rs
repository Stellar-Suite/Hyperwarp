#![feature(linkage)]

use gl::types::*;

/*extern "C" {
    pub fn glReadPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, type_: GLenum, pixels: *mut u8);
}*/
// reimpl as used from
// https://github.com/madsim-rs/madsim/blob/main/madsim/src/sim/time/system_time.rs

lazy_static::lazy_static! {
    pub static ref glReadPixels: unsafe extern "C" fn(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, type_: GLenum, pixels: *mut u8) -> libc::c_void = unsafe {
        let ptr = libc::dlsym(libc::RTLD_NEXT, b"glReadPixels\0".as_ptr() as _);
        assert!(!ptr.is_null());
        std::mem::transmute(ptr)
    };
}

// cursed constant hardcoding
// TODO: better system
pub const K_GL_RGBA: GLenum = 0x1908;
pub const K_GL_UNSIGNED_BYTE: GLenum = 0x1401;