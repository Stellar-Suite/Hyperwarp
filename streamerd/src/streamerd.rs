use core::prelude::rust_2015;
use std::{cmp, io::{Read, Seek}, path::PathBuf, sync::{atomic::AtomicBool, Arc, Mutex, RwLock}, thread::JoinHandle};

use clap::{Parser, ValueEnum, command};

use anyhow::{bail, Result};

use crossbeam_queue::SegQueue;
use gio::glib::{self, bitflags::Flags};
use gstreamer::{prelude::*, Buffer, BufferFlags, Element, ErrorMessage};
use gstreamer_app::AppSrc;
use gstreamer_video::{prelude::*, VideoInfo};
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig}, network::adapter::NetworkAddr, node::{self, NodeEvent, NodeHandler}};

use stellar_protocol::protocol::{StellarChannel, StellarMessage};

use std::time::Instant;

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum OperationMode {
    Hyperwarp,
    Ingest
}

pub enum InternalMessage {
    HandshakeReceived(stellar_protocol::protocol::Handshake),
    SetShouldUpdate(bool),
    SynchornizationReceived(stellar_protocol::protocol::Synchornization),
}



#[derive(Parser, Debug)]
#[command(version, about = "rust streaming daemon using gstreamer", long_about = None)]
pub struct StreamerConfig {
    #[arg(short, long, default_value_t = OperationMode::Hyperwarp, help = "Operation mode to use. Can be used to run without Hyperwarp injected application.")]
    mode: OperationMode,
    #[arg(short, long, help = "Socket to connect to for Hyperwarp")]
    socket: Option<PathBuf>,
    #[arg(short, long, help = "Test mode", default_value_t = true)]
    test_mode: bool,
}

impl std::fmt::Display for OperationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

pub enum StreamerSignal {
    DataChannelContent(Vec<u8>)   
}

pub struct Streamer {
    pub config: Arc<StreamerConfig>,
    pub stop: Arc<AtomicBool>,
    pub started: bool,
    pub handles: Vec<JoinHandle<()>>,
    pub messaging_handler: Option<Arc<Mutex<NodeHandler<StreamerSignal>>>>,
    pub streaming_command_queue: Arc<SegQueue<InternalMessage>>,
    pub frame: Arc<RwLock<Vec<u8>>>,
}

pub fn calc_offset(width: usize, height: usize, x: usize, y: usize) -> Option<usize> {
    if x <= width && y <= height {
        return Some(((y * width + x) * 4));
    }
    None
}

