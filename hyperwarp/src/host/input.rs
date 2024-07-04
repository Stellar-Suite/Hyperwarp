use std::collections::HashMap;

use super::hosting::HOST;

// abstraction for data
pub struct Mouse {
    pub x: i32,
    pub y: i32,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            x: 0,
            y: 0,
        }
    }
}

pub struct Keyboard {
    pub state: HashMap<i32, bool>, // key code _> pressed
}

pub fn create_init_keyboard_state() -> HashMap<i32, bool> {
    let mut state = HashMap::new();
    for i in 0..256 {
        state.insert(i, false);
    }
    state
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            state: create_init_keyboard_state(),
        }
    }
}

pub struct InputEvent {
    pub payload: InputEventPayload,
}

pub struct InputMetadata {
    pub sdl2_timestamp_ticks: Option<u32>,
    pub sdl2_timestamp_ticks_u64: Option<u64>,
    pub sdl3_timestamp_ticks: Option<u32>,
    pub sdl3_timestamp_ticks_u64: Option<u64>,
}



pub enum InputEventPayload {
    MouseMoveRelative(i32, i32),
    MouseMoveAbsolute(i32, i32),
}

pub struct InputManager {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    // TODO: vec of gamepads
    pub event_queue: Vec<InputEvent>,
}

impl InputManager {
    pub fn move_mouse_absolute(&mut self, x: i32, y: i32) {
        
    }

    pub fn move_mouse_relative(&mut self, x: i32, y: i32) {
        let mut final_x = self.mouse.x + x;
        let mut final_y = self.mouse.y + y;
        // clamp
        if let Some((width, height)) = HOST.get_behavior().get_fb_size() {
            final_x = final_x.clamp(0, width as i32);
            final_y = final_y.clamp(0, height as i32);
        }
        if self.mouse.x == final_x && self.mouse.y == final_y {
            return;
        }
        self.mouse.x = final_x;
        self.mouse.y = final_y;
        
    }

    pub fn set_key(&mut self, key: i32, state: bool) {
        self.keyboard.state.insert(key, state);
    }

    pub fn new() -> InputManager {
        InputManager {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            event_queue: Vec::new(),
        }
    }
}