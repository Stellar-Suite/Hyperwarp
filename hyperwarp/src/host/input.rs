use std::{collections::HashMap, time::Instant};

use backtrace::Backtrace;
use sdl2_sys_lite::bindings::SDL_GameController;
use stellar_protocol::protocol::{InputContext, InputEvent, InputEventPayload, InputMetadata, UsbIdentification};
use stellar_shared::constants::sdl2::*;
use stellar_shared::vendor::sdl_bindings::SDL_KeyCode;

use crate::{bind::sdl2_safe::{self, SDL_GetScancodeFromKey_safe, SDL_GetTicks_safe, SDL_PushEvent_safe}, constants::sdl2::SDL_OUR_FAKE_MOUSEID, hooks::dlsym::check_cache_integrity};

use super::{feature_flags, hosting::HOST};

// abstraction for data
pub struct Mouse {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
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

pub fn create_init_keyboard_state() -> HashMap<u32, bool> {
    let mut state = HashMap::new();
    // the below actuallu onlu worked for scancode
    /*for i in 0..1024 { // maybe make this 256
        state.insert(i, false);
    }*/
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
                self.sdl2_virt_array[sdl_scancode_u32 as usize] = state as u8; // 1 means pressed, 0 means released
            } else {
                println!("uh oh, sdl2 scancode out of bounds {}, impossible?", sdl_scancode_u32);
            }
        }

        output
    }

    pub fn get_keycode_state(&self, keycode: u32) -> bool {
        self.keycodes_state.get(&keycode).unwrap_or(&false).clone()
    }

    pub fn calc_modifiers(&self) -> u16 {
        let mut modifiers = 0u16;

        if self.get_keycode_state(SDL_KeyCode::SDLK_LSHIFT as u32) {
            modifiers |= KMOD_LSHIFT;
        }
        if self.get_keycode_state(SDL_KeyCode::SDLK_RSHIFT as u32) {
            modifiers |= KMOD_RSHIFT;
        }
        if self.get_keycode_state(SDL_KeyCode::SDLK_LCTRL as u32) {
            modifiers |= KMOD_LCTRL;
        }
        if self.get_keycode_state(SDL_KeyCode::SDLK_RCTRL as u32) {
            modifiers |= KMOD_RCTRL;
        }
        if self.get_keycode_state(SDL_KeyCode::SDLK_LALT as u32) {
            modifiers |= KMOD_LALT;
        }
        if self.get_keycode_state(SDL_KeyCode::SDLK_RALT as u32) {
            modifiers |= KMOD_RALT;
        }

        return modifiers;
    }

    pub fn get_virt_array_ptr(&self) -> *const u8 {
        self.sdl2_virt_array.as_ptr()
    }
}

pub struct Gamepad {
    pub name: String,
    pub usb_id: UsbIdentification,
    pub product_type: stellar_protocol::protocol::GameControllerType,
    pub id: String,
}

impl Gamepad {
    pub fn as_ptr(&self) -> *const Gamepad {
        self as *const Gamepad
    }

    pub fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn new(name: String, usb_id: UsbIdentification, product_type: stellar_protocol::protocol::GameControllerType) -> Gamepad {
        Gamepad {
            name: name,
            usb_id,
            product_type,
            id: Gamepad::generate_id(),
        }
    }

    pub fn from_product_type(name: String, product_type: stellar_protocol::protocol::GameControllerType) -> Gamepad {
        Gamepad::new(name, UsbIdentification::from_product_type(product_type), product_type)
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
                self.sdl2_timestamp_ticks = Some(SDL_GetTicks_safe());
                self.sdl2_timestamp_ticks_u64 = Some(SDL_GetTicks_safe() as u64); // GetTicks64 not avali in some versions of sdl2, this is sad
            }
            // TODO: sdl3
        }
    }
}

pub trait CreatableFromInputManager {
    fn from_input_manager(input_manager: &InputManager) -> Self;
}

impl CreatableFromInputManager for InputContext {
    fn from_input_manager(input_manager: &InputManager) -> InputContext {
        InputContext {
            modifiers: input_manager.keyboard.calc_modifiers(),
            buttons: input_manager.mouse.buttons,
            mouse_x: input_manager.mouse.x,
            mouse_y: input_manager.mouse.y,
        }
    }
}

pub trait CanModifyWithInputManager {
    fn modify_with_input_manager(&mut self, input_manager: &InputManager);
}

impl CanModifyWithInputManager for InputEvent {
    fn modify_with_input_manager(&mut self, input_manager: &InputManager) {
        self.add_context(InputContext::from_input_manager(input_manager));
    }
}

