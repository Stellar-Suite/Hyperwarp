pub mod sdl2;

pub const LIBRARY_NAME: &str = "Hyperwarp";

#[derive(Debug, Copy, Clone)]
pub enum Library {
    SDL2,
    SDL3,
    GLFW,
    Xlib,
    // idk
    // not quite platforms but we still can use this to label resources
    EGL,
    GLX,
    // misc types
    Other,
    RustNative,
    CNative
}