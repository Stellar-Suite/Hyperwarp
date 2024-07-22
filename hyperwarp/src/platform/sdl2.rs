use std::{cmp::max, collections::HashMap};

use sdl2_sys_lite::bindings::{SDL_Event, SDL_EventType, SDL_GameControllerAxis, SDL_GameControllerButton, SDL_Joystick, SDL_MouseMotionEvent};
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

// firefox inits at 0, off is -1, on is 1
pub fn sdl2_translate_joystick_axis_value_for_trigger(value: f64) -> i16 {
    let floated = value.clamp(0.0, 1.0) * 32767.0;
    floated as i16
}

// apparently a little lying needs to be done
pub fn calc_btns_for_virtual_gamepad(buttons_count: u8) -> u8 {
    max(21, buttons_count)
}

pub fn calc_axes_for_virtual_gamepad(axes_count: u8) -> u8 {
    max(6, axes_count)
}

// https://stackoverflow.com/questions/76296734/game-controller-button-number-incompability-between-sdl2-and-other-tools-window
// https://stackoverflow.com/a/76310551
pub fn convert_update_to_sdl_form(event: &InputEvent) -> (&String, GamepadState) {
    // TODO: clean this code up
    match &event.payload {
        InputEventPayload::JoystickBrowserUpdate { id, axis: axes, buttons } => {
            let mut state = GamepadState::from_gamepad_init_specs(GamepadInitializationSpecs { axes: calc_axes_for_virtual_gamepad(axes.len() as u8) as i32, buttons: calc_btns_for_virtual_gamepad(buttons.len() as u8) as i32, hats: 0 });
            // here is a long if
            // technically written in HTML5 gamepad api button order
            if buttons.len() > 0 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_A as usize] = buttons[0];
            }
            if buttons.len() > 1 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_B as usize] = buttons[1];
            }
            if buttons.len() > 2 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_X as usize] = buttons[2];
            }
            if buttons.len() > 3 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_Y as usize] = buttons[3];
            }
            if buttons.len() > 4 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSHOULDER as usize] = buttons[4];
            }
            if buttons.len() > 5 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSHOULDER as usize] = buttons[5];
            }
            // next two correspond to axis so we ignore their button positioning
            // we rescale later
            /*if axes.len() > 4 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT as usize] = axes[4];
            }
            if axes.len() > 5 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT as usize] = axes[5];
            }*/

            // code moved to bottom
            
            if buttons.len() > 8 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_BACK as usize] = buttons[8];
            }

            if buttons.len() > 9 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_START as usize] = buttons[9];
            }

            if buttons.len() > 10 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_LEFTSTICK as usize] = buttons[10];
            }

            if buttons.len() > 11 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_RIGHTSTICK as usize] = buttons[11];
            }

            // dpad up,down,left,right

            if buttons.len() > 12 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_UP as usize] = buttons[12];
            }

            if buttons.len() > 13 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_DOWN as usize] = buttons[13];
            }

            if buttons.len() > 14 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_LEFT as usize] = buttons[14];
            }

            if buttons.len() > 15 {
                state.buttons[SDL_GameControllerButton::SDL_CONTROLLER_BUTTON_DPAD_RIGHT as usize] = buttons[15];
            }

            // normal axis

            if axes.len() > 0 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTX as usize] = axes[0];
            }

            if axes.len() > 1 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_LEFTY as usize] = axes[1];
            }

            if axes.len() > 2 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTX as usize] = axes[2];
            }

            if axes.len() > 3 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_RIGHTY as usize] = axes[3];
            }

            if axes.len() > 4 {
                // firefox actually gives trigger status
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERLEFT as usize] = axes[4];
            }

            if axes.len() > 5 {
                state.axes[SDL_GameControllerAxis::SDL_CONTROLLER_AXIS_TRIGGERRIGHT as usize] = axes[5];
            }

            (&id, state)
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