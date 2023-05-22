use gl::types::*;

extern "C" {
    pub fn glReadPixels(x: GLint, y: GLint, width: GLsizei, height: GLsizei, format: GLenum, type_: GLenum, pixels: *mut u8);
}

// cursed constant hardcoding
// TODO: better system
pub const K_GL_RGBA: GLenum = 0x1908;
pub const K_GL_UNSIGNED_BYTE: GLenum = 0x1401;