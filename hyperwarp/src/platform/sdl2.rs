use sdl2_sys::SDL_PushEvent;

use crate::host::{hosting::HOST, input::{InputEvent, InputEventPayload}};

pub fn process_event(input_event: &InputEvent) {
    match input_event.payload {
        InputEventPayload::MouseMoveRelative { x, y, x_absolute, y_absolute } => {
            let mut mouse_event = sdl2_sys::SDL_Event {
                motion: sdl2_sys::SDL_MouseMotionEvent {
                    type_: sdl2_sys::SDL_EventType::SDL_MOUSEMOTION as u32,
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
            unsafe {
                SDL_PushEvent(&mut mouse_event);
            }
        },
        InputEventPayload::MouseMoveAbsolute(x, y) => {

        },
        _ => {
            if HOST.config.debug_mode {
                println!("unknown input event payload {:?}", input_event);
            }
        }
    }
}