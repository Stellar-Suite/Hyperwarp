// https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
// https://github.com/torvalds/linux/blob/f8f9c1f4d0c7a64600e2ca312dec824a0bc2f1da/include/uapi/linux/input-event-codes.h#L356
// crossref with
// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button

pub const WEB_BTN_TO_LINUX_BUTTON: [u32; 5] = [
    0x110, // left BTN_LEFT
    0x112, // middle BTN_MIDDLE
    0x111, // right BTN_RIGHT
    0x116, // back BTN_BACK (browser back)
    0x115 // forward BTN_FORWARD (browser forward)
];

// I hope no new mouse buttons are added for a while.

// claude wrote this
pub fn decode_keyevent_code_to_evdev(code: &str) -> u32 {
    match code {
        // Letters
        "KeyA" => 30,
        "KeyB" => 48,
        "KeyC" => 46,
        "KeyD" => 32,
        "KeyE" => 18,
        "KeyF" => 33,
        "KeyG" => 34,
        "KeyH" => 35,
        "KeyI" => 23,
        "KeyJ" => 36,
        "KeyK" => 37,
        "KeyL" => 38,
        "KeyM" => 50,
        "KeyN" => 49,
        "KeyO" => 24,
        "KeyP" => 25,
        "KeyQ" => 16,
        "KeyR" => 19,
        "KeyS" => 31,
        "KeyT" => 20,
        "KeyU" => 22,
        "KeyV" => 47,
        "KeyW" => 17,
        "KeyX" => 45,
        "KeyY" => 21,
        "KeyZ" => 44,
        
        // Numbers
        "Digit0" => 11,
        "Digit1" => 2,
        "Digit2" => 3,
        "Digit3" => 4,
        "Digit4" => 5,
        "Digit5" => 6,
        "Digit6" => 7,
        "Digit7" => 8,
        "Digit8" => 9,
        "Digit9" => 10,
        
        // Function keys
        "F1" => 59,
        "F2" => 60,
        "F3" => 61,
        "F4" => 62,
        "F5" => 63,
        "F6" => 64,
        "F7" => 65,
        "F8" => 66,
        "F9" => 67,
        "F10" => 68,
        "F11" => 87,
        "F12" => 88,
        "F13" => 183,
        "F14" => 184,
        "F15" => 185,
        "F16" => 186,
        "F17" => 187,
        "F18" => 188,
        "F19" => 189,
        "F20" => 190,
        "F21" => 191,
        "F22" => 192,
        "F23" => 193,
        "F24" => 194,
        
        // Special keys
        "Escape" => 1,
        "Backspace" => 14,
        "Tab" => 15,
        "Enter" => 28,
        "Space" => 57,
        "CapsLock" => 58,
        "ShiftLeft" => 42,
        "ShiftRight" => 54,
        "ControlLeft" => 29,
        "ControlRight" => 97,
        "AltLeft" => 56,
        "AltRight" => 100,
        "MetaLeft" => 125,
        "MetaRight" => 126,
        "ContextMenu" => 127,
        
        // Punctuation
        "Minus" => 12,
        "Equal" => 13,
        "BracketLeft" => 26,
        "BracketRight" => 27,
        "Backslash" => 43,
        "Semicolon" => 39,
        "Quote" => 40,
        "Backquote" => 41,
        "Comma" => 51,
        "Period" => 52,
        "Slash" => 53,
        "IntlBackslash" => 86,
        
        // Navigation
        "Insert" => 110,
        "Delete" => 111,
        "Home" => 102,
        "End" => 107,
        "PageUp" => 104,
        "PageDown" => 109,
        "ArrowUp" => 103,
        "ArrowDown" => 108,
        "ArrowLeft" => 105,
        "ArrowRight" => 106,
        
        // Lock keys
        "NumLock" => 69,
        "ScrollLock" => 70,
        
        // Numpad
        "Numpad0" => 82,
        "Numpad1" => 79,
        "Numpad2" => 80,
        "Numpad3" => 81,
        "Numpad4" => 75,
        "Numpad5" => 76,
        "Numpad6" => 77,
        "Numpad7" => 71,
        "Numpad8" => 72,
        "Numpad9" => 73,
        "NumpadAdd" => 78,
        "NumpadSubtract" => 74,
        "NumpadMultiply" => 55,
        "NumpadDivide" => 98,
        "NumpadDecimal" => 83,
        "NumpadEnter" => 96,
        "NumpadEqual" => 117,
        "NumpadComma" => 121,
        
        // Media keys
        "AudioVolumeMute" => 113,
        "AudioVolumeDown" => 114,
        "AudioVolumeUp" => 115,
        "MediaPlayPause" => 164,
        "MediaStop" => 166,
        "MediaTrackNext" => 163,
        "MediaTrackPrevious" => 165,
        "MediaRecord" => 167,
        "MediaRewind" => 168,
        "MediaFastForward" => 208,
        
        // Browser keys
        "BrowserBack" => 158,
        "BrowserForward" => 159,
        "BrowserRefresh" => 173,
        "BrowserStop" => 128,
        "BrowserSearch" => 217,
        "BrowserFavorites" => 156,
        "BrowserHome" => 172,
        
        // System keys
        "Power" => 116,
        "Sleep" => 142,
        "WakeUp" => 143,
        "Eject" => 161,
        "Pause" => 119,
        "PrintScreen" => 99,
        
        // Application keys
        "LaunchApp1" => 148,
        "LaunchApp2" => 149,
        "LaunchMail" => 155,
        "LaunchMediaPlayer" => 226,
        "LaunchCalculator" => 140,
        "LaunchFileManager" => 144,
        
        // Other keys
        "Help" => 138,
        "Undo" => 131,
        "Cut" => 137,
        "Copy" => 133,
        "Paste" => 135,
        "Find" => 136,
        "Open" => 134,
        "Props" => 130,
        "Again" => 129,
        "Select" => 0, // Not directly mapped
        
        // Asian language keys
        "Lang1" => 122, // Hangul
        "Lang2" => 123, // Hanja
        "Lang3" => 90,  // Katakana
        "Lang4" => 91,  // Hiragana
        "Lang5" => 85,  // Zenkaku/Hankaku
        "Convert" => 92, // Henkan
        "NonConvert" => 94, // Muhenkan
        "KanaMode" => 93, // Katakana/Hiragana
        "IntlRo" => 89,
        "IntlYen" => 124,
        
        // Default for unknown keys
        _ => 0,
    }
}