impl Streamer {
    pub fn new(config: StreamerConfig) -> Self {
        Self { 
            config: Arc::new(config),
            stop: Arc::new(AtomicBool::new(false)),
            started: false,
            handles: vec![],
            messaging_handler: None,
            streaming_command_queue: Arc::new(SegQueue::new()),
            frame: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn run(&mut self) {
        println!("Starting streamer processing thread");
        if self.config.mode == OperationMode::Hyperwarp {
            println!("Starting Hyperwarp client thread");
            let hyperwarp_thread_handle = self.start_hyperwarp_client_thread();
            self.handles.push(hyperwarp_thread_handle);
        }
        self.started = true;
        self.run_gstreamer();
    }

    pub fn run_gstreamer(&mut self) {
        let config = self.config.clone();
        let stopper = self.stop.clone();
        

        gstreamer::init().expect("bruh gstreamer didn't load");

        println!("grabbing a main loop");
        let main_loop = glib::MainLoop::new(None, false);

        println!("initalizing streaming");
        gstreamer::init().expect("library load failed");

        print!("loaded streaming library");

        let pipeline = gstreamer::Pipeline::default();

        println!("pipeline initalizing");

        let videoconvert = gstreamer::ElementFactory::make("videoconvert").build().expect("could not create video processor");
        let videoflip = gstreamer::ElementFactory::make("videoflip").build().expect("could not create optional video flipper");
        let sink = gstreamer::ElementFactory::make("autovideosink").build().expect("could not create output");
        
        let mut running = true;
        let streaming_cmd_queue_2 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_1 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_2 = self.streaming_command_queue.clone();
        let self_frame = self.frame.clone();

        let mut video_info =
        // default to 100x100
        gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, 100, 100)
    //         .fps(gst::Fraction::new(2, 1))
            .build()
            .expect("Failed to create video info on demand for source");

        let appsrc = gstreamer_app::AppSrc::builder()
        .caps(&video_info.to_caps().expect("Cap generation failed"))
        // .is_live(true)
        .block(false)
        .do_timestamp(true)
        .format(gstreamer::Format::Time)
        .build();

        // link
        pipeline.add_many([appsrc.upcast_ref::<Element>(), &videoconvert, &sink]).expect("adding els failed");
        gstreamer::Element::link_many([appsrc.upcast_ref(), &videoconvert, &sink]).expect("linking failed");

        println!("pipeline linked");

        // pipeline.set_state(gstreamer::State::Playing)?;

        // println!("pipeline started");

        let bus = pipeline.bus().expect("Bus not found?");
        let sys_clock = gstreamer::SystemClock::obtain();

        println!("begin event ingest");

        let mut should_update = false;

        let update_frame_func = |appsrc: &AppSrc, video_info: &VideoInfo| {
            
            // benchmark thing
            // let starting = Instant::now();
            
            let mut buffer = gstreamer::Buffer::with_size(video_info.size()).unwrap();
            
            // set pts to current time
            {
                let buffer = buffer.get_mut().unwrap();
                // buffer.set_pts(sys_clock.time());
                // buffer.set_flags(BufferFlags::LIVE);
                let mut vframe = gstreamer_video::VideoFrameRef::from_buffer_ref_writable(buffer, &video_info).unwrap();
                // Remember some values from the frame for later usage
                let width = vframe.width() as usize;
                let height = vframe.height() as usize;

                // Each line of the first plane has this many bytes
                let stride = vframe.plane_stride()[0] as usize;
                // let buf_mut = vframe.planes_data_mut();
                let frame_reader = self_frame.read().unwrap();

                // println!("producing frame of {}x{}", width, height);
                // Iterate over each of the height many lines of length stride
                for line in vframe
                    .plane_data_mut(0)
                    .unwrap()
                    .chunks_exact_mut(stride * height)
                    .take(1)
                {
                    // Iterate over each pixel of 4 bytes in that line
                    /*let chunk = line[..(4 * width)].chunks_exact_mut(4);
                    let mut x = 0;
                    for pixel in chunk {
                        if let Some(offset) = calc_offset(width, height, x, y) {
                            if offset + 3 < frame_reader.len() {
                                // RGBA color format
                                pixel[0] = frame_reader[offset];
                                pixel[1] = frame_reader[offset + 1];
                                pixel[2] = frame_reader[offset + 2];
                                pixel[3] = frame_reader[offset + 3];
                            } else {
                                // mismatch
                                pixel[0] = 255;
                                pixel[1] = 0;
                                pixel[2] = 0;
                                pixel[3] = 0;
                            }
                        } else {
                            // does not exist color
                            pixel[0] = 128;
                            pixel[1] = 128;
                            pixel[2] = 128;
                            pixel[3] = 0;
                        }
                        x += 1;
                    }
                    y += 1;*/

                    // new version: (still slow)
                    /*for i in 0..(width * height) {
                        let offset = i * 4;
                        if offset + 3 < frame_reader.len() {
                            // RGBA color format
                            line[offset] = frame_reader[offset];
                            line[offset + 1] = frame_reader[offset + 1];
                            line[offset + 2] = frame_reader[offset + 2];
                            line[offset + 3] = frame_reader[offset + 3];
                        } else {
                            // mismatch
                            line[offset] = 255;
                            line[offset + 1] = 0;
                            line[offset + 2] = 0;
                            line[offset + 3] = 0;
                        }
                    }*/

                    // faster version:
                    let bound = cmp::min(stride * height, frame_reader.len());
                    // ok I hope this works
                    // filling makes it 30x slower, in fact this is like I think an extra 20ms to30ms
                    // line.fill(0);
                    if bound > 0 {
                        line[0..bound].copy_from_slice(&frame_reader[0..bound]);
                    }
                }
                // println!("cped {}x{}", width, height)
            }
            match appsrc.push_buffer(buffer) {
                Ok(_) => {
                    
                },
                Err(err) => {
                    println!("Error pushing buffer: {:?}", err);
                },
            }

            // benchmark thing
            // let elapsed = starting.elapsed();
            // println!("elapsed {:?}", elapsed);
        };

        appsrc.set_callbacks(
            gstreamer_app::AppSrcCallbacks::builder().need_data(move |appsrc, _| {
                // println!("want data");
                streaming_cmd_queue_for_cb_1.push(InternalMessage::SetShouldUpdate(true));

            }).enough_data(move |appsrc| {
                // println!("enough data");
                streaming_cmd_queue_for_cb_2.push(InternalMessage::SetShouldUpdate(false));
            }).build()
        );

    

        // glib::idle_add(move );
        println!("attempting to set play pipeline");
        pipeline.set_state(gstreamer::State::Playing).expect("Could not set pipeline to playing");

        /*while running {
            for msg in bus.iter_timed(gstreamer::ClockTime::from_mseconds(1)) {
                use gstreamer::MessageView;

                println!("{:?}", msg);
        
                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        pipeline.set_state(gstreamer::State::Null)?;
                        running = false;
                        bail!("Error: {} {:?}", err.error(), err.debug());
                    }
                    _ => (),
                }
            }
            
        }*/
        println!("entering run loop");

        while running {
            let mut temp_update = false;
            // println!("iter loop");
            while let Some(msg) = bus.pop() {
                use gstreamer::MessageView;
                // qos is spammy
                if !matches!(msg.view(), MessageView::Qos(..)) {
                    println!("{:?}", msg);
                }

                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        pipeline.set_state(gstreamer::State::Null).expect("could not reset pipeline state");
                        running = false;
                        println!("Error: {} {:?}", err.error(), err.debug());
                        return;
                    }
                    _ => (),
                }
            }
            // main_loop.context().iteration(true);
            if let Some(imsg) = streaming_cmd_queue_2.pop() {
                match imsg {
                    // TODO: deduplicate code between handshake and sync, but closure does not currently work because it needs to mutate video_info
                    InternalMessage::HandshakeReceived(handshake) => {
                        let res=  handshake.resolution;
                        println!("updating to {:?}", res);
                        video_info =
                            gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                            //         .fps(gst::Fraction::new(2, 1))
                                .build()
                                .expect("Failed to create video info on demand for source");
                        println!("video info {:#?}",video_info);
                        appsrc.set_caps(Some(&video_info.to_caps().expect("Cap generation failed")));
                        appsrc.set_state(gstreamer::State::Playing).expect("Could not set appsrc to playing");
                        videoconvert.set_state(gstreamer::State::Playing).expect("Could not set videoconvert to playing");
                        sink.set_state(gstreamer::State::Playing).expect("Could not set sink to playing");
                        println!("Adjusted caps for resolution {:?}", res);
                    },
                    InternalMessage::SetShouldUpdate(new_should_update) => {
                        should_update = new_should_update;
                    },
                    InternalMessage::SynchornizationReceived(sync_details) => {
                        if let Some(res) = sync_details.resolution {
                            video_info =
                            gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                            //         .fps(gst::Fraction::new(2, 1))
                                .build()
                                .expect("Failed to create video info on demand for source");
                            println!("video info {:#?}",video_info);
                            appsrc.set_caps(Some(&video_info.to_caps().expect("Cap generation failed")));
                            appsrc.set_state(gstreamer::State::Playing).expect("Could not set appsrc to playing");
                            videoconvert.set_state(gstreamer::State::Playing).expect("Could not set videoconvert to playing");
                            sink.set_state(gstreamer::State::Playing).expect("Could not set sink to playing");
                            println!("Adjusted caps for resolution {:?}", res);
                        }
                    },
                    
                };
            }
            if temp_update || should_update {
                update_frame_func(&appsrc, &video_info);
            }
            // TOOD: make this loop not thrash cpu by waiting for events
        }

