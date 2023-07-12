use std::{sync::{Arc, mpsc, Mutex}, time::{UNIX_EPOCH, Instant}, path::Path, thread::{Thread, JoinHandle}, fs::File, io::{Write, SeekFrom, Seek}};

use gl::{RGBA, UNSIGNED_BYTE};

use crate::{utils::{config::Config, manual_types::sdl2, utils::convert_header_to_u8}, bind::{gl_safe::glReadPixelsSafe, gl::{K_GL_RGBA, K_GL_UNSIGNED_BYTE}, sdl2_safe}};

use super::{hosting::HOST, window::Window, message::{allocate_header_buffer, MessagePayload}};

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
    // update whenever
    fn tick(&mut self) {
        // TODO: impl in trait implementor
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
        self.tick();
        let start = Instant::now();
        if HOST.config.capture_mode {
            let features = HOST.features.lock().unwrap();
            if features.gl_enabled {
                // let wh = sdl2_safe::SDL_GetWindowSize_safe();
                // surely no one uses both sdl2 and something else
                if features.sdl2_enabled {
                    if let Some((width, height)) = self.get_largest_sdl2_window() {
                        if self.fb_width != Some(width.try_into().unwrap()) || self.fb_height != Some(height.try_into().unwrap()) {
                            println!("resize fb {}x{}", width, height);
                            self.setup_framebuffer(width.try_into().unwrap(), height.try_into().unwrap());
                        }
                    } else {
                       //  println!("sdl2 window not found");
                    }
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

    fn onFrameSwapEnd(&mut self) {
        self.tick();
        let _ = self.broadcast_message_no_type(MessagePayload::RenderedFrame {
            width: self.fb_width.unwrap(),
            height: self.fb_height.unwrap(),
        });
    }

    fn getFramebufferForCapture (&self) -> Option<&Vec<u8>> {
        Some(self.fb.as_ref())
    }

    fn tick(&mut self) {
        let mut header_buf = allocate_header_buffer();
        if let Some(conn_arcmutex) = &HOST.connection {
            let mut conn = conn_arcmutex.lock().unwrap();
            let mut transporter = conn.transporter.lock().unwrap();
            match transporter.tick() {
                Ok(_) => {},
                Err(e) => {
                    if HOST.config.debug_mode {
                        println!("Error in tick (transporter): {:?}", e);
                    }
                }
            }
            let transporters_lock = transporter.get_transports();
            let mut transports = transporters_lock.lock().unwrap();

            transports.retain(|transport| transport.is_connected());
            // iterate through each transport
            // wow iter_mut exists
            for transport in transports.iter_mut() {
                loop {
                    let polling_result = transport.recv(&mut header_buf);
                    if let Ok(would_block) = polling_result {
                        if would_block {
                            break;
                        }
                        // actual message!
                        // let's parse
                        let message_type: u32 = u32::from_le_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
                        let message_size: u32 = u32::from_le_bytes([header_buf[4], header_buf[5], header_buf[6], header_buf[7]]);
                        println!("recieved message type: {} size: {}", message_type, message_size); 
                    } else {
                        if HOST.config.debug_mode {
                            println!("Error in recv: {:?}", polling_result);
                        }
                        break;
                    }
                }
            }
        }
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

    pub fn broadcast_message_no_type(&mut self, payload: MessagePayload) -> Result<(), std::io::Error>{ // do I really need to own this 
        // TODO: explore other serialization?
        // TODO: handle errors better cause serde_json introduces it's own error 
        let serialized = serde_json::to_vec(&payload).expect("Unable to serialize broadcast message. ");
        let mut header: [u32; 2] = [0,0];
        header[1] = serialized.len() as u32;

        let header_u8 = convert_header_to_u8(header);
        
        if let Some(conn_arcmutex) = &HOST.connection {
            let mut conn = conn_arcmutex.lock().unwrap();
            let mut transporter = conn.transporter.lock().unwrap();
            match transporter.tick() {
                Ok(_) => {},
                Err(e) => {
                    if HOST.config.debug_mode {
                        println!("Error in tick (transporter): {:?}", e);
                    }
                }
            }
            let transporters_lock = transporter.get_transports();
            let mut transports = transporters_lock.lock().unwrap();

            transports.retain(|transport| transport.is_connected());
            // iterate through each transport
            // wow iter_mut exists
            for transport in transports.iter_mut() {
                transport.send(&header_u8)?;
                transport.send_vec(&serialized)?;
            }
        }

        Ok(())
    }

    pub fn get_largest_sdl2_window(&self) -> Option<(i32, i32)> {
        let mut lw = 0;
        let mut lh = 0;
        for window in &self.windows {
            let (w, h) = sdl2_safe::SDL_GetWindowSize_safe(window.id as *mut sdl2::SDL_Window);
            // println!("gws {} {}", w, h);
            if w > lw {
                lw = w;
            }
            if h > lh {
                lh = h;
            }
        }
        if lw == 0 || lh == 0 {
            None
        } else {
            Some((lw, lh))
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