use std::{path::PathBuf, sync::{atomic::AtomicBool, Arc, Mutex}, thread::JoinHandle};

use clap::{Parser, ValueEnum, command};

use anyhow::{bail, Result};

use gio::glib;
use gstreamer::{prelude::*, ErrorMessage};
use gstreamer_video::prelude::*;
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig}, node::{self, NodeEvent, NodeHandler}};

use stellar_protocol::protocol::StellarMessage;

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum OperationMode {
    Hyperwarp,
    Ingest
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
}

impl Streamer {
    pub fn new(config: StreamerConfig) -> Self {
        Self { 
            config: Arc::new(config),
            stop: Arc::new(AtomicBool::new(false)),
            started: false,
            handles: vec![],
            messaging_handler: None,
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

            println!("pipeline initalizing");

            let videotestsrc = gstreamer::ElementFactory::make("videotestsrc").build()?;
            let videoconvert = gstreamer::ElementFactory::make("videoconvert").build()?;
            let sink = gstreamer::ElementFactory::make("autovideosink").build()?;

            // link
            pipeline.add_many(&[&videotestsrc, &videoconvert, &sink])?;
            gstreamer::Element::link_many(&[videotestsrc, videoconvert, sink])?;

            println!("pipeline linked");

            pipeline.set_state(gstreamer::State::Playing)?;

            println!("pipeline started");

            let bus = pipeline.bus().expect("Bus not found?");

            println!("begin event ingest");

            for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
                use gstreamer::MessageView;
        
                match msg.view() {
                    MessageView::Eos(..) => break,
                    MessageView::Error(err) => {
                        pipeline.set_state(gstreamer::State::Null)?;
                        bail!("Error: {} {:?}", err.error(), err.debug());
                    }
                    _ => (),
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
        handler.network().connect_with(message_io::network::TransportConnect::UnixSocket(UnixSocketConnectConfig::new(socket_path)), create_null_socketaddr());
        let handler_wrapper = Arc::new(Mutex::new(handler));
        let handler_wrapper_2 = handler_wrapper.clone();
        self.messaging_handler = Some(handler_wrapper_2);
        println!("Starting Hyperwarp client event thread");

        std::thread::spawn(move || {

            let inner_run = || -> Result<()> {
                println!("Enter Hyperwarp client event processing");
                listener.for_each(move |event| {
                    match event {
                        NodeEvent::Network(netevent) => {
                            match netevent {
                                message_io::network::NetEvent::Connected(_, _) => {
                                    println!("Connected to Hyperwarp socket");
                                },
                                message_io::network::NetEvent::Accepted(_, _) => {
                                    println!("Connect accepted from Hyperwarp socket");
                                },
                                message_io::network::NetEvent::Message(_endpoint, data) => {
                                    match stellar_protocol::deserialize_safe(&data) {
                                        Some(message) => {
                                            println!("{:?} message", message);
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
                                StreamerSignal::DataChannelContent(_) => todo!(),
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
