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

// https://www.toptal.com/developers/keycode/table
// this sucks because "This table is based on the US standard 101 keyboard, the values may vary based on a specific hardware."

// VERY IMPERFECT KEYBOARD UTILITY
pub fn decode_keyevent_code(code: &str) -> SDL_Scancode {
    match code {
        "KeyA" => SDL_Scancode::SDL_SCANCODE_A,
        "KeyB" => SDL_Scancode::SDL_SCANCODE_B,
        "KeyC" => SDL_Scancode::SDL_SCANCODE_C,
        "KeyD" => SDL_Scancode::SDL_SCANCODE_D,
        "KeyE" => SDL_Scancode::SDL_SCANCODE_E,
        "KeyF" => SDL_Scancode::SDL_SCANCODE_F,
        "KeyG" => SDL_Scancode::SDL_SCANCODE_G,
        "KeyH" => SDL_Scancode::SDL_SCANCODE_H,
        "KeyI" => SDL_Scancode::SDL_SCANCODE_I,
        "KeyJ" => SDL_Scancode::SDL_SCANCODE_J,
        "KeyK" => SDL_Scancode::SDL_SCANCODE_K,
        "KeyL" => SDL_Scancode::SDL_SCANCODE_L,
        "KeyM" => SDL_Scancode::SDL_SCANCODE_M,
        "KeyN" => SDL_Scancode::SDL_SCANCODE_N,
        "KeyO" => SDL_Scancode::SDL_SCANCODE_O,
        "KeyP" => SDL_Scancode::SDL_SCANCODE_P,
        "KeyQ" => SDL_Scancode::SDL_SCANCODE_Q,
        "KeyR" => SDL_Scancode::SDL_SCANCODE_R,
        "KeyS" => SDL_Scancode::SDL_SCANCODE_S,
        "KeyT" => SDL_Scancode::SDL_SCANCODE_T,
        "KeyU" => SDL_Scancode::SDL_SCANCODE_U,
        "KeyV" => SDL_Scancode::SDL_SCANCODE_V,
        "KeyW" => SDL_Scancode::SDL_SCANCODE_W,
        "KeyX" => SDL_Scancode::SDL_SCANCODE_X,
        "KeyY" => SDL_Scancode::SDL_SCANCODE_Y,
        "KeyZ" => SDL_Scancode::SDL_SCANCODE_Z,
        "Unidentified" => SDL_Scancode::SDL_SCANCODE_UNKNOWN,
        "Cancel" => SDL_Scancode::SDL_SCANCODE_CANCEL,
        "Backspace" => SDL_Scancode::SDL_SCANCODE_BACKSPACE,
        "Tab" => SDL_Scancode::SDL_SCANCODE_TAB,
        "NumLock" => SDL_Scancode::SDL_SCANCODE_CLEAR,
        "Enter" => SDL_Scancode::SDL_SCANCODE_RETURN, // aka enter
        "ShiftLeft" => SDL_Scancode::SDL_SCANCODE_LSHIFT,
        "ShiftRight" => SDL_Scancode::SDL_SCANCODE_RSHIFT,
        "ControlLeft" => SDL_Scancode::SDL_SCANCODE_LCTRL,
        "ControlRight" => SDL_Scancode::SDL_SCANCODE_RCTRL,
        "AltLeft" => SDL_Scancode::SDL_SCANCODE_LALT,
        "AltRight" => SDL_Scancode::SDL_SCANCODE_RALT,
        "Pause" => SDL_Scancode::SDL_SCANCODE_PAUSE,
        "CapsLock" => SDL_Scancode::SDL_SCANCODE_CAPSLOCK,
        // Lang1 and Lang2 ommited
        "Escape" => SDL_Scancode::SDL_SCANCODE_ESCAPE,
        "Space" => SDL_Scancode::SDL_SCANCODE_SPACE,
        "PageUp" => SDL_Scancode::SDL_SCANCODE_PAGEUP,
        "PageDown" => SDL_Scancode::SDL_SCANCODE_PAGEDOWN,
        "End" => SDL_Scancode::SDL_SCANCODE_END,
        "Home" => SDL_Scancode::SDL_SCANCODE_HOME,
        "ArrowLeft" => SDL_Scancode::SDL_SCANCODE_LEFT,
        "ArrowRight" => SDL_Scancode::SDL_SCANCODE_RIGHT,
        "ArrowUp" => SDL_Scancode::SDL_SCANCODE_UP,
        "ArrowDown" => SDL_Scancode::SDL_SCANCODE_DOWN,
        "F13" => SDL_Scancode::SDL_SCANCODE_F13, // this exists?
        "Numpad0" => SDL_Scancode::SDL_SCANCODE_KP_0,
        "NumpadDecimal" => SDL_Scancode::SDL_SCANCODE_DELETE,
        "Digit0" => SDL_Scancode::SDL_SCANCODE_0,
        "Digit1" => SDL_Scancode::SDL_SCANCODE_1,
        "Digit2" => SDL_Scancode::SDL_SCANCODE_2,
        "Digit3" => SDL_Scancode::SDL_SCANCODE_3,
        "Digit4" => SDL_Scancode::SDL_SCANCODE_4,
        "Digit5" => SDL_Scancode::SDL_SCANCODE_5,
        "Digit6" => SDL_Scancode::SDL_SCANCODE_6,
        "Digit7" => SDL_Scancode::SDL_SCANCODE_7,
        "Digit8" => SDL_Scancode::SDL_SCANCODE_8,
        "Digit9" => SDL_Scancode::SDL_SCANCODE_9,
        "Period" => SDL_Scancode::SDL_SCANCODE_PERIOD,
        "Comma" => SDL_Scancode::SDL_SCANCODE_COMMA,
        "Slash" => SDL_Scancode::SDL_SCANCODE_SLASH,
        "Semicolon" => SDL_Scancode::SDL_SCANCODE_SEMICOLON,
        "Quote" => SDL_Scancode::SDL_SCANCODE_APOSTROPHE,
        "LeftBracket" => SDL_Scancode::SDL_SCANCODE_LEFTBRACKET,
        "Backslash" => SDL_Scancode::SDL_SCANCODE_BACKSLASH,
        "RightBracket" => SDL_Scancode::SDL_SCANCODE_RIGHTBRACKET,
        // will add is more people complain

        _ => SDL_Scancode::SDL_SCANCODE_UNKNOWN
    }
}