pub trait WithInputManagerBuilder {
    fn with_input_manager(self, input_manager: &InputManager) -> Self;
}

impl WithInputManagerBuilder for InputEvent {
    fn with_input_manager(mut self, input_manager: &InputManager) -> Self {
        self.modify_with_input_manager(input_manager);
        self
    }
}

pub struct InputManager {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub gamepads: Vec<Gamepad>,
    pub gamepads_locked: bool,
    pub event_queue: Vec<InputEvent>,
    pub event_queue_joystick_metaops: Vec<InputEvent>,
}

impl InputManager {

    pub fn add_context_to(&self, input_event: &mut InputEvent) {
        input_event.add_context(InputContext::from_input_manager(self));
    }

    pub fn new_timestamped_input_event(payload: InputEventPayload) -> InputEvent {
        let mut input_event = InputEvent::new(payload);
        input_event.metadata.timestamp();
        input_event
    }

    pub fn move_mouse_absolute(&mut self, x: i32, y: i32) {
        if let Some((width, height)) = HOST.get_behavior().get_fb_size() {
            let final_x = x.clamp(0, width as i32);
            let final_y = y.clamp(0, height as i32);
            let relative_x = final_x - self.mouse.x;
            let relative_y = final_y - self.mouse.y;
            self.mouse.x = final_x;
            self.mouse.y = final_y;
            self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::MouseMoveAbsolute(final_x, final_y, relative_x, relative_y)).with_input_manager(self));
        }
        
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
            // https://wiki.libsdl.org/SDL2/SDL_SetRelativeMouseMode
            // "SDL will report continuous relative mouse motion even if the mouse is at the edge of the window."
            x: x,
            y: y,
            x_absolute: final_x,
            y_absolute: final_y,
        }).with_input_manager(self));
    }

    pub fn drain_mouse_motion(&mut self){
        // remove all mouse motion events
        // for when you toggle relative mouse mode
        self.event_queue.retain(|event| {
            match event.payload {
                InputEventPayload::MouseMoveRelative { .. } => false,
                _ => true,
            }
        });
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
                }).with_input_manager(self));
            }
        }
    }

    pub fn new() -> InputManager {
        InputManager {
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
            gamepads: Vec::new(),
            event_queue: Vec::new(),
            gamepads_locked: false,
            event_queue_joystick_metaops: Vec::new(),
        }
    }

    pub fn add_gamepad(&mut self, gamepad: Gamepad) -> usize {
        let index = self.gamepads.len();
        self.gamepads.push(gamepad);
        // TODO: emit events
        index
    }

    pub fn get_gamepad(&self, index: usize) -> Option<&Gamepad> {
        self.gamepads.get(index)
    }

    pub fn find_gamepad(&self, id: *mut SDL_GameController) -> Option<&Gamepad> {
        let id_usize = id as usize;
        self.gamepads.iter().find(|gamepad| (*gamepad) as *const Gamepad as usize == id_usize)
    }

    pub fn count_gamepads(&self) -> usize {
        self.gamepads.len()
    }

    pub fn remove_gamepad(&mut self, index: usize) {
        // TODO: emit events
        self.gamepads.remove(index);
    }

    pub fn flush_queue(&mut self) {
        let feature_flags = HOST.features.lock().unwrap();
        for event in self.event_queue.drain(..) {
            match event.payload {
                // type confusion note: no sdl enum key values are negative yet
                InputEventPayload::KeyEvent { key, scancode, state, modifiers } => {
                    // we're going to ignore scancode for now
                    // let start_time = Instant::now();
                    if feature_flags.sdl2_enabled {
                        let event_type = if state { sdl2_sys_lite::bindings::SDL_EventType::SDL_KEYDOWN } else { sdl2_sys_lite::bindings::SDL_EventType::SDL_KEYUP };
                        let sdl_state = if state { sdl2_sys_lite::bindings::SDL_PRESSED } else { sdl2_sys_lite::bindings::SDL_RELEASED };
                        // println!("Resolving keycode enum of {}", key);
                        let keycode = get_sdl_keycode(key);
                        // println!("Resolving scancode of {:?}", keycode);
                        let scancode = SDL_GetScancodeFromKey_safe(keycode); // hack for now
                        // TODO: move hack into function
                        // println!("perform stupid transmute");
                        let scancode_for_bindings: sdl2_sys_lite::bindings::SDL_Scancode = unsafe {
                            std::mem::transmute(scancode)
                        };
                        // println!("stupid transmute works");
                        let keysym = 0;
                        let wid = HOST.get_behavior().get_largest_sdl2_window_id().unwrap_or(0);
                        // println!("wid is {}", wid);
                        let timestamp = event.metadata.sdl2_timestamp_ticks.unwrap_or(0);
                        // println!("metadata generated for event in {}ms", start_time.elapsed().as_millis());
                        // println!("timestamp is {}", timestamp);

                        // println!("constructing artifical event");

                        let mut event = sdl2_sys_lite::bindings::SDL_Event {
                            key: sdl2_sys_lite::bindings::SDL_KeyboardEvent { 
                                type_: event_type as u32,
                                timestamp: timestamp,
                                windowID: wid,
                                state: sdl_state as u8,
                                repeat: 0,
                                padding2: 0,
                                padding3: 0,
                                keysym: sdl2_sys_lite::bindings::SDL_Keysym { scancode: scancode_for_bindings, sym: key as i32, mod_: modifiers, unused: 0 } }
                        };
                        // println!("constructed event in {}ms", start_time.elapsed().as_millis());

                        // TODO: remove this
                        unsafe {
                            // TODO: handle errors
                            // https://github.com/Rust-SDL2/rust-sdl2/blob/dba66e80b14e16de309df49df0c20fdaf35b8c67/src/sdl2/event.rs#L2812
                            // also maybe don't use unsafe directly?
                            let result_ok = SDL_PushEvent_safe(&mut event);
                            // println!("pushing event in {}ms", start_time.elapsed().as_millis());
                            if result_ok != 1 {
                                let error_str = sdl2_safe::SDL_GetError_safe();
                                if HOST.config.debug_mode {
                                    println!("uh oh event push error: {}, {} {}", error_str, wid, timestamp);
                                }
                            }
                        }

                        println!("pushed event new kbd event");
                    }
                    // println!("pushed event new kbd event in {}ms", start_time.elapsed().as_millis());
                },
                InputEventPayload::MouseMoveRelative { x, y, x_absolute, y_absolute } => {
                    // println!("mouse move relative");
                    if feature_flags.sdl2_enabled {
                        if let Some(context) = event.context {
                            let wid = HOST.get_behavior().get_largest_sdl2_window_id().unwrap_or(0);
                            let timestamp = event.metadata.sdl2_timestamp_ticks.unwrap_or(0);
                            let mut event = sdl2_sys_lite::bindings::SDL_Event {
                                motion: sdl2_sys_lite::bindings::SDL_MouseMotionEvent {
                                    type_: (sdl2_sys_lite::bindings::SDL_EventType::SDL_MOUSEMOTION as u32),
                                    timestamp: timestamp,
                                    windowID: wid,
                                    which: SDL_OUR_FAKE_MOUSEID,
                                    state: context.buttons as u32, // TODO: figure out whether we want to u32 or u8 this
                                    x: x_absolute,
                                    y: y_absolute,
                                    xrel: x,
                                    yrel: y,
                                }
                            };
                            unsafe {
                                // TODO: handle errors
                                // https://github.com/Rust-SDL2/rust-sdl2/blob/dba66e80b14e16de309df49df0c20fdaf35b8c67/src/sdl2/event.rs#L2812
                                // also maybe don't use unsafe directly?
                                let result_ok = SDL_PushEvent_safe(&mut event);
                                // println!("pushing event in {}ms", start_time.elapsed().as_millis());
                                if result_ok != 1 {
                                    let error_str = sdl2_safe::SDL_GetError_safe();
                                    if HOST.config.debug_mode {
                                        println!("uh oh event push error: {}, {} {}", error_str, wid, timestamp);
                                    }
                                }
                            }
                        }else{
                            println!("no context for mouse move relative");
                        }
                    }
                },
                InputEventPayload::MouseMoveAbsolute(x, y, rel_x, rel_y) => {
                    // println!("mouse move absolute");
                    if feature_flags.sdl2_enabled {
                        if let Some(context) = event.context {
                            let wid = HOST.get_behavior().get_largest_sdl2_window_id().unwrap_or(0);
                            let timestamp = event.metadata.sdl2_timestamp_ticks.unwrap_or(0);
                            let mut event = sdl2_sys_lite::bindings::SDL_Event {
                                motion: sdl2_sys_lite::bindings::SDL_MouseMotionEvent {
                                    type_: (sdl2_sys_lite::bindings::SDL_EventType::SDL_MOUSEMOTION as u32),
                                    timestamp: timestamp,
                                    windowID: wid,
                                    which: SDL_OUR_FAKE_MOUSEID,
                                    state: context.buttons as u32, // TODO: figure out whether we want to u32 or u8 this
                                    x: x,
                                    y: y,
                                    xrel: rel_x,
                                    yrel: rel_y,
                                }
                            };
                            unsafe {
                                // TODO: handle errors
                                // https://github.com/Rust-SDL2/rust-sdl2/blob/dba66e80b14e16de309df49df0c20fdaf35b8c67/src/sdl2/event.rs#L2812
                                // also maybe don't use unsafe directly?
                                let result_ok = SDL_PushEvent_safe(&mut event);
                                // println!("pushing event in {}ms", start_time.elapsed().as_millis());
                                if result_ok != 1 {
                                    let error_str = sdl2_safe::SDL_GetError_safe();
                                    if HOST.config.debug_mode {
                                        println!("uh oh event push error: {}, {} {}", error_str, wid, timestamp);
                                    }
                                }
                            }
                        }else{
                            println!("no context for mouse move absolute");
                        }
                    }

                },
                InputEventPayload::MouseButtonsChange { change, state } => {
                    if feature_flags.sdl2_enabled {
                        if let Some(context) = event.context {
                            let event_type = if state { sdl2_sys_lite::bindings::SDL_EventType::SDL_MOUSEBUTTONDOWN } else { sdl2_sys_lite::bindings::SDL_EventType::SDL_MOUSEBUTTONUP };
                            let sdl_state = if state { sdl2_sys_lite::bindings::SDL_PRESSED } else { sdl2_sys_lite::bindings::SDL_RELEASED };
                            // println!("ResolutionBroadcastResponse: {:?}", event);
                            let wid = HOST.get_behavior().get_largest_sdl2_window_id().unwrap_or(0);
                            let timestamp = event.metadata.sdl2_timestamp_ticks.unwrap_or(0);
                            
                            let mut event = sdl2_sys_lite::bindings::SDL_Event {
                                button: sdl2_sys_lite::bindings::SDL_MouseButtonEvent {
                                    type_: event_type as u32,
                                    timestamp: timestamp,
                                    windowID: wid,
                                    which: SDL_OUR_FAKE_MOUSEID,
                                    button: change.ilog2() as u8, // this can't error right?
                                    state: sdl_state as u8, // 1 and 0 regardless
                                    clicks: 1,
                                    padding1: 0,
                                    x: context.mouse_x,
                                    y: context.mouse_y,
                                }
                            };
                            unsafe {
                                // TODO: handle errors
                                // also maybe don't use unsafe directly?
                                let result_ok = SDL_PushEvent_safe(&mut event);
                                // println!("pushing event in {}ms", start_time.elapsed().as_millis());
                                if result_ok != 1 {
                                    let error_str = sdl2_safe::SDL_GetError_safe();
                                    if HOST.config.debug_mode {
                                        println!("uh oh event push error: {}, {} {}", error_str, wid, timestamp);
                                    }
                                }
                            }
                        }else{
                            println!("no context for mouse button set");
                        }
                    }
                },
                _ => {
                    println!("unhandled event in queue: {:?}", event);
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
            InputEventPayload::MouseMoveRelative { x, y, x_absolute, y_absolute } => {
                self.move_mouse_relative(x, y);
            },
            InputEventPayload::MouseMoveAbsolute(x, y, _, _) => {
                self.move_mouse_absolute(x, y);
            },
            InputEventPayload::MouseButtonsSet { buttons } => {
                self.set_mouse_buttons(buttons);
            },
            InputEventPayload::MouseButtonsChange { change, state } => {
                let new_buttons = self.calculate_change(change, state);
                self.set_mouse_buttons(new_buttons);
            },
            _ => {
                if HOST.config.debug_mode {
                    println!("unhandled event in processing: {:?}", event);
                }
                self.event_queue.push(new_event);
            },
        }

    }

    fn calculate_change(&self, change: u8, state: bool) -> u8 {
        if state {
            self.mouse.buttons | change
        } else {
            self.mouse.buttons & !change
        }
    }

    pub fn set_mouse_buttons(&mut self, buttons: u8) {
        let changed = self.mouse.buttons != buttons;
        if changed {
            // we diff
            // we send release events for buttons that were released
            // we send press events for buttons that were pressed
            for i in 0..8 {
                let mask = 1 << i;
                if buttons & mask != self.mouse.buttons & mask && buttons & mask == 0  {
                    self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::MouseButtonsChange { change: mask, state: false }).with_input_manager(self));
                }
            }
            for i in 0..8 {
                let mask = 1 << i;
                if buttons & mask != self.mouse.buttons & mask && buttons & mask == mask {
                    self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::MouseButtonsChange { change: mask, state: true }).with_input_manager(self));
                }
            }
            // original impl
            // self.event_queue.push(Self::new_timestamped_input_event(InputEventPayload::MouseButtonsSet { buttons }));
        }
        self.mouse.buttons = buttons;
    }
}