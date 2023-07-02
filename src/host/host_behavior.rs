use std::{sync::{Arc, mpsc, Mutex}, time::{UNIX_EPOCH, Instant}, path::Path, thread::{Thread, JoinHandle}, fs::File, io::{Write, SeekFrom, Seek}};

use gl::{RGBA, UNSIGNED_BYTE};

use crate::{utils::config::Config, bind::{gl_safe::glReadPixelsSafe, gl::{K_GL_RGBA, K_GL_UNSIGNED_BYTE}, sdl2_safe}};

use super::{hosting::HOST, window::Window};

use std::time::Duration;
use std::thread::sleep;
use std::thread;

// for now we only handle a single window
pub trait HostBehavior: Send {
    fn onStart(&mut self) {

    }

    fn onWindowCreate(&mut self, win: Window, x: Option<i32>,y: Option<i32>,width: Option<u32>, height: Option<u32>) {

    }

    fn onFrameSwapBegin(&mut self) {

    }

    fn onFrameSwapEnd(&mut self){

    }

    fn getFramebufferForCapture (&self) -> Option<&Vec<u8>> {
        None
    }
}


#[derive(Debug)]
pub struct DefaultHostBehavior {
    // just access the config from the host lazy_static instead seems to be the workaround
    fb_width: Option<u32>,
    fb_height: Option<u32>,
    fb_enabled: bool,
    pub fb: Vec<u8>,
    pub tx: Option<mpsc::Sender<ThreadMessage>>,
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

impl HostBehavior for DefaultHostBehavior {
    fn onWindowCreate(&mut self, win: Window, x: Option<i32>,y: Option<i32>,width: Option<u32>, height: Option<u32>) {
        self.windows.push(win);
        if let Some(width) = width {
            if let Some(height) = height {
                self.setup_framebuffer(width, height);
            }
        }
    }

    fn onFrameSwapBegin(&mut self) {
        let start = Instant::now();
        if HOST.config.capture_mode {
            let features = HOST.features.lock().unwrap();
            if features.gl_enabled {
                // let wh = sdl2_safe::SDL_GetWindowSize_safe();
                // surely no one uses both sdl2 and something else
                if features.sdl2_enabled {

                }
                if let Some(_capture) = HOST.capture_helper.as_ref() {
                    if let (Some(fb_width), Some(fb_height)) = (self.fb_width, self.fb_height) {
                        // use opengl to capture the framebuffer if we have capture initalized
                        glReadPixelsSafe(0, 0, fb_width as i32, fb_height as i32,RGBA, UNSIGNED_BYTE, self.fb.as_mut_ptr());
                        // pov: you are a rustacean and you are reading this code (copilot wrote this and the comment)
                        // println!("a sample of captured pixels {}", self.fb[std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as usize % (self.fb.len() - 1)]);
                        // artifical lag debug
                        // sleep(Duration::from_millis(150));
                        if let Some(sender) = &self.tx {
                            sender.send(ThreadMessage::NewFrame).unwrap();
                        }
                    }
                } else {
                    if HOST.config.debug_mode {
                        println!("unknown framebuffer dimensions");
                    }
                }
            } else{
                // println!("gl not enabled");
            }
        }
        // println!("onFrameSwapBegin took {:?}", start.elapsed());
    }

    fn getFramebufferForCapture (&self) -> Option<&Vec<u8>> {
        Some(self.fb.as_ref())
    }
}

pub enum ThreadMessage {
    Stop,
    NewFrame
}

impl DefaultHostBehavior {
    pub fn new() -> Self {
        DefaultHostBehavior {
            fb_width: None,
            fb_height: None,
            fb_enabled: false,
            fb: Vec::new(),
            tx: None,
            windows: Vec::new(),
        }
    }

    pub fn spawn_writer_thread(&mut self, config: &Config) -> JoinHandle<()> {
        let base_loc = Path::new("/dev/shm");
        let file_loc = base_loc.join(format!("{}{}",config.session_id, ".raw"));

        println!("file_loc: {}", file_loc.display());

        let (tx, rx) = mpsc::channel::<ThreadMessage>(); // we use to signal to the thread to dump the frames. 

        self.tx = Some(tx);

        thread::spawn(move || {
            let mut file = File::create(file_loc).unwrap();
            loop {
                match rx.recv() {
                    Ok(ThreadMessage::Stop) => {
                        break;
                    },
                    Ok(ThreadMessage::NewFrame) => {
                        // write the frame to the file
                        if HOST.config.debug_mode {
                            // println!("new frame writing");
                        }
                        // let start = Instant::now();
                        let behavior = HOST.get_behavior();
                        file.write(behavior.getFramebufferForCapture().unwrap()).unwrap();
                        file.seek(SeekFrom::Start(0)).unwrap();
                        // println!("shm write took {:?}", start.elapsed());
                    },
                    Err(e) => {
                        println!("Error in thread: {:?}", e);
                        break;
                    }
                }
            }
        })
    }
}