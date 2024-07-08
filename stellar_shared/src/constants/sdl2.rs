// portable key mods, from SDL
// https://wiki.libsdl.org/SDL2/SDL_Keymod

use std::mem::transmute;

use crate::vendor::sdl_bindings::{SDL_KeyCode, SDL_Scancode};

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

pub fn decode_keyevent_key_int(code: &str) -> i32 {
    decode_keyevent_key(code) as i32
}

pub fn map_keycode_to_scancode(code: SDL_KeyCode) -> SDL_Scancode {
    match code {
        SDL_KeyCode::SDLK_a => SDL_Scancode::SDL_SCANCODE_A,
        SDL_KeyCode::SDLK_b => SDL_Scancode::SDL_SCANCODE_B,
        SDL_KeyCode::SDLK_c => SDL_Scancode::SDL_SCANCODE_C,
        SDL_KeyCode::SDLK_d => SDL_Scancode::SDL_SCANCODE_D,
        SDL_KeyCode::SDLK_e => SDL_Scancode::SDL_SCANCODE_E,
        SDL_KeyCode::SDLK_f => SDL_Scancode::SDL_SCANCODE_F,
        SDL_KeyCode::SDLK_g => SDL_Scancode::SDL_SCANCODE_G,
        SDL_KeyCode::SDLK_h => SDL_Scancode::SDL_SCANCODE_H,
        SDL_KeyCode::SDLK_i => SDL_Scancode::SDL_SCANCODE_I,
        SDL_KeyCode::SDLK_j => SDL_Scancode::SDL_SCANCODE_J,
        SDL_KeyCode::SDLK_k => SDL_Scancode::SDL_SCANCODE_K,
        SDL_KeyCode::SDLK_l => SDL_Scancode::SDL_SCANCODE_L,
        SDL_KeyCode::SDLK_m => SDL_Scancode::SDL_SCANCODE_M,
        SDL_KeyCode::SDLK_n => SDL_Scancode::SDL_SCANCODE_N,
        SDL_KeyCode::SDLK_o => SDL_Scancode::SDL_SCANCODE_O,
        SDL_KeyCode::SDLK_p => SDL_Scancode::SDL_SCANCODE_P,
        SDL_KeyCode::SDLK_q => SDL_Scancode::SDL_SCANCODE_Q,
        SDL_KeyCode::SDLK_r => SDL_Scancode::SDL_SCANCODE_R,
        SDL_KeyCode::SDLK_s => SDL_Scancode::SDL_SCANCODE_S,
        SDL_KeyCode::SDLK_t => SDL_Scancode::SDL_SCANCODE_T,
        SDL_KeyCode::SDLK_u => SDL_Scancode::SDL_SCANCODE_U,
        SDL_KeyCode::SDLK_v => SDL_Scancode::SDL_SCANCODE_V,
        SDL_KeyCode::SDLK_w => SDL_Scancode::SDL_SCANCODE_W,
        SDL_KeyCode::SDLK_x => SDL_Scancode::SDL_SCANCODE_X,
        SDL_KeyCode::SDLK_y => SDL_Scancode::SDL_SCANCODE_Y,
        SDL_KeyCode::SDLK_z => SDL_Scancode::SDL_SCANCODE_Z,
        SDL_KeyCode::SDLK_1 => SDL_Scancode::SDL_SCANCODE_1,
        SDL_KeyCode::SDLK_2 => SDL_Scancode::SDL_SCANCODE_2,
        SDL_KeyCode::SDLK_3 => SDL_Scancode::SDL_SCANCODE_3,
        SDL_KeyCode::SDLK_4 => SDL_Scancode::SDL_SCANCODE_4,
        SDL_KeyCode::SDLK_5 => SDL_Scancode::SDL_SCANCODE_5,
        SDL_KeyCode::SDLK_6 => SDL_Scancode::SDL_SCANCODE_6,
        SDL_KeyCode::SDLK_7 => SDL_Scancode::SDL_SCANCODE_7,
        SDL_KeyCode::SDLK_8 => SDL_Scancode::SDL_SCANCODE_8,
        SDL_KeyCode::SDLK_9 => SDL_Scancode::SDL_SCANCODE_9,
        SDL_KeyCode::SDLK_0 => SDL_Scancode::SDL_SCANCODE_0,
        SDL_KeyCode::SDLK_RETURN => SDL_Scancode::SDL_SCANCODE_RETURN,
        SDL_KeyCode::SDLK_SPACE => SDL_Scancode::SDL_SCANCODE_SPACE,
        SDL_KeyCode::SDLK_MINUS => SDL_Scancode::SDL_SCANCODE_MINUS,
        SDL_KeyCode::SDLK_EQUALS => SDL_Scancode::SDL_SCANCODE_EQUALS,
        SDL_KeyCode::SDLK_BACKSPACE => SDL_Scancode::SDL_SCANCODE_BACKSPACE,
        SDL_KeyCode::SDLK_TAB => SDL_Scancode::SDL_SCANCODE_TAB,
        SDL_KeyCode::SDLK_LSHIFT => SDL_Scancode::SDL_SCANCODE_LSHIFT,
        SDL_KeyCode::SDLK_RSHIFT => SDL_Scancode::SDL_SCANCODE_RSHIFT,
        SDL_KeyCode::SDLK_LCTRL => SDL_Scancode::SDL_SCANCODE_LCTRL,
        SDL_KeyCode::SDLK_RCTRL => SDL_Scancode::SDL_SCANCODE_RCTRL,
        SDL_KeyCode::SDLK_LALT => SDL_Scancode::SDL_SCANCODE_LALT,
        SDL_KeyCode::SDLK_RALT => SDL_Scancode::SDL_SCANCODE_RALT,
        SDL_KeyCode::SDLK_LGUI => SDL_Scancode::SDL_SCANCODE_LGUI,
        SDL_KeyCode::SDLK_RGUI => SDL_Scancode::SDL_SCANCODE_RGUI,
        SDL_KeyCode::SDLK_CAPSLOCK => SDL_Scancode::SDL_SCANCODE_CAPSLOCK,
        SDL_KeyCode::SDLK_F1 => SDL_Scancode::SDL_SCANCODE_F1,
        SDL_KeyCode::SDLK_F2 => SDL_Scancode::SDL_SCANCODE_F2,
        SDL_KeyCode::SDLK_F3 => SDL_Scancode::SDL_SCANCODE_F3,
        SDL_KeyCode::SDLK_F4 => SDL_Scancode::SDL_SCANCODE_F4,
        SDL_KeyCode::SDLK_F5 => SDL_Scancode::SDL_SCANCODE_F5,
        SDL_KeyCode::SDLK_F6 => SDL_Scancode::SDL_SCANCODE_F6,
        SDL_KeyCode::SDLK_F7 => SDL_Scancode::SDL_SCANCODE_F7,
        SDL_KeyCode::SDLK_F8 => SDL_Scancode::SDL_SCANCODE_F8,
        SDL_KeyCode::SDLK_F9 => SDL_Scancode::SDL_SCANCODE_F9,
        SDL_KeyCode::SDLK_F10 => SDL_Scancode::SDL_SCANCODE_F10,
        SDL_KeyCode::SDLK_F11 => SDL_Scancode::SDL_SCANCODE_F11,
        SDL_KeyCode::SDLK_F12 => SDL_Scancode::SDL_SCANCODE_F12,
        SDL_KeyCode::SDLK_PRINTSCREEN => SDL_Scancode::SDL_SCANCODE_PRINTSCREEN,
        SDL_KeyCode::SDLK_ESCAPE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_EXCLAIM => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_QUOTEDBL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_HASH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PERCENT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_DOLLAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AMPERSAND => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_QUOTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_LEFTPAREN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_RIGHTPAREN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_ASTERISK => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PLUS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_COMMA => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PERIOD => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SLASH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_COLON => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SEMICOLON => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_LESS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_GREATER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_QUESTION => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_LEFTBRACKET => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_BACKSLASH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_RIGHTBRACKET => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CARET => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_UNDERSCORE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_BACKQUOTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SCROLLLOCK => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PAUSE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_INSERT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_HOME => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PAGEUP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_DELETE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_END => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PAGEDOWN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_RIGHT => SDL_Scancode::SDL_SCANCODE_RIGHT,
        SDL_KeyCode::SDLK_LEFT => SDL_Scancode::SDL_SCANCODE_LEFT,
        SDL_KeyCode::SDLK_DOWN => SDL_Scancode::SDL_SCANCODE_DOWN,
        SDL_KeyCode::SDLK_UP => SDL_Scancode::SDL_SCANCODE_UP,
        SDL_KeyCode::SDLK_NUMLOCKCLEAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_DIVIDE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MULTIPLY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MINUS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_PLUS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_ENTER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_1 => SDL_Scancode::SDL_SCANCODE_KP_1,
        SDL_KeyCode::SDLK_KP_2 => SDL_Scancode::SDL_SCANCODE_KP_2,
        SDL_KeyCode::SDLK_KP_3 => SDL_Scancode::SDL_SCANCODE_KP_3,
        SDL_KeyCode::SDLK_KP_4 => SDL_Scancode::SDL_SCANCODE_KP_4,
        SDL_KeyCode::SDLK_KP_5 => SDL_Scancode::SDL_SCANCODE_KP_5,
        SDL_KeyCode::SDLK_KP_6 => SDL_Scancode::SDL_SCANCODE_KP_6,
        SDL_KeyCode::SDLK_KP_7 => SDL_Scancode::SDL_SCANCODE_KP_7,
        SDL_KeyCode::SDLK_KP_8 => SDL_Scancode::SDL_SCANCODE_KP_8,
        SDL_KeyCode::SDLK_KP_9 => SDL_Scancode::SDL_SCANCODE_KP_9,
        SDL_KeyCode::SDLK_KP_0 => SDL_Scancode::SDL_SCANCODE_KP_0,
        SDL_KeyCode::SDLK_KP_PERIOD => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_APPLICATION => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_POWER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_EQUALS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F13 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F14 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F15 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F16 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F17 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F18 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F19 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F20 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F21 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F22 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F23 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_F24 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_EXECUTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_HELP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_MENU => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SELECT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_STOP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AGAIN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_UNDO => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CUT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_COPY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PASTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_FIND => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_MUTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_VOLUMEUP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_VOLUMEDOWN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_COMMA => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_EQUALSAS400 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_ALTERASE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SYSREQ => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CANCEL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CLEAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_PRIOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_RETURN2 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SEPARATOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_OUT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_OPER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CLEARAGAIN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CRSEL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_EXSEL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_00 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_000 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_THOUSANDSSEPARATOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_DECIMALSEPARATOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CURRENCYUNIT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CURRENCYSUBUNIT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_LEFTPAREN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_RIGHTPAREN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_LEFTBRACE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_RIGHTBRACE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_TAB => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_BACKSPACE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_A => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_B => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_C => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_D => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_E => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_F => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_XOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_POWER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_PERCENT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_LESS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_GREATER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_AMPERSAND => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_DBLAMPERSAND => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_VERTICALBAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_DBLVERTICALBAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_COLON => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_HASH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_SPACE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_AT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_EXCLAM => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMSTORE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMRECALL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMCLEAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMADD => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMSUBTRACT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMMULTIPLY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_MEMDIVIDE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_PLUSMINUS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_CLEAR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_CLEARENTRY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_BINARY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_OCTAL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_DECIMAL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KP_HEXADECIMAL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_MODE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIONEXT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOPREV => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOSTOP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOPLAY => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOMUTE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_MEDIASELECT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_WWW => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_MAIL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CALCULATOR => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_COMPUTER => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_SEARCH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_HOME => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_BACK => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_FORWARD => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_STOP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_REFRESH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AC_BOOKMARKS => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_BRIGHTNESSDOWN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_BRIGHTNESSUP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_DISPLAYSWITCH => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KBDILLUMTOGGLE => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KBDILLUMDOWN => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_KBDILLUMUP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_EJECT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SLEEP => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_APP1 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_APP2 => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOREWIND => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_AUDIOFASTFORWARD => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SOFTLEFT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_SOFTRIGHT => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_CALL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_ENDCALL => SDL_Scancode::SDL_SCANCODE_UNKNOWN, // TODO
        SDL_KeyCode::SDLK_UNKNOWN => SDL_Scancode::SDL_SCANCODE_UNKNOWN,
    }
}

pub fn is_unknwon(scancode: SDL_Scancode) -> bool {
    match scancode {
        SDL_Scancode::SDL_SCANCODE_UNKNOWN => true,
        _ => false,
    }
}

pub fn is_unknown_keycode_u32(scancode: u32) -> bool {
    scancode == SDL_KeyCode::SDLK_UNKNOWN as u32
}

pub fn is_unknown_scancode_u32(scancode: u32) -> bool {
    scancode == SDL_Scancode::SDL_SCANCODE_UNKNOWN as u32
}

pub fn map_key_code_to_scancode_cursed(code: u32) -> SDL_Scancode {
    if is_unknown_keycode_u32(code) {
        return SDL_Scancode::SDL_SCANCODE_UNKNOWN;
    }
    let keycode: SDL_KeyCode = unsafe {
        transmute(code) 
    };
    let scancode_enum = map_keycode_to_scancode(keycode);
    scancode_enum
}

pub fn map_key_code_to_scancode_cursed_u32(code: u32) -> u32 {
    map_key_code_to_scancode_cursed(code) as u32
}