        println!("streamer thread exited cleanly.");
    }

    pub fn start_hyperwarp_client_thread(&mut self) -> JoinHandle<()> {
        let config = self.config.clone();
        let stopper = self.stop.clone();
        let socket_path = self.config.socket.clone().expect("socket path not set or not valid");
        let (handler, listener) = node::split::<StreamerSignal>();
        println!("Connecting to socket: {}", socket_path.display());
        // _ is so rustc doesn't complain about unused variable
        let temp_socket_path: PathBuf = format!("/tmp/hyperwarp/client-{}.sock", std::process::id()).into();
        let _ = handler.network().connect_with(message_io::network::TransportConnect::UnixSocketDatagram(UnixSocketConnectConfig::new(temp_socket_path)), NetworkAddr::Path(socket_path));
        let handler_wrapper = Arc::new(Mutex::new(handler));
        let handler_wrapper_2 = handler_wrapper.clone();
        self.messaging_handler = Some(handler_wrapper_2);
        println!("Starting Hyperwarp client event thread");

        let streaming_cmd_queue = self.streaming_command_queue.clone();
        let frame = self.frame.clone();
        
        std::thread::spawn(move || {

            let inner_run = || -> Result<()> {
                println!("Enter Hyperwarp client event processing");
                let mut shm_file: Option<std::fs::File> = None;
                listener.for_each(move |event| {
                    match event {
                        NodeEvent::Network(netevent) => {
                            match netevent {
                                message_io::network::NetEvent::Connected(endpoint, ready) => {
                                    println!("Connected to Hyperwarp socket");
                                    if ready {
                                        // say hello
                                        println!("sending hello");
                                        let handler = handler_wrapper.lock().unwrap();
                                        let network = handler.network();
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::Hello));
                                        println!("sending initial handshake request");
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HandshakeRequest));
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HelloName("Testing protocol".to_string())));
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::SubscribeChannel(StellarChannel::Frame)));
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::SubscribeChannel(StellarChannel::Synchornizations)));
                                    } else {
                                        println!("One client did not successfully ready. {}", endpoint.addr());
                                    }
                                },
                                message_io::network::NetEvent::Accepted(_, _) => {
                                    println!("Connect accepted from Hyperwarp socket");
                                },
                                message_io::network::NetEvent::Message(_endpoint, data) => {
                                    match stellar_protocol::deserialize_safe(&data) {
                                        Some(message) => {
                                            if !matches!(message, StellarMessage::NewFrame) {
                                                // println!("{:?} message", message);
                                            }
                                            match message {
                                                StellarMessage::HandshakeResponse(handshake) => {
                                                    // setup buffer
                                                    {
                                                        let mut writable_frame = frame.write().unwrap();
                                                        writable_frame.clear();
                                                        let resolution = handshake.resolution;
                                                        writable_frame.resize((4 * resolution.0 * resolution.1) as usize, 0);
                                                        println!("init streamer frame buffer {} bytes", (4 * resolution.0 * resolution.1));
                                                    }
                                                    {
                                                        shm_file = Some(std::fs::File::open(&handshake.shimg_path).expect("Failed to open shm file"));
                                                        println!("opened shm file for frame buffer");
                                                    }
                                                    streaming_cmd_queue.push(InternalMessage::HandshakeReceived(handshake));
                                                },
                                                StellarMessage::NewFrame => {
                                                    if let Some(shm_file) = &mut shm_file {
                                                        let mut writable_frame = frame.write().unwrap();
                                                        writable_frame.clear(); // because read_to_end "appends"
                                                        shm_file.read_to_end(&mut writable_frame).expect("Reading from shm file failed");
                                                        shm_file.seek(std::io::SeekFrom::Start(0)).expect("Seeking to start of image failed");
                                                    } else {
                                                        println!("shm file not setup yet, can't acquire frame");
                                                    }
                                                },
                                                StellarMessage::SynchronizationEvent(sync_details) => {
                                                    println!("recieving sync event on hyperwarp conn thread");
                                                    streaming_cmd_queue.push(InternalMessage::SynchornizationReceived(sync_details));
                                                },
                                                _ => {

                                                }
                                            }
                                        },
                                        None => {
                                            println!("Received invalid message from Hyperwarp socket...");
                                        }
                                    }
                                },
                                message_io::network::NetEvent::Disconnected(_) => {
                                    println!("Disconnected from Hyperwarp socket...");
                                },
                            }
                        },
                        NodeEvent::Signal(signal) => {
                            match signal {
                                StreamerSignal::DataChannelContent(_) => {
                                    
                                },
                            }
                        }
                    }
                });

                Ok(())
            };

            inner_run().expect("Hyperwarp client thread panicked");
            println!("Hyperwarp client thread exited cleanly.");
        })
    }

    pub fn stop(&self) {
        self.stop.swap(true, std::sync::atomic::Ordering::Relaxed);
    }
}
