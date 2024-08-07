pub mod sdl2;
pub mod xlib;

pub const LIBRARY_NAME: &str = "Hyperwarp";
pub const GAMEPAD_NAME: &str = "Hyperwarp Virtual Gamepad";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

// try to avoid doing stuff here
pub const BLACKLISTED_PROCESS_NAMES: &[&str] = &[
    "env",
    "exec",
    "valgrind"
];