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

pub fn is_SDL(lib: Library) -> bool {
    match lib {
        Library::SDL2 | Library::SDL3 => true,
        _ => false,
    }
}

