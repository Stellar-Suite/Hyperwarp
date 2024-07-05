use std::collections::HashMap;

use stellar_protocol::protocol::{InputEvent, InputEventPayload, InputMetadata};

use super::{feature_flags, hosting::HOST};

// abstraction for data
pub struct Mouse {
    pub x: i32,
    pub y: i32,
    pub buttons: i32,
}

impl Mouse {
    pub fn new() -> Mouse {
        Mouse {
            x: 0,
            y: 0,
            buttons: 0,
        }
    }
}

pub struct Keyboard {
    pub scancodes_state: HashMap<i32, bool>,
    pub keycodes_state: HashMap<i32, bool>,
     // key code _> pressed
}

pub fn create_init_keyboard_state() -> HashMap<i32, bool> {
    let mut state = HashMap::new();
    for i in 0..1024 { // maybe make this 256
        state.insert(i, false);
    }
    state
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            scancodes_state: create_init_keyboard_state(),
            keycodes_state: create_init_keyboard_state(),
        }
    }

    pub fn reset(&mut self) {
        self.scancodes_state = create_init_keyboard_state();
        self.keycodes_state = create_init_keyboard_state();
    }

    pub fn calc_modifiers(&self) -> u32 {
        let mut modifiers = 0;

        

        return 0;
    }
}

pub trait Timestampable {
    fn timestamp(&mut self);
}

impl Timestampable for InputMetadata {
    fn timestamp(&mut self) {
        {
            let feature_flags = HOST.features.lock().unwrap();
            if feature_flags.sdl2_enabled {
                unsafe {
                    self.sdl2_timestamp_ticks = Some(sdl2_sys::SDL_GetTicks());
                    self.sdl2_timestamp_ticks_u64 = Some(sdl2_sys::SDL_GetTicks64());
                }
            }
            // TODO: sdl3
        }
    }
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
        // synth input event
        let real_dx = final_x - self.mouse.x;
        let real_dy = final_y - self.mouse.y;
        self.mouse.x = final_x;
        self.mouse.y = final_y;
        // synth an actual event
        self.event_queue.push(InputEvent::new(InputEventPayload::MouseMoveRelative {
            x: real_dx,
            y: real_dy,
            x_absolute: final_x,
            y_absolute: final_y,
        }));
    }

    pub fn set_key(&mut self, key: i32, state: bool) {
        self.keyboard.scancodes_state.insert(key, state);
    }

    pub fn new() -> InputManager {
        InputManager {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn flush_queue(&mut self) {
        for event in self.event_queue.drain(..) {

        }
    }

    pub fn push_event(&mut self, event: InputEvent) {
        self.event_queue.push(event);
    }
}