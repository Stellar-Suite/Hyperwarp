use sdl2_sys_lite::bindings::{SDL_Event, SDL_MouseMotionEvent, SDL_EventType};
use stellar_protocol::protocol::{InputEvent, InputEventPayload};

use crate::{bind::sdl2_safe::SDL_PushEvent_safe, host::hosting::HOST};

pub fn process_event(input_event: &InputEvent) {
    match input_event.payload {
        InputEventPayload::MouseMoveRelative { x, y, x_absolute, y_absolute } => {
            let mut mouse_event = SDL_Event {
                motion: SDL_MouseMotionEvent {
                    type_: SDL_EventType::SDL_MOUSEMOTION as u32,
                    timestamp: input_event.metadata.sdl2_timestamp_ticks.unwrap(),
                    windowID: 0,
                    which: 0,
                    state: 0,
                    x: x_absolute,
                    y: y_absolute,
                    xrel: x,
                    yrel: y,
                }
            };
            SDL_PushEvent_safe(&mut mouse_event);
        },
        InputEventPayload::MouseMoveAbsolute(x, y) => {
            // pov: drawing tablets
            // TODO: I don't have one so I can test but maybe a touchscreen will suffice?
        },
        _ => {
            if HOST.config.debug_mode {
                println!("unknown input event payload {:?}", input_event);
            }
        }
    }
}