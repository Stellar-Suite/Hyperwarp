use std::{sync::Arc, time::UNIX_EPOCH};

use gl::{RGBA, UNSIGNED_BYTE};

use crate::{utils::config::Config, bind::{gl_safe::glReadPixelsSafe, gl::{K_GL_RGBA, K_GL_UNSIGNED_BYTE}}};

use super::hosting::HOST;

use std::time::Duration;
use std::thread::sleep;

// for now we only handle a single window
pub trait HostBehavior: Send {
    fn onStart(&mut self) {

    }

    fn onWindowCreate(&mut self, x: Option<i32>,y: Option<i32>,width: Option<u32>, height: Option<u32>) {

    }

    fn onFrameSwapBegin(&mut self) {

    }

    fn onFrameSwapEnd(&mut self){

    }
}


#[derive(Debug)]
pub struct DefaultHostBehavior {
    // just access the config from the host lazy_static instead seems to be the workaround
    fb_width: Option<u32>,
    fb_height: Option<u32>,
    fb_enabled: bool,
    fb: Vec<u8>,
}

impl DefaultHostBehavior {
    fn setup_framebuffer(&mut self, width: u32, height: u32) {
        println!("Create fb: {}x{}", width, height);
        self.fb_width = Some(width);
        self.fb_height = Some(height);
        self.fb = vec![0; (width * height * 4) as usize];
        self.fb_enabled = true;
    }
}

impl HostBehavior for DefaultHostBehavior {
    fn onWindowCreate(&mut self, x: Option<i32>,y: Option<i32>,width: Option<u32>, height: Option<u32>) {
        if let Some(width) = width {
            if let Some(height) = height {
                self.setup_framebuffer(width, height);
            }
        }
    }

    fn onFrameSwapBegin(&mut self) {
        if HOST.config.capture_mode {
            let features = HOST.features.lock().unwrap();
            if features.gl_enabled {
                // use opengl to capture the framebuffer
                glReadPixelsSafe(0, 0, self.fb_width.unwrap() as i32, self.fb_height.unwrap() as i32,RGBA, UNSIGNED_BYTE, self.fb.as_mut_ptr());
                // pov: you are a rustacean and you are reading this code (copilot wrote this and the comment)
                println!("a sample of captured pixels {}", self.fb[std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as usize % (self.fb.len() - 1)]);
                sleep(Duration::from_millis(50));
            }
        }
    }
}


impl DefaultHostBehavior {
    pub fn new() -> Self {
        DefaultHostBehavior {
            fb_width: None,
            fb_height: None,
            fb_enabled: false,
            fb: Vec::new(),
        }
    }
}