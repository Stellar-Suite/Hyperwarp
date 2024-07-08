use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread::{JoinHandle, Thread},
    time::{Instant, UNIX_EPOCH},
};

use gl::{RGBA, UNSIGNED_BYTE};

use crate::{
    bind::{
        gl::{K_GL_RGBA, K_GL_UNSIGNED_BYTE},
        gl_safe::glReadPixelsSafe,
        sdl2_safe,
    },
    utils::{config::Config, manual_types::sdl2, utils::convert_header_to_u8},
};

use super::{hosting::HOST, window::Window};

use std::thread;
use std::thread::sleep;
use std::time::Duration;

// for now we only handle a single window
// TODO: make casing consistent
/*pub trait HostBehavior: Send {
    fn onStart(&mut self) {}

    fn onWindowCreate(
        &mut self,
        win: Window,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) {
    }

    fn onFrameSwapBegin(&mut self) {}

    fn onFrameSwapEnd(&mut self) {}

    fn getFramebufferForCapture(&self) -> Option<&Vec<u8>> {
        None
    }
    // update whenever
    fn tick(&mut self) {
        // TODO: impl in trait implementor
    }

    fn get_fb_size(&self) -> Option<(u32, u32)> {
        None
    }

    fn get_shimg_path(&self, config: &Config) -> PathBuf {
        let base_loc = Path::new("/dev/shm");
        let file_loc = base_loc.join(format!("{}{}", config.session_id, ".raw"));
        file_loc
    }
}*/

#[derive(Debug)]
pub struct DefaultHostBehavior {
    // just access the config from the host lazy_static instead seems to be the workaround
    fb_width: Option<u32>,
    fb_height: Option<u32>,
    fb_enabled: bool,
    pub fb: Vec<u8>,
    pub tx: Option<mpsc::Sender<FrameWriterThreadMessage>>,
    pub windows: Vec<Window>,
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

impl DefaultHostBehavior {

    pub fn get_shimg_path(&self, config: &Config) -> PathBuf {
        let base_loc = Path::new("/dev/shm");
        let file_loc = base_loc.join(format!("{}{}", config.session_id, ".raw"));
        file_loc
    }

    pub fn onWindowCreate(
        &mut self,
        win: Window,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) {
        self.windows.push(win);
        if let Some(width) = width {
            if let Some(height) = height {
                self.setup_framebuffer(width, height);
            }
        }
    }

    pub fn onFrameSwapBegin(&mut self) {
        // HOST.tick();
        let start = Instant::now();
        if HOST.config.capture_mode {
            let features = HOST.features.lock().unwrap();
            if features.gl_enabled {
                // let wh = sdl2_safe::SDL_GetWindowSize_safe();
                // surely no one uses both sdl2 and something else
                if features.sdl2_enabled {
                    if let Some((width, height)) = self.get_largest_sdl2_window() {
                        if self.fb_width != Some(width.try_into().unwrap())
                            || self.fb_height != Some(height.try_into().unwrap())
                        {
                            if HOST.config.debug_mode {
                                println!("resize fb {}x{}", width, height);
                            }
                            self.setup_framebuffer(
                                width.try_into().unwrap(),
                                height.try_into().unwrap(),
                            );
                        }
                    } else {
                        //  println!("sdl2 window not found");
                    }
                }
                if let Some(_capture) = HOST.capture_helper.as_ref() {
                    if let (Some(fb_width), Some(fb_height)) = (self.fb_width, self.fb_height) {
                        // use opengl to capture the framebuffer if we have capture initalized
                        glReadPixelsSafe(
                            0,
                            0,
                            fb_width as i32,
                            fb_height as i32,
                            RGBA,
                            UNSIGNED_BYTE,
                            self.fb.as_mut_ptr(),
                        );
                        // pov: you are a rustacean and you are reading this code (copilot wrote this and the comment)
                        // println!("a sample of captured pixels {}", self.fb[std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as usize % (self.fb.len() - 1)]);
                        // artifical lag debug
                        // sleep(Duration::from_millis(150));
                        if let Some(sender) = &self.tx {
                            sender.send(FrameWriterThreadMessage::NewFrame).unwrap();
                        }
                    }
                } else {
                    if HOST.config.debug_mode {
                        println!("unknown framebuffer dimensions");
                    }
                }
            } else {
                // println!("gl not enabled");
            }
        }
        if HOST.config.tracing_mode {
            println!("onFrameSwapBegin took {:?}", start.elapsed());
        }
    }

