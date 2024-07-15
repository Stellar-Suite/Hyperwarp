use sdl2_sys_lite::bindings::{SDL_Event, SDL_MouseMotionEvent, SDL_EventType};
use stellar_protocol::protocol::{InputEvent, InputEventPayload};

use crate::{bind::sdl2_safe::SDL_PushEvent_safe, host::hosting::HOST};

pub fn process_event(input_event: &InputEvent) {
    // TODO: actually use? it will be when we need to modularize input manager
}

pub fn sdl2_translate_mouse_state(state: u8) -> u8 {
    state // don't need to atm?
}