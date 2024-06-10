use core::prelude::rust_2015;
use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc, Mutex}, thread::JoinHandle};

use clap::{Parser, ValueEnum, command};

use anyhow::{bail, Result};

use crossbeam_queue::SegQueue;
use gio::glib;
use gstreamer::{prelude::*, Element, ErrorMessage};
use gstreamer_app::AppSrc;
use gstreamer_video::prelude::*;
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig}, network::adapter::NetworkAddr, node::{self, NodeEvent, NodeHandler}};

use stellar_protocol::protocol::{StellarMessage};

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum OperationMode {
    Hyperwarp,
    Ingest
}

pub enum InternalMessage {
    HandshakeRecieved(stellar_protocol::protocol::StellarMessage)
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

        let inner_run = || -> Result<()> {
            println!("initalizing streaming");
            // gstreamer::init()?;

            print!("loaded streaming library");

            let pipeline = gstreamer::Pipeline::default();

            let mut testing_phase = true;

            println!("pipeline initalizing");

            let videotestsrc = match config.test_mode {
                true => gstreamer::ElementFactory::make("videotestsrc").build()?,
                false => gstreamer::ElementFactory::make("videotestsrc").build()?,
            };
            let videoconvert = gstreamer::ElementFactory::make("videoconvert").build()?;
            let sink = gstreamer::ElementFactory::make("autovideosink").build()?;

            // link
            pipeline.add_many(&[&videotestsrc, &videoconvert, &sink])?;
            gstreamer::Element::link_many(&[&videotestsrc, &videoconvert, &sink])?;

            println!("pipeline linked");

            pipeline.set_state(gstreamer::State::Playing)?;

            println!("pipeline started");

            let bus = pipeline.bus().expect("Bus not found?");

            println!("begin event ingest");

            
            let mut running = true;
            let mut prod_appsrc: Option<AppSrc> = None;

            // TODO: better interleave
            while running {
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
                if let Some(imsg) = self.streaming_command_queue.pop() {
                    match imsg {
                        InternalMessage::HandshakeRecieved(message) => {
                            let handshake = match message {
                                StellarMessage::HandshakeResponse(handshake) => handshake,
                                _ => panic!("Received invalid handshake message")
                            };
                            let res=  handshake.resolution;
                            println!("updating to {:?}", res);
                            let video_info =
                                gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Bgrx, res.0, res.1)
                           //         .fps(gst::Fraction::new(2, 1))
                                    .build()
                                    .expect("Failed to create video info on demand for source");
                            if testing_phase {
                                gstreamer::Element::unlink_many(&[&videotestsrc, &videoconvert]);
                                let appsrc = gstreamer_app::AppSrc::builder()
                                .caps(&video_info.to_caps().expect("Cap generation failed"))
                                .format(gstreamer::Format::Time)
                                .build();

                                if let Err(err) = pipeline.add_many([appsrc.upcast_ref::<Element>()]) {
                                    println!("Error adding appsrc to pipeline: {:?}", err);
                                }
                                if let Err(err) = gstreamer::Element::link_many([appsrc.upcast_ref(), &videoconvert]) {
                                    println!("Error linking appsrc to videoconvert: {:?}", err);
                                }
                                // we need to remove the test src as well
                                if let Err(err) = pipeline.remove(&videotestsrc){
                                    println!("Error removing test source: {:?}", err);
                                }

                                appsrc.set_callbacks(
                                    gstreamer_app::AppSrcCallbacks::builder().need_data(move |appsrc, _| {
                                        let mut buffer = gstreamer::Buffer::with_size(video_info.size()).unwrap();
                                        // set pts to current time
                                        {
                                            let buffer = buffer.get_mut().unwrap();
                                            buffer.set_pts(gstreamer::ClockTime::from_mseconds(0));
                                            let mut vframe = gstreamer_video::VideoFrameRef::from_buffer_ref_writable(buffer, &video_info).unwrap();
                                            // Remember some values from the frame for later usage
                                            let width = vframe.width() as usize;
                                            let height = vframe.height() as usize;

                                            // Each line of the first plane has this many bytes
                                            let stride = vframe.plane_stride()[0] as usize;
                                            let buf_mut = vframe.planes_data_mut();

                                            // Iterate over each of the height many lines of length stride
                                            for line in vframe
                                                .plane_data_mut(0)
                                                .unwrap()
                                                .chunks_exact_mut(stride)
                                                .take(height)
                                            {
                                                // Iterate over each pixel of 4 bytes in that line
                                                for pixel in line[..(4 * width)].chunks_exact_mut(4) {
                                                    pixel[0] = 124;
                                                    pixel[1] = 124;
                                                    pixel[2] = 124;
                                                    pixel[3] = 0;
                                                }
                                            }


                                        }
                                    }).build()
                                );

                                prod_appsrc = Some(appsrc);
                                testing_phase = false;
                            } else {
                                let appsrc = prod_appsrc.as_mut().unwrap();
                                appsrc.set_caps(Some(&video_info.to_caps().expect("Cap generation failed")));
                                println!("Adjusted caps for resolution {:?}", res);
                            }
                        },
                    }
                }
            }

            Ok(())
        };

        inner_run().expect("Gstreamer thread panicked");
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

        std::thread::spawn(move || {

            let inner_run = || -> Result<()> {
                println!("Enter Hyperwarp client event processing");
                listener.for_each(move |event| {
                    match event {
                        NodeEvent::Network(netevent) => {
                            match netevent {
                                message_io::network::NetEvent::Connected(endpoint, ready) => {
                                    println!("Connected to Hyperwarp socket");
                                    if ready {
                                        // say hello
                                        println!("sending hello");
                                        handler_wrapper.lock().unwrap().network().send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::Hello));
                                        println!("sending initial handshake request");
                                        handler_wrapper.lock().unwrap().network().send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HandshakeRequest));
                                        handler_wrapper.lock().unwrap().network().send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HelloName("Testing protocol".to_string())));
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
                                            println!("{:?} message", message);
                                            match message {
                                                StellarMessage::ResolutionBroadcastResponse(res_opt) => {
                                                    if let Some(res) = res_opt {
                                                        streaming_cmd_queue.push(InternalMessage::SetupResolution(res));
                                                    }
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
