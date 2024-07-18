use std::collections::HashMap;

use sdl2_sys_lite::bindings::{SDL_Event, SDL_MouseMotionEvent, SDL_EventType, SDL_Joystick};
use stellar_protocol::protocol::{InputEvent, InputEventPayload};

use crate::{bind::sdl2_safe::SDL_PushEvent_safe, host::hosting::HOST};

pub fn process_event(input_event: &InputEvent) {
    // TODO: actually use? it will be when we need to modularize input manager
}

pub fn sdl2_translate_mouse_state(state: u8) -> u8 {
    state // don't need to atm?
}

pub fn sdl2_translate_joystick_axis_value(value: f64) -> i16 {
    let floated = (value * 32767.0).clamp(-32768.0, 32767.0);
    floated as i16
}

/*struct SDLPlatform {
    joystick_vec: Vec<SDL_Joystick>,
    joystick_kv: HashMap<u32, String>
}

impl Default for SDLPlatform {
    fn default() -> Self {
        Self { joystick_vec: Default::default(), joystick_kv: Default::default() }
    }
    
}

pub trait ConstructHack {
    fn construct_hack() -> Self;
}

impl ConstructHack for SDL_Joystick {
    fn construct_hack() -> Self {
        Self {
            _unused: [],
        }
    }
}

impl SDLPlatform {
    pub fn create_joystick(&mut self, value: &str) -> u32 {
        let joystick = SDL_Joystick {
            _unused: [],
        };

        self.joystick_vec.push(joystick);
        let joystick_ref = self.joystick_vec.last().unwrap();
        joystick_ref as *const SDL_Joystick as u32
    }
}*/