    pub fn onFrameSwapEnd(&mut self) {

    }

    pub fn getFramebufferForCapture(&self) -> Option<&Vec<u8>> {
        Some(self.fb.as_ref())
    }

    pub fn tick(&mut self) {}

    pub fn get_fb_size(&self) -> Option<(u32, u32)> {
        if let Some(width) = self.fb_width {
            if let Some(height) = self.fb_height {
                return Some((width, height));
            }
        }
        None
    }
}

impl DefaultHostBehavior {}

pub enum FrameWriterThreadMessage {
    Stop,
    NewFrame,
}

impl DefaultHostBehavior {
    pub fn new() -> Self {
        DefaultHostBehavior {
            fb_width: None,
            fb_height: None,
            fb_enabled: false,
            fb: Vec::new(),
            tx: None,
            windows: Vec::new()
        }
    }

    pub fn get_largest_sdl2_window(&self) -> Option<(i32, i32)> {
        let mut la = 0;
        let mut lw = 0;
        let mut lh = 0;
        for window in &self.windows {
            let (w, h) = sdl2_safe::SDL_GetWindowSize_safe(window.id as *mut sdl2::SDL_Window);
            // println!("gws {} {}", w, h);
            
            if w * h > la {
                la = w * h;
                lw = w;
                lh = h;
            }
        }
        if lw == 0 || lh == 0 {
            None
        } else {
            Some((lw, lh))
        }
    }

    pub fn get_largest_sdl2_window_id(&self) -> Option<u32> {
        let mut la = 0;
        let mut lw = 0;
        let mut lh = 0;
        if self.windows.len() == 0 {
            return None;
        }
        let mut chosen: u32 = 0;
        for window in &self.windows {
            let (w, h) = sdl2_safe::SDL_GetWindowSize_safe(window.id as *mut sdl2::SDL_Window);
            // println!("gws {} {}", w, h);
            if w * h > la {
                la = w * h;
                lw = w;
                lh = h;
                chosen = sdl2_safe::SDL_GetWindowID_safe(window.id as *mut sdl2::SDL_Window);
            }
        }
        Some(chosen)
    }

    pub fn spawn_writer_thread(&mut self, config: &Config) -> JoinHandle<()> {
        let base_loc = Path::new("/dev/shm");
        let file_loc = base_loc.join(format!("{}{}", config.session_id, ".raw"));

        println!("file_loc: {}", file_loc.display());

        let (tx, rx) = mpsc::channel::<FrameWriterThreadMessage>(); // we use to signal to the thread to dump the frames.

        self.tx = Some(tx);

        thread::spawn(move || {
            let mut file = File::create(file_loc).unwrap();
            loop {
                match rx.recv() {
                    Ok(FrameWriterThreadMessage::Stop) => {
                        break;
                    }
                    Ok(FrameWriterThreadMessage::NewFrame) => {
                        // write the frame to the file
                        if HOST.config.debug_mode {
                            // println!("new frame writing");
                        }
                        // let start = Instant::now();
                        let behavior = HOST.get_behavior();
                        file.write(behavior.getFramebufferForCapture().unwrap())
                            .unwrap();
                        file.seek(SeekFrom::Start(0)).unwrap();
                        // println!("shm write took {:?}", start.elapsed());
                    }
                    Err(e) => {
                        println!("Error in thread: {:?}", e);
                        break;
                    }
                }
            }
        })
    }
}
