use sdl2_sys::{SDL_KeyCode, SDL_Scancode};

use crate::utils::manual_types::libc::ENUM_TYPE;

// TRUE FALSE are enums

// usually a signed integer
pub const SDL_TRUE: ENUM_TYPE = 1;
pub const SDL_FALSE: ENUM_TYPE = 0;

// portable key mods, from SDL
// https://wiki.libsdl.org/SDL2/SDL_Keymod

pub const KMOD_NONE: u32 = 0x0000;
pub const KMOD_LSHIFT: u32 = 0x0001;
pub const KMOD_RSHIFT: u32 = 0x0002;
pub const KMOD_LCTRL: u32 = 0x0040;
pub const KMOD_RCTRL: u32 = 0x0080;
pub const KMOD_LALT: u32 = 0x0100;
pub const KMOD_RALT: u32 = 0x0200;
pub const KMOD_LGUI: u32 = 0x0400;
pub const KMOD_RGUI: u32 = 0x0800;
pub const KMOD_NUM: u32 = 0x1000;
pub const KMOD_CAPS: u32 = 0x2000;
pub const KMOD_MODE: u32 = 0x4000;
pub const KMOD_SCROLL: u32 = 0x8000;

pub const KMOD_CTRL: u32 = KMOD_LCTRL | KMOD_RCTRL;
pub const KMOD_SHIFT: u32 = KMOD_LSHIFT | KMOD_RSHIFT;
pub const KMOD_ALT: u32 = KMOD_LALT | KMOD_RALT;
pub const KMOD_GUI: u32 = KMOD_LGUI | KMOD_RGUI;

pub const KMOD_RESERVED: u32 = KMOD_SCROLL; /* "This is for source-level compatibility with SDL 2.0.0."" */