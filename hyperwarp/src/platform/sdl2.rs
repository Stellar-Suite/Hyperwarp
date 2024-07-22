use std::{cmp::max, collections::HashMap};

use sdl2_sys_lite::bindings::{SDL_Event, SDL_MouseMotionEvent, SDL_EventType, SDL_Joystick};
use stellar_protocol::protocol::{InputEvent, InputEventPayload};

use crate::{bind::sdl2_safe::SDL_PushEvent_safe, host::{hosting::HOST, input::{GamepadInitializationSpecs, GamepadState}}};

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

// apparently a little lying needs to be done
pub fn get_btns_for_virtual_gamepad(buttons_count: u8) -> u8 {
    max(21, buttons_count)
}

pub fn convert_update_to_sdl_form(event: &InputEvent) -> GamepadState {

    match event.payload {
        InputEventPayload::JoystickBrowserUpdate { id, axis, buttons } => {
            let mut state = GamepadState::from_gamepad_init_specs(GamepadInitializationSpecs { axes: axis.len() as i32, buttons: get_btns_for_virtual_gamepad(buttons.len() as u8) as i32, hats: 0 });
            // here is a long if
            if buttons.len() > 0 {
                state.buttons[0] = buttons[0];
            }
            if buttons.len() > 1 {
                state.buttons[1] = buttons[1];
            }
            if buttons.len() > 2 {
                state.buttons[2] = buttons[2];
            }
            if buttons.len() > 3 {
                state.buttons[3] = buttons[3];
            }
            if buttons.len() > 4 {
                state.buttons[4] = buttons[4];
            }
            state
        },
        _ => {
            panic!("convert_update_to_sdl_form does not handle this event {:?}", event.payload);
        }
    }    
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