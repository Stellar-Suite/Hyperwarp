use gl::types::*;

pub fn glReadPixelsSafe(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, type_: GLenum, pixels: *mut u8) {
    unsafe {
        super::gl::glReadPixels(x, y, width, height, format, type_, pixels);
    }
}