pub fn decode_keyevent_code_int(code: &str) -> i32 {
    decode_keyevent_code(code) as i32
}

pub fn decode_keyevent_key(key: &str) -> SDL_KeyCode {
    match key {
        "a" => SDL_KeyCode::SDLK_a,
        "b" => SDL_KeyCode::SDLK_b,
        "c" => SDL_KeyCode::SDLK_c,
        "d" => SDL_KeyCode::SDLK_d,
        "e" => SDL_KeyCode::SDLK_e,
        "f" => SDL_KeyCode::SDLK_f,
        "g" => SDL_KeyCode::SDLK_g,
        "h" => SDL_KeyCode::SDLK_h,
        "i" => SDL_KeyCode::SDLK_i,
        "j" => SDL_KeyCode::SDLK_j,
        "k" => SDL_KeyCode::SDLK_k,
        "l" => SDL_KeyCode::SDLK_l,
        "m" => SDL_KeyCode::SDLK_m,
        "n" => SDL_KeyCode::SDLK_n,
        "o" => SDL_KeyCode::SDLK_o,
        "p" => SDL_KeyCode::SDLK_p,
        "q" => SDL_KeyCode::SDLK_q,
        "r" => SDL_KeyCode::SDLK_r,
        "s" => SDL_KeyCode::SDLK_s,
        "t" => SDL_KeyCode::SDLK_t,
        "u" => SDL_KeyCode::SDLK_u,
        "v" => SDL_KeyCode::SDLK_v,
        "w" => SDL_KeyCode::SDLK_w,
        "x" => SDL_KeyCode::SDLK_x,
        "y" => SDL_KeyCode::SDLK_y,
        "z" => SDL_KeyCode::SDLK_z,
        "1" => SDL_KeyCode::SDLK_1,
        "2" => SDL_KeyCode::SDLK_2,
        "3" => SDL_KeyCode::SDLK_3,
        "4" => SDL_KeyCode::SDLK_4,
        "5" => SDL_KeyCode::SDLK_5,
        "6" => SDL_KeyCode::SDLK_6,
        "7" => SDL_KeyCode::SDLK_7,
        "8" => SDL_KeyCode::SDLK_8,
        "9" => SDL_KeyCode::SDLK_9,
        "0" => SDL_KeyCode::SDLK_0,
        "\n" => SDL_KeyCode::SDLK_RETURN,
        " " => SDL_KeyCode::SDLK_SPACE,
        "-" => SDL_KeyCode::SDLK_MINUS,
        "=" => SDL_KeyCode::SDLK_EQUALS,
        _ => SDL_KeyCode::SDLK_UNKNOWN
    }
}

