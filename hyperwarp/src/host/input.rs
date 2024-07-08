use std::collections::HashMap;

use stellar_protocol::protocol::{InputEvent, InputEventPayload, InputMetadata};
use stellar_shared::constants::sdl2::{is_unknown_scancode_u32, map_key_code_to_scancode_cursed_u32};

use crate::bind::sdl2_safe;

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
    // pub scancodes_state: HashMap<i32, bool>,
    pub keycodes_state: HashMap<u32, bool>,
    pub sdl2_virt_array: [u8; 513],
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
            // scancodes_state: create_init_keyboard_state(),
            keycodes_state: create_init_keyboard_state(),
            sdl2_virt_array: [0; 513],
        }
    }

    pub fn reset(&mut self) {
        // self.scancodes_state = create_init_keyboard_state();
        self.keycodes_state = create_init_keyboard_state();
        self.sdl2_virt_array.fill(0);
    }

    pub fn set_key(&mut self, key: u32, state: bool) -> Option<bool>{
        let output = self.keycodes_state.insert(key, state);

        let sdl_scancode_u32 = map_key_code_to_scancode_cursed_u32(key);
        
        if !is_unknown_scancode_u32(sdl_scancode_u32) {
            if sdl_scancode_u32 < self.sdl2_virt_array.len() as u32 {
                self.sdl2_virt_array[sdl_scancode_u32 as usize] = state as u8;
            } else {
                println!("uh oh, sdl2 scancode out of bounds {}, impossible?", sdl_scancode_u32);
            }
        }

        output
    }

    pub fn calc_modifiers(&self) -> u16 {
        let mut modifiers = 0u16;


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

    pub fn new_timestamped_input_event(payload: InputEventPayload) -> InputEvent {
        let mut input_event = InputEvent::new(payload);
        input_event.metadata.timestamp();
        input_event
    }

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
        self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::MouseMoveRelative {
            x: real_dx,
            y: real_dy,
            x_absolute: final_x,
            y_absolute: final_y,
        }));
    }

    pub fn set_key(&mut self, key: u32, state: bool) {
        if let Some(prev) = self.keyboard.set_key(key, state) {
            let repeating = prev == state;
            // we only care about these for now
            if prev != state {
                self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::KeyEvent {
                    key,
                    scancode: 0, // not used for now
                    state,
                    modifiers: self.keyboard.calc_modifiers(),
                }));
            }
        }
    }

    pub fn new() -> InputManager {
        InputManager {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn flush_queue(&mut self) {
        let feature_flags = HOST.features.lock().unwrap();
        for event in self.event_queue.drain(..) {
            match event.payload {
                InputEventPayload::KeyEvent { key, scancode, state, modifiers } => {
                    // we're going to ignore scancode for now
                    if feature_flags.sdl2_enabled {
                        let event_type = if state { sdl2_sys::SDL_EventType::SDL_KEYDOWN } else { sdl2_sys::SDL_EventType::SDL_KEYUP };
                        let sdl_state = if state { sdl2_sys::SDL_PRESSED } else { sdl2_sys::SDL_RELEASED };
                        let scancode = unsafe {
                            sdl2_sys::SDL_GetScancodeFromKey(key) // hack for now
                        };
                        let keysym = 0;
                        let wid = HOST.get_behavior().get_largest_sdl2_window_id().unwrap_or(0);
                        let timestamp = event.metadata.sdl2_timestamp_ticks.unwrap_or(0);

                        let mut event = sdl2_sys::SDL_Event {
                            key: sdl2_sys::SDL_KeyboardEvent { 
                                type_: event_type as u32,
                                timestamp: timestamp,
                                windowID: wid,
                                state: sdl_state as u8,
                                repeat: 0,
                                padding2: 0,
                                padding3: 0,
                                keysym: sdl2_sys::SDL_Keysym { scancode: scancode, sym: key, mod_: modifiers, unused: 0 } }
                        };

                        unsafe {
                            // TODO: handle errors
                            // https://github.com/Rust-SDL2/rust-sdl2/blob/dba66e80b14e16de309df49df0c20fdaf35b8c67/src/sdl2/event.rs#L2812
                            // also maybe don't use unsafe directly?
                            let result_ok = sdl2_sys::SDL_PushEvent(&mut event);
                            if result_ok != 1 {
                                let error_str = sdl2_safe::SDL_GetError_safe();
                                println!("uh oh event push error: {}, {} {}", error_str, wid, timestamp);
                            }
                        }

                        println!("pushed event new kbd event");
                    }
                }
                _ => {
                    println!("unhandled event: {:?}", event);
                }
            }
        }
    }

    pub fn push_event(&mut self, event: InputEvent) {
        self.event_queue.push(event);
    }

    pub fn process_event(&mut self, event: InputEvent) {
        let mut new_event = event.clone();
        // is there any point in doing this?
        new_event.metadata.timestamp();

        match event.payload {
            InputEventPayload::KeyEvent { key, scancode, state, modifiers } => {
                self.set_key(key, state);
            },
            _ => {
                self.event_queue.push(new_event);
            }
        }

    }
}