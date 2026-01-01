use core::prelude::rust_2015;
use std::{any::Any, cmp, collections::HashMap, io::{Read, Seek}, path::PathBuf, str::FromStr, sync::{Arc, Mutex, MutexGuard, RwLock, atomic::AtomicBool}, thread::JoinHandle, time::{SystemTime, UNIX_EPOCH}};

use clap::{Parser, ValueEnum, command};

use anyhow::{bail, Result};

use crossbeam_channel::{Receiver, Sender};
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use gio::glib::{self, bitflags::Flags};
use gstreamer::{prelude::*, Buffer, BufferFlags, DebugGraphDetails, Element, ErrorMessage, PadProbeReturn, PadProbeType};
use gstreamer_app::AppSrc;
use gstreamer_video::{prelude::*, VideoColorimetry, VideoFlags, VideoInfo, VideoInterlaceMode};
use gstreamer_webrtc::{WebRTCDataChannel, WebRTCPeerConnectionState, WebRTCSessionDescription};
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig}, network::{adapter::NetworkAddr, Endpoint}, node::{self, NodeEvent, NodeHandler}, util::thread};

use rust_socketio::{client::Client, ClientBuilder};
use serde_json::json;
use stellar_protocol::protocol::{create_default_acl, may_mutate_pipeline, streamer_state_to_u8, EncodingPreset, GraphicsAPI, InputEvent, InputEventPayload, PipelineOptimization, PrivligeDefinition, StellarChannel, StellarDirectControlMessage, StellarFrontendMessage, StellarMessage, StreamerState};
use stellar_shared::constants::{linux::{WEB_BTN_TO_LINUX_BUTTON, decode_keyevent_code_to_evdev}, sdl2::{decode_keyevent_code_int, decode_keyevent_key_int}};

use std::time::Instant;

use crate::webrtc::{self, WebRTCPeer, WebRTCPreprocessor};

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperationMode {
    Hyperwarp,
    Ingest, // unimpl future mode where data is sent in oversocket mb?
    WaylandDesktop
}

impl OperationMode {
    pub fn is_external_capture(&self) -> bool {
        if matches!(self, OperationMode::Hyperwarp) {
            true
        } else {
            false
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OfferGenerationOriginType {
    Client,
    Element
}

pub enum InternalMessage {
    HandshakeReceived(stellar_protocol::protocol::Handshake),
    SetShouldUpdate(bool),
    SynchornizationReceived(stellar_protocol::protocol::Synchornization),
    SocketConnected,
    SocketAuthenticated,
    SocketPeerFrontendMessageWithPipeline(String, stellar_protocol::protocol::StellarFrontendMessage),
    SocketSdpAnswer(String, WebRTCSessionDescription),
    SocketSdpOffer(String, WebRTCSessionDescription),
    SocketOfferGeneration(String, OfferGenerationOriginType),
    PeerStateChange(String, WebRTCPeerConnectionState),
    SocketRtcReady(String),
    AddDataChannelForSocket(String, WebRTCDataChannel, bool), // last bool is for if it originated from the client
    ProcessDirectMessage(String, stellar_protocol::protocol::StellarDirectControlMessage),
    SendDirectMessage(String, String, stellar_protocol::protocol::StellarDirectControlMessage),
    BroadcastDirectMessage(String, stellar_protocol::protocol::StellarDirectControlMessage),
}

pub struct SystemHints {

}

pub const INTERNAL_DEBUG: bool = false;

#[derive(Parser, Debug)]
#[command(version, about = "rust streaming daemon using gstreamer", long_about = None)]
pub struct StreamerConfig {
    #[arg(short, long, default_value_t = OperationMode::Hyperwarp, help = "Operation mode to use. Can be used to run without Hyperwarp injected application (in the future).")]
    pub mode: OperationMode,
    #[arg(short, long, help = "Socket to connect to for Hyperwarp")]
    socket: Option<PathBuf>,
    #[arg(short = 't', long = "test", help = "Test mode", default_value_t = false)]
    test_mode: bool,
    #[arg(long = "realtime", help = "experimental realtime tricks", default_value_t = true)]
    experimental_realtime: bool,
    #[arg(short, long, default_value_t = GraphicsAPI::Unknown, help = "Graphics api to assume. Will autodetect if not specified.")]
    pub graphics_api: GraphicsAPI,
    #[arg(short = 'u', long = "url", default_value_t = { "http://127.0.0.1:8001".to_string() }, help = "Stargate address to connect to. Needed for signaling and other small things.")]
    stargate_addr: String,
    #[arg(long = "secret", env = "STARGATE_SECRET", help = "Session secret to authenticate and elevate when connecting to Stargate server.")]
    secret: String,
    #[arg(short = 'p', long = "pid", env = "TARGET_PROCESS_PID", help = "determine socket based off pid instead")]
    pid: Option<u32>,
    #[arg(short = 'd', long = "debug", help = "Ask process for debug info as well.")]
    debug: bool,
    #[arg(long = "stun", default_value_t = { "stun://stun.l.google.com:19302".to_string() }, help = "stun server to use")]
    stun_server: String,
    #[arg(short = 'e', long = "encoder", default_value_t = EncodingPreset::H264, help = "encoding preset to use, defaults to vp8")]
    pub encoder: EncodingPreset,
    #[arg(short = 'O', long = "optimizations", default_value_t = PipelineOptimization::None, help = "extra pipeline optimizations to apply")]
    pub optimizations: PipelineOptimization,
    #[arg(short, long = "fps", default_value = "60", help = "fps to use in streaming pipeline", env = "STREAMER_FPS")]
    fps: u32,
    #[arg(long = "mtu", default_value = "1200", help = "optional mtu to use for some network interfaces")]
    mtu: Option<u32>,
    #[arg(long = "width", default_value_t = 1920, help = "width of the stream for modes that support it", env = "STREAMER_WIDTH")]
    width: u32,
    #[arg(long = "height", default_value_t = 1080, help = "height of the stream for modes that support it", env = "STREAMER_HEIGHT")]
    height: u32,
    #[arg(long, help = "render node to use on the Wayland compositor")]
    render_node: Option<String>,
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
    DataChannelContent(Vec<u8>),   // apparently useless, will deprecated later
    ProcessInput(stellar_protocol::protocol::InputEvent),
    DebugInfoRequest,
    SocketCreated(Arc<Mutex<Client>>),
    ForwardedDataChannelMessage(String, stellar_protocol::protocol::StellarDirectControlMessage),
}

pub struct DataChannelTracker {
    pub channel_id_to_socket_id: HashMap<i32, String>,
    pub socket_id_to_channel_ids: HashMap<String, Vec<i32>>,
}

impl Default for DataChannelTracker {
    fn default() -> Self {
        Self {
            channel_id_to_socket_id: HashMap::new(),
            socket_id_to_channel_ids: HashMap::new(),
        }
    }
}

impl DataChannelTracker {
    pub fn add_data_channel(&mut self, socket_id: &str, channel: &WebRTCDataChannel) {
        let channel_id = channel.id();
        self.channel_id_to_socket_id.insert(channel_id, socket_id.to_string());
        if let Some(socket_id_list) = self.socket_id_to_channel_ids.get_mut(socket_id) {
            socket_id_list.push(channel_id);
        } else {
            self.socket_id_to_channel_ids.insert(socket_id.to_string(), vec![channel_id]);
        }
    }

    pub fn remove_data_channel(&mut self, channel_id: i32) {
        let socket_id = self.channel_id_to_socket_id.remove(&channel_id).expect("Association between channel and socket id not found");
        if let Some(socket_id_list) = self.socket_id_to_channel_ids.get_mut(&socket_id) {
            socket_id_list.retain(|id| *id != channel_id);
        }
        if self.socket_id_to_channel_ids.get(&socket_id).unwrap().len() == 0 {
            self.socket_id_to_channel_ids.remove(&socket_id);
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

pub struct Streamer {
    pub config: Arc<StreamerConfig>,
    pub stop: Arc<AtomicBool>,
    pub started: bool,
    pub handles: Vec<JoinHandle<()>>,
    pub messaging_handler: Option<Arc<Mutex<NodeHandler<StreamerSignal>>>>,
    pub streaming_command_queue: Sender<InternalMessage>,
    pub streaming_command_recv: Receiver<InternalMessage>,
    pub client_comms_command_queue: Sender<InternalMessage>,
    pub client_comms_command_recv: Receiver<InternalMessage>,
    pub frame: Arc<RwLock<Vec<u8>>>,
    pub socketio_client: Option<Arc<Mutex<Client>>>,
    pub data_channel_tracker: Arc<Mutex<DataChannelTracker>>,
    pub acls: DashMap<String, PrivligeDefinition>,
}

pub fn calc_offset(width: usize, height: usize, x: usize, y: usize) -> Option<usize> {
    if x <= width && y <= height {
        return Some(((y * width + x) * 4));
    }
    None
}

pub fn calc_offset_rgb(width: usize, height: usize, x: usize, y: usize) -> Option<usize> {
    if x <= width && y <= height {
        return Some(((y * width + x) * 3));
    }
    None
}

pub fn build_capsfilter(caps: gstreamer::Caps) -> anyhow::Result<gstreamer::Element> {
    let capsfilter = gstreamer::ElementFactory::make("capsfilter")
        .build()?;
    capsfilter.set_property("caps", &caps);
    Ok(capsfilter)
}


pub const DEFAULT_ACL: PrivligeDefinition = create_default_acl();

pub fn should_forward_data_channel_message(message: &StellarDirectControlMessage) -> bool {
    if matches!(message, StellarDirectControlMessage::AddGamepad { .. }) {
        return true;
    }
    if matches!(message, StellarDirectControlMessage::AddGamepadReply { .. }) {
        return true;
    }
    if matches!(message, StellarDirectControlMessage::UpdateGamepad { .. }) {
        return true;
    }
    if matches!(message, StellarDirectControlMessage::RemoveGamepad { .. }) {
        return true;
    }
    if matches!(message, StellarDirectControlMessage::MouseLock { .. }) || matches!(message, StellarDirectControlMessage::RequestTitle) {
        return true;
    }
    false
}

impl Streamer {
    pub fn new(config: StreamerConfig) -> Self {

        let (sender, receiver) = crossbeam_channel::unbounded::<InternalMessage>();
        let (sender_2, receiver_2) = crossbeam_channel::unbounded::<InternalMessage>();

        Self { 
            config: Arc::new(config),
            stop: Arc::new(AtomicBool::new(false)),
            started: false,
            handles: vec![],
            messaging_handler: None,
            streaming_command_queue: sender,
            streaming_command_recv: receiver,
            frame: Arc::new(RwLock::new(vec![])),
            socketio_client: None,
            data_channel_tracker: Arc::new(Mutex::new(DataChannelTracker::new())),
            acls: DashMap::new(),
            client_comms_command_queue: sender_2,
            client_comms_command_recv: receiver_2,
        }
    }

    pub fn is_externally_capturing(&self) -> bool {
        self.config.mode.is_external_capture()
    }

    /*pub fn get_acl(&self, socket_id: &str) -> &PrivligeDefinition {
        if let Some(acl_ref) = self.acls.get(socket_id) {
            acl_ref.value()
        } else {
            &DEFAULT_ACL
        }
    }*/

    pub fn run(&mut self) {
        println!("Starting streamer processing thread");
        if self.config.mode == OperationMode::Hyperwarp {
            println!("Starting Hyperwarp client thread");
            let hyperwarp_thread_handle = self.start_hyperwarp_client_thread();
            self.handles.push(hyperwarp_thread_handle);
        }
        self.started = true;
        self.start_stargate_client_thread().expect("Stargate connection failed.");
        self.start_data_channels_thread();
        self.run_gstreamer().expect("Streamer thread panicked");
    }


    pub fn get_socket(&self) -> MutexGuard<Client> {
        self.socketio_client.as_ref().expect("Socketio client not initialized").lock().unwrap()
    }

    pub fn complain_to_socket(&self, socket_id: &str, message: &str) {
        let socket =  self.get_socket();
        let error_msg = StellarFrontendMessage::Error { error: message.to_string() };
        if let Err(err) = socket.emit("send_to", json!([socket_id, error_msg])) {
            println!("Error complaining to socket: {:?}", err);
        }
    }

    pub fn broadcast_to_data_channels(&self, message: &StellarDirectControlMessage) {
        // TODO: make internal message
    }

    pub fn start_data_channels_thread(&mut self) -> JoinHandle<()> {
        let config = self.config.clone();
        let stopper = self.stop.clone();
        let socket = self.get_socket();
        let my_comms_queue = self.client_comms_command_recv.clone();
        let main_thread_queue_sender = self.streaming_command_queue.clone();
        let handler_lock_option = self.messaging_handler.clone();
        std::thread::spawn(move || {
            println!("Starting data channel message processing thread");
            while let Ok(msg) = my_comms_queue.recv() {
                match msg {
                    InternalMessage::ProcessDirectMessage(source_socket_id, message ) => {
                        let handler_option: Option<MutexGuard<'_, NodeHandler<StreamerSignal>>> = match &handler_lock_option { 
                            Some(handler_lock) => Some(handler_lock.lock().unwrap()),
                            None => None,
                        };
                        //handler_lock.lock().unwrap(); // TODO: mimimize locking this
                        match message {
                            StellarDirectControlMessage::KeyChange { key, code, composition, state, timestamp } => {
                                let input_event = InputEvent::new(InputEventPayload::KeyEvent {
                                     key: decode_keyevent_key_int(&key),
                                     scancode: decode_keyevent_code_int(&key),
                                     state,
                                     modifiers: 0 // will be calculated by Hyperwarp
                                });
                                // send signal
                                if let Some(handler) = handler_option {
                                    handler.signals().send(StreamerSignal::ProcessInput(input_event));
                                }
                            },
                            StellarDirectControlMessage::MouseMoveRelative { x, y, timestamp } => {
                                // x_absolute, y_absolute not used yet (ignored), it is calculated manually in the input manager
                                let input_event = InputEvent::new(InputEventPayload::MouseMoveRelative { x: x, y: y, x_absolute: -1, y_absolute: -1 });
                                if let Some(handler) = handler_option {
                                    handler.signals().send(StreamerSignal::ProcessInput(input_event));
                                }
                            },
                            StellarDirectControlMessage::MouseMoveAbsolute { x, y, timestamp } => {
                                // relative x,y not used, calculated manually in the input manager
                                let input_event = InputEvent::new(InputEventPayload::MouseMoveAbsolute(x,y,0,0));
                                if let Some(handler) = handler_option {
                                    handler.signals().send(StreamerSignal::ProcessInput(input_event));
                                }
                            },
                            StellarDirectControlMessage::MouseButton { change, buttons, state, timestamp } => {
                                let input_event = InputEvent::new(InputEventPayload::MouseButtonsSet { buttons: buttons });
                                if let Some(handler) = handler_option {
                                    handler.signals().send(StreamerSignal::ProcessInput(input_event));
                                }
                            },
                            _ => {
                                // check if is forwardable
                                if should_forward_data_channel_message(&message) {
                                    if let Some(handler) = handler_option {
                                        handler.signals().send(StreamerSignal::ForwardedDataChannelMessage(source_socket_id.clone(), message));
                                    } else {
                                        println!("Unhandled direct message {:?} from socket id {:?} (nowhere to forward)", message, source_socket_id);
                                        // this just means an internal streamerd impl has not handled this in addition for other modes
                                    }
                                } else {
                                    println!("Unhandled direct message {:?} from socket id {:?}", message, source_socket_id);
                                }
                            }
                        }
                    }
                    _ => {
                        // overlap in unhandled messages
                    }
                }
            }
            println!("data channel work queue died");
        })
    }

    pub fn run_gstreamer(&mut self) -> anyhow::Result<()> {
        let config = self.config.clone();
        let stopper = self.stop.clone();
        

        // println!("grabbing a main loop");
        // let main_loop = glib::MainLoop::new(None, false);

        println!("initalizing streaming");
        gstreamer::init().expect("library load failed");

        if self.config.experimental_realtime {
            WebRTCPreprocessor::install_experimental_extensions_globally().expect("could not install experimental extensions");
        }

        print!("loaded streaming library");

        let pipeline = gstreamer::Pipeline::default();
        

        println!("pipeline initalizing");

        // requires --gst-enable-gst-debug at build time for gstreamer
        pipeline.debug_to_dot_data(DebugGraphDetails::all());

        // let videoupload = gstreamer::ElementFactory::make("cudaupload").build().expect("could not create video processor");
        let capture_el = {
            if self.is_externally_capturing() {
                gstreamer::ElementFactory::make("ximagesrc").name("capture").build()
            } else if self.config.mode == OperationMode::WaylandDesktop {
                let mut builder = gstreamer::ElementFactory::make("waylanddisplaysrc").name("capture");
                if self.config.optimizations == PipelineOptimization::None {
                    // force software rendering.
                    builder = builder.property("render_node", "software");
                }
                builder.build()
            }else {
                gstreamer::ElementFactory::make("videotestsrc").name("capture").build() // fake "capture"
            }
        }.expect("could not create capture element");
        let videoconvert = gstreamer::ElementFactory::make("videoconvert").build().expect("could not create video processor");
        let videoflip = gstreamer::ElementFactory::make("videoflip").build().expect("could not create optional video flipper");
        let debug_tee = gstreamer::ElementFactory::make("tee").name("debug_tee").build().expect("could not create debugtee");
        let sink = gstreamer::ElementFactory::make("autovideosink").build().expect("could not create output");
        
        if INTERNAL_DEBUG || !self.is_externally_capturing() {
            pipeline.add(&capture_el).expect("adding debug ximagesrc to pipeline failed");
        }

        // let caps_filter_1 = build_capsfilter(gstreamer::Caps::builder("video/x-raw").field("format", "NV12").build()).expect("could not create capsfilter");

        let mut running = true;
        let streaming_cmd_queue_2 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_1 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_2 = self.streaming_command_queue.clone();
        let self_frame = self.frame.clone();

        // videoconvert.set_property_from_str("qos", "true");

        let mut downstream_peers: HashMap<String, WebRTCPeer> = HashMap::new();

        let mut video_info =
        // default to 100x100
        gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, 500, 500)
    //         .fps(gst::Fraction::new(2, 1))
           // .flags(VideoFlags::VARIABLE_FPS)
            .interlace_mode(VideoInterlaceMode::Progressive)
            .fps(gstreamer::Fraction::new(self.config.fps as i32, 1))
            .build()
            .expect("Failed to create video info on demand for source");

        let preview_sink = false;

        let appsrc = gstreamer_app::AppSrc::builder()
        .caps(&video_info.to_caps().expect("Cap generation failed"))
        // .is_live(true)
        .leaky_type(gstreamer_app::AppLeakyType::Downstream)
        .stream_type(gstreamer_app::AppStreamType::Stream)
        .block(false)
        // this is apparently important
        .is_live(true)
        .do_timestamp(true)
        .automatic_eos(false)
        .format(gstreamer::Format::Time)
        .build();

        let mut initial_link = match !INTERNAL_DEBUG && self.is_externally_capturing() {
            true => vec![appsrc.upcast_ref::<Element>()],
            false => vec![&capture_el], //vec![appsrc.upcast_ref::<Element>()];
        };

        let mut extra_els: HashMap<String, Element> = HashMap::new();

        if self.config.mode == OperationMode::WaylandDesktop {
            // negotiate dmabuf
            let caps_ = gstreamer::Caps::builder("video/x-raw")
                .field("framerate", gstreamer::Fraction::new(self.config.fps as i32, 1))
                .field("width", self.config.width)
                .field("height", self.config.height)
                .features(["memory:DMABuf"]
            ).build();

            let caps = if self.config.optimizations == PipelineOptimization::DMABuf {
                gstreamer::Caps::from_str(&format!("video/x-raw(memory:DMABuf),width={},height={},framerate={}/1", self.config.width, self.config.height, self.config.fps)).expect("could not create caps from parsed str")
            } else if self.config.optimizations == PipelineOptimization::NVIDIA {
                gstreamer::Caps::from_str(&format!("video/x-raw(memory:CUDAMemory),width={},height={},framerate={}/1", self.config.width, self.config.height, self.config.fps)).expect("could not create caps from parsed str")
            } else {
                gstreamer::Caps::from_str(&format!("video/x-raw,width={},height={},format=RGBx,framerate={}/1", self.config.width, self.config.height, self.config.fps)).expect("could not create caps from parsed str")
            };

            let capsfilter_wayland_display = build_capsfilter(caps)
            .expect("could not create capsfilter");
            pipeline.add(&capsfilter_wayland_display).expect("adding dmabuf capsfilter to pipeline failed");
            extra_els.insert("wayland_display_capfilter".to_string(), capsfilter_wayland_display);
            initial_link.push(&extra_els["wayland_display_capfilter"]);
        }

        // intiial_link.push(&videoupload);
        if self.config.mode == OperationMode::Hyperwarp  || self.config.optimizations == PipelineOptimization::None {
            initial_link.push(&videoconvert);
        }
        if self.config.mode == OperationMode::Hyperwarp {
            // unflip framebuffers
            initial_link.push(&videoflip);
            initial_link.push(&debug_tee);
            // preview sink doesn't work with dmabuf and can cause issues
            if preview_sink {
                initial_link.push(&sink);
            }
        }
        

        // link
        // pipeline.add(&videoupload).expect("adding upload element to pipeline failed");
        // pipeline.add(&caps_filter_1).expect("adding capsfilter to pipeline failed");
        if !INTERNAL_DEBUG && self.is_externally_capturing() {
            pipeline.add(appsrc.upcast_ref::<Element>()).expect("adding frames source element to pipeline failed");
        }
        if preview_sink {
            pipeline.add(&sink).expect("adding preview sink to pipeline failed");
        }
        pipeline.add_many([&videoconvert, &videoflip, &debug_tee]).expect("adding els failed");
        gstreamer::Element::link_many(&initial_link).expect("linking failed");

        /*println!("create queue before preprocessor");
        let queue = gstreamer::ElementFactory::make("queue").build().expect("could not create queue element");
        pipeline.add(&queue).expect("adding elements to pipeline failed");
        gstreamer::Element::link_many([&debug_tee, &queue]).expect("linking failed");*/

        println!("initing preprocessor");

        let mut preprocessor = WebRTCPreprocessor::new_preset(self.config.encoder, self.config.optimizations, self.config.mode);
        preprocessor.set_config(config.clone());
        preprocessor.add_congestion_control_extension();
        if self.config.experimental_realtime {
            preprocessor.add_experimental_extension().expect("could not add experimental extension");
        }
        preprocessor.set_default_settings();
        if let Some(mtu) = self.config.mtu {
            println!("setting mtu to {}", mtu);
            preprocessor.payloader.set_property("mtu", mtu);
        }
        preprocessor.attach_to_pipeline(&pipeline, {
            if self.config.mode == OperationMode::Hyperwarp {
                &debug_tee
            } else { // TODO: is last sufficient?
                // on wayland this is prob going to just be the caps filter after capture element
                initial_link.last().unwrap()
            }
        });

        println!("setting up second tee element");

        let video_tee = gstreamer::ElementFactory::make("tee").property("allow-not-linked", true).name("video_tee").build().expect("could not create video tee");
        // connect video tee to preprocessor\
        pipeline.add(&video_tee).expect("adding video tee to pipeline failed");
        gstreamer::Element::link_many([preprocessor.get_last_element(), &video_tee]).expect("linking video tee to preprocessor failed");

        println!("pipeline shared section linked");

        println!("getting bus and clock");

        let bus = pipeline.bus().expect("Bus not found?");
        let sys_clock = gstreamer::SystemClock::obtain();

        println!("begin event ingest");

        let mut should_update = false;
        let mut socket_connected = false;
        let mut socket_authed = false;
        let mut wayland_display: Option<String> = None;
        let mut graphics_api = config.graphics_api;
        let mut streamer_state = StreamerState::Handshaking;

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

        if !INTERNAL_DEBUG {
            appsrc.set_callbacks(
                gstreamer_app::AppSrcCallbacks::builder().need_data(move |appsrc, _| {
                    // println!("want data");
                    streaming_cmd_queue_for_cb_1.send(InternalMessage::SetShouldUpdate(true));

                }).enough_data(move |appsrc| {
                    // println!("enough data");
                    streaming_cmd_queue_for_cb_2.send(InternalMessage::SetShouldUpdate(false));
                }).build()
            );
        }
    
        /*let dbg_filesink = gstreamer::ElementFactory::make("filesink").property_from_str("location", "/tmp/gst-debug-file").build().expect("could not create filesink element");
        pipeline.add(&dbg_filesink).expect("adding debug filesink to pipeline failed");
        video_tee.link(&dbg_filesink).expect("linking tee to filesink failed");*/

        // this forces video flow
        let video_hacky_sink = gstreamer::ElementFactory::make("fakesink").property_from_str("async", "false").property_from_str("sync", "true").build().expect("could not create fakesink element");
        pipeline.add(&video_hacky_sink).expect("adding null fakesink to pipeline failed");
        video_tee.link(&video_hacky_sink).expect("linking tee to fakesink failed");

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

        bus.add_watch(|bus, msg| {
            glib::ControlFlow::Continue
        }).expect("Could not add bus watch");

        while running {
            let mut temp_update = false;
            // println!("iter loop");
            let mut iter_count = 0;
            while let Some(msg) = bus.pop() {
                use gstreamer::MessageView;
                // qos is spammy
                // if !matches!(msg.view(), MessageView::Qos(..)) {
                    println!("gst: {:?}", msg);
                    if msg.type_() == gstreamer::MessageType::Application {
                        let structure_opt = msg.structure();
                        if let Some(structure) = structure_opt {
                            if self.config.mode == OperationMode::WaylandDesktop && structure.name() == "wayland.src" {
                                // found wayland src
                                let display_value: String = structure.get("WAYLAND_DISPLAY").unwrap();
                                println!("wayland display value {}", display_value);
                                if socket_authed {
                                    self.get_socket().emit("ext_wayland_init", json!(display_value)).expect("Could not send wayland init to socket");
                                }
                                wayland_display = Some(display_value);
                            }
                        }
                    }
                // }

                if INTERNAL_DEBUG {
                    // pipeline.debug_to_dot_file(DebugGraphDetails::all(), PathBuf::from("pipeline.dump.dot"));
                }

                match msg.view() {
                    MessageView::Eos(..) => {
                        println!("Exiting at end of stream.");
                        return Ok(());
                    },
                    MessageView::Error(err) => {
                        println!("Error: {} {:?}", err.error(), err.debug());
                        pipeline.debug_to_dot_file_with_ts(DebugGraphDetails::all(), PathBuf::from("errordump.dot"));
                        if let Some(src) = err.src() {
                            // enum webrtc peers
                            let mut to_remove: Vec<String> = vec![];
                            for (peer_id, webrtc_peer) in downstream_peers.iter() {
                                if src.has_ancestor(&webrtc_peer.bin) {
                                    // TODO: handle disconnect by detaching
                                    println!("traced error a webrtc component, stopping webrtc peer");
                                    to_remove.push(peer_id.clone());
                                }
                            }

                            for peer_id in to_remove {
                                let downstream_peer = downstream_peers.remove(&peer_id);
                                if let Some(downstream_peer) = downstream_peer {
                                    if let Err(err) = downstream_peer.destroy(&pipeline, &video_tee) {
                                        println!("Error destroying peer on error/disconnect: {:?}", err);
                                    } else {
                                        println!("Destroyed peer on error/disconnect {}", peer_id);
                                    }
                                } else {
                                    println!("unexpected missing peer on remove? {}", peer_id);
                                }
                            }
                        };
                        pipeline.set_state(gstreamer::State::Playing).expect("could not reset pipeline state");
                        // running = false;
                        // println!("Error (repeat): {} {:?}", err.error(), err.debug());
                    }
                    _ => (),
                }
                iter_count += 1;
                if iter_count > 1000 {
                    // force processing the queue
                    break;
                }
            }
            // main_loop.context().iteration(true);

            let imsg = self.streaming_command_recv.recv()?;
            { // this block here is remenant of the old if statement
                match imsg {
                    // TODO: deduplicate code between handshake and sync, but closure does not currently work because it needs to mutate video_info
                    InternalMessage::HandshakeReceived(handshake) => {
                        println!("handshake details {:#?}", handshake);
                        if self.is_externally_capturing() {
                            let res=  handshake.resolution;
                            println!("updating to {:?}", res);
                            video_info =
                                gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                                //         .fps(gst::Fraction::new(2, 1))
                                .fps(gstreamer::Fraction::new(self.config.fps as i32, 1))
                                    .build()
                                    .expect("Failed to create video info on demand for source");
                            println!("video info {:#?}",video_info);
                            if !INTERNAL_DEBUG && self.is_externally_capturing() {
                                appsrc.set_caps(Some(&video_info.to_caps().expect("Cap generation failed")));
                            }
                            println!("Adjusted caps for resolution {:?}", res);
                            graphics_api = handshake.graphics_api;
                            println!("setting graphics api to {:?}", graphics_api);
                            let flip = stellar_protocol::protocol::should_flip_buffers_for_graphics_api(graphics_api);
                            if flip {
                                // wow gstreamer needs to make like constants for these
                                videoflip.set_property_from_str("method", "vertical-flip");
                            } else {
                                videoflip.set_property_from_str("method", "none");
                            }
                        }
                        streamer_state = StreamerState::Running;
                        {
                            // annouce that we are up and running
                            let socket = self.get_socket();
                            // this tells the ui to switch out of the loading screen
                            let state_num = streamer_state_to_u8(streamer_state);
                            println!("syncing state to ui: {:?}", state_num);
                            if let Err(err) = socket.emit("set_session_state", json!(state_num)) {
                                println!("Error setting session state on remote Stargate server: {:?}", err);
                            }else{
                                println!("Requested to set session state on remote Stargate server.");
                            }
                        }
                    },
                    InternalMessage::SetShouldUpdate(new_should_update) => {
                        should_update = new_should_update;
                    },
                    InternalMessage::SynchornizationReceived(sync_details) => {
                        println!("syncing {:?}", sync_details);
                        if let Some(res) = sync_details.resolution {
                            video_info =
                            gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                            //         .fps(gst::Fraction::new(2, 1))
                                .build()
                                .expect("Failed to create video info on demand for source");
                            println!("video info {:#?}",video_info);
                            if !INTERNAL_DEBUG && self.is_externally_capturing() {
                                appsrc.set_caps(Some(&video_info.to_caps().expect("Cap generation failed")));
                            }
                            println!("Adjusted caps for resolution {:?}", res);
                        }

                        if let Some(new_graphics_api) = sync_details.graphics_api {
                            println!("setting graphics api to {:?}", new_graphics_api);
                            graphics_api = new_graphics_api;
                            let flip = stellar_protocol::protocol::should_flip_buffers_for_graphics_api(graphics_api);
                            if flip {
                                // wow gstreamer needs to make like constants for these
                                videoflip.set_property_from_str("method", "vertical-flip");
                            } else {
                                videoflip.set_property("method", "none");
                            }
                        
                        }

                        if !INTERNAL_DEBUG && self.is_externally_capturing() {
                            appsrc.set_state(gstreamer::State::Playing);
                        }
                    },
                    InternalMessage::SocketConnected => {
                        if !socket_connected {
                            println!("Stargate socket connected for the first time.");
                            socket_connected = true;
                            // first connect logic
                            let socket =  self.get_socket();
                        } else {
                            println!("Stargate socket reconnected.");
                        }
                    },
                    InternalMessage::SocketAuthenticated => {
                        socket_authed = true;
                        let socket =  self.get_socket();
                        // this tells the ui to switch out of the loading screen

                        // start stream independently if not waiting for external capture
                        if !self.is_externally_capturing() {
                            streamer_state = StreamerState::Running;

                            if let Some(wayland_display) = &wayland_display {
                                // wayland display was set up before we finished authenticating
                                socket.emit("ext_wayland_init", json!(wayland_display)).expect("Could not send wayland init to socket after auth");
                            }
                        }
                        if let Err(err) = socket.emit("set_session_state", json!(streamer_state_to_u8(streamer_state))) {
                            println!("Error setting session state on remote Stargate server: {:?}", err);
                        }else{
                            println!("Request to set session state on remote Stargate server. New state {}", streamer_state);
                        }
                    },
                    InternalMessage::SocketPeerFrontendMessageWithPipeline(origin_socketid, frontend_message) => {
                        match frontend_message {
                            StellarFrontendMessage::ProvisionWebRTC { rtc_provision_start } => 'handle_provisioning_webrtc: {
                                println!("Provisioning webrtc for socket id {:?} client claim start: {:?}", origin_socketid, rtc_provision_start);
                                if let Err(err) = preprocessor.play() {
                                    println!("Error forceplaying preprocessor: {:?}", err);
                                }

                                if downstream_peers.contains_key(&origin_socketid) {
                                    println!("Already have a peer for socket id {:?}. Duplicate connection?", origin_socketid);
                                    break 'handle_provisioning_webrtc; // intresting new label syntax
                                }
                                // if downstream_peers.is_empty() {
                                    // pipeline.set_state(gstreamer::State::Paused).expect("pause failure");
                                // }
                                let mut downstream_peer_el_group = webrtc::WebRTCPeer::new(origin_socketid.clone());
                                downstream_peer_el_group.set_stun_server(&config.stun_server);
                                // downstream_peer_el_group.add_default_data_channels();
                                downstream_peer_el_group.link_internally().expect("Could not link webrtc peer internally");
                                

                                // if downstream_peers.is_empty() {
                                    // pipeline.set_state(gstreamer::State::Playing).expect("play failure");
                                // }
                                // downstream_peer_el_group.setup_with_pipeline(&pipeline, &video_tee);
                                // println!("{:?}", downstream_peer_el_group.queue.parent());
                                // downstream_peer_el_group.add_to_pipeline(&pipeline).expect("Could not add peer to pipeline");  
                                // let streaming_cmd_queue_for_setup = self.streaming_command_queue.clone();
                                // let queue_sink_pad = downstream_peer_el_group.queue.static_pad("sink").expect("Could not get queue sink pad");
                                // let video_tee_dynamic_pad = video_tee.request_pad_simple("src_%u").expect("Could not get a pad from tee");
                                // if let Ok(_) = downstream_peer_el_group.play() {
                                let streaming_cmd_queue_for_negotiation = self.streaming_command_queue.clone();
                                let origin_socketid_for_negotiation = origin_socketid.clone();

                                downstream_peer_el_group.webrtcbin.connect_closure("on-negotiation-needed", false, glib::closure!(move |_webrtcbin: &gstreamer::Element| {
                                    println!("element prompted negotiation");
                                    // this was causing some headaches by being spammed
                                    // hopefully it stops doing this
                                    // normally in js you can call setLocalDescription and do something
                                    // streaming_cmd_queue_for_negotiation.send(InternalMessage::SocketOfferGeneration(origin_socketid_for_negotiation.clone(), OfferGenerationOriginType::Element));
                                }));

                                let streaming_cmd_queue_for_data_channel_adding = self.streaming_command_queue.clone();
                                let origin_socketid_for_data_channel_adding = origin_socketid.clone();
                                let data_channel_tracker_for_adding = self.data_channel_tracker.clone();
                                
                                // https://github.com/servo/media/blob/45756bef67037ade0f4f0125d579fdc3f3d457c8/backends/gstreamer/webrtc.rs#L584
                                downstream_peer_el_group.webrtcbin.connect("on-data-channel", false, move |channel| {
                                    println!("on-data-channel called");
                                    let channel = channel[1]
                                        .get::<WebRTCDataChannel>()
                                        .map_err(|e| e.to_string())
                                        .expect("Invalid data channel");
                                    {
                                        data_channel_tracker_for_adding.lock().unwrap().add_data_channel(&origin_socketid_for_data_channel_adding, &channel);
                                    }
                                    streaming_cmd_queue_for_data_channel_adding.send(InternalMessage::AddDataChannelForSocket(origin_socketid_for_data_channel_adding.clone(), channel, true));
                                    None
                                });

                                downstream_peer_el_group.webrtcbin.connect_closure("on-new-transceiver", false, glib::closure!(move |_webrtcbin: &gstreamer::Element| {
                                    println!("on-new-transceiver called");
                                }));

                                let socket_arc = self.socketio_client.clone().unwrap();
                                let origin_socketid_for_ice_sending = origin_socketid.clone();

                                downstream_peer_el_group.webrtcbin.connect_closure("on-ice-candidate", false, glib::closure!(move |_webrtcbin: &gstreamer::Element, mlineindex: u32, candidate: &str| {
                                    println!("element got (produced) an ice candidate {} {}", mlineindex, candidate);
                                    let candidate_str = candidate.to_string();
                                    if candidate_str.len() == 0 {
                                        println!("{}'s webrtcbin is done sending ice candidates", origin_socketid_for_ice_sending);
                                    }
                                    let socket = socket_arc.lock().unwrap();
                                    if let Err(err) = socket.emit("send_to", json!([origin_socketid_for_ice_sending, StellarFrontendMessage::Ice { candidate: candidate_str, sdp_mline_index: mlineindex }])) {
                                        println!("Error sending ice candidate to socket id {:?}: {:?}", origin_socketid_for_ice_sending, err);
                                    }
                                }));

                                let origin_socketid_for_state_change = origin_socketid.clone();
                                let streaming_cmd_queue_for_state_change = self.streaming_command_queue.clone();

                                // https://github.com/centricular/webrtcsink/blob/main/plugins/src/webrtcsink/imp.rs
                                downstream_peer_el_group.webrtcbin.connect_notify(Some("connection-state"),  move |webrtcbin, _pspec| {
                                    let state = webrtcbin.property::<WebRTCPeerConnectionState>("connection-state");
                                    println!("{}'s connection state is {:?}", origin_socketid_for_state_change, state);
                                    streaming_cmd_queue_for_state_change.send(InternalMessage::PeerStateChange(origin_socketid_for_state_change.clone(), state));
                                });

                                // https://gitlab.freedesktop.org/gstreamer/gstreamer/-/blob/main/subprojects/gst-examples/webrtc/multiparty-sendrecv/gst-rust/src/main.rs?ref_type=heads#L464

                                let video_sink_pad = &downstream_peer_el_group.pad; // downstream_peer_el_group.queue.static_pad("sink").expect("Could not get queue sink pad"); // data enters here

                                // let video_src_pad = video_tee.request_pad_simple("src_%u").expect("Could not get a pad from video tee"); // data leaves here
                                /*let video_block = video_src_pad
                                    .add_probe(gstreamer::PadProbeType::BLOCK_DOWNSTREAM, |_pad, _info| {
                                        gstreamer::PadProbeReturn::Ok
                                    })
                                    .unwrap();*/

                                // println!("pad setup begin");
                                // println!("{}", video_src_pad.allowed_caps().unwrap().to_string());
                                // println!("{}", video_src_pad.allowed_caps().unwrap().to_string());
                                pipeline.add(&downstream_peer_el_group.bin).expect("Could not add peer bin to pipeline");
                                // video_src_pad.link(video_sink_pad).expect("Linking video src pad to video sink pad failed");
                                video_tee.link(&downstream_peer_el_group.bin).expect("Linking video tee to peer bin failed");

                                let streaming_cmd_queue_for_ready = self.streaming_command_queue.clone();

                                let origin_socketid_for_ready = origin_socketid.clone();

                                downstream_peer_el_group.bin.call_async(move |bin| {
                                    if let Err(err) = bin.sync_state_with_parent() {
                                        println!("Error syncing bin state with parent: {:?}", err);
                                    } else {
                                        println!("Bin state synced with parent, new status {:?}", bin.current_state());
                                    }

                                    // video_src_pad.remove_probe(video_block);

                                    // send init thing
                                    let _ = streaming_cmd_queue_for_ready.send(InternalMessage::SocketRtcReady(origin_socketid_for_ready));

                                });

                                // downstream_peer_el_group.bin.set_state(gstreamer::State::Playing).expect("Could not set webrtcpeer's bin state to ready");

                                downstream_peer_el_group.bin.set_state(gstreamer::State::Ready).expect("Could not set webrtcpeer's bin state to ready");
                                downstream_peer_el_group.add_default_data_channels();
                                // existing data channels are def created by us
                                
                                for channel in downstream_peer_el_group.get_data_channels() {
                                    let channel_id = channel.id();
                                    // TODO: attach handlers manually here
                                    channel.connect_on_message_data(|channel, data_opt| {
                                        if let Some(data) = data_opt {
                                            // TODO: another thread for data channel io!
                                            // not implemented yet for bytes
                                        }
                                    });

                                    let message_handler_queue = self.client_comms_command_queue.clone();
                                    let streamer_message_handler_queue = self.streaming_command_queue.clone();
                                    let socket_id_for_data_channels = origin_socketid.clone();

                                    channel.connect_on_message_string(move |channel, data_opt| {
                                        if let Some(data) = data_opt {
                                            // parse it
                                            let message = match serde_json::from_str::<stellar_protocol::protocol::StellarDirectControlMessage>(data) {
                                                Ok(message) => message,
                                                Err(err) => {
                                                    println!("Error parsing direct control message from data channel: {:?} string contents {}", err, data);
                                                    return;
                                                }
                                            };

                                            let _ = streamer_message_handler_queue.send(InternalMessage::ProcessDirectMessage(socket_id_for_data_channels.clone(), message.clone()));
                                            let _ = message_handler_queue.send(InternalMessage::ProcessDirectMessage(socket_id_for_data_channels.clone(), message));
                                        }
                                    });
                                    {
                                        self.data_channel_tracker.lock().unwrap().add_data_channel(&origin_socketid, channel);
                                    }
                                }

                                downstream_peers.insert(origin_socketid.clone(), downstream_peer_el_group);
                                println!("Added downstream peer to pipeline");

                                
                                /*if let Err(err) = socket.emit("send_to", json!([origin_socketid, StellarFrontendMessage::ProvisionWebRTCReply { provision_ok: true }])) {
                                    println!("Error sending provision reply to socket id {:?}: {:?}", origin_socketid, err);
                                }*/

                                /* } else {
                                    println!("Failed to play downstream peer");
                                    downstream_peer_el_group.remove_from_pipeline(&pipeline, &video_tee);
                                } */
                                

                            },
                            StellarFrontendMessage::Ice { candidate, sdp_mline_index } => {
                                if candidate.len() == 0 {
                                    println!("{} is done sending ice candidates", origin_socketid);
                                }
                                if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {
                                    webrtc_peer.webrtcbin.emit_by_name::<()>("add-ice-candidate", &[&sdp_mline_index, &candidate]);
                                } else {
                                    println!("warn: ice candidate accidentally dropped, race?");
                                }
                            },
                            StellarFrontendMessage::Sdp { type_, sdp } => {
                                if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                                    if type_ == "answer" {
                                        webrtc_peer.may_offer = true;
                                        println!("processing client sdp answer {}", origin_socketid);
                                        println!("{}", sdp);
                                        if let Err(err) = webrtc_peer.process_sdp_answer(&sdp) {
                                            self.complain_to_socket(&origin_socketid, &format!("Error processing sdp answer from socket id {:?}", err));
                                            println!("Error processing sdp answer from socket id {:?}: {:?}", origin_socketid, err);
                                        }
                                    }else if type_ == "offer" {
                                        // client makes offer, unused no more.
                                        if webrtc_peer.may_offer {
                                            let streaming_cmd_queue_for_reply = self.streaming_command_queue.clone();
                                            let source_id = origin_socketid.clone();
                                            println!("processing sdp offer");
                                            if let Err(err) = webrtc_peer.process_sdp_offer(&sdp, Box::new(move |reply| {
                                                println!("answering sdp offer");
                                                streaming_cmd_queue_for_reply.send(InternalMessage::SocketSdpAnswer(source_id.clone(), reply));
                                            })) {
                                                self.complain_to_socket(&origin_socketid, &format!("Error processing sdp offer from socket id {:?}", err));
                                                println!("Error processing sdp offer from socket id {:?}: {:?}", origin_socketid, err);
                                            }    
                                        } else {
                                            println!("ignoring sdp offer are already doing something");
                                        }
                                        
                                    }else{
                                        println!("Unhandled sdp type {:?} from socket id {:?}", type_, origin_socketid);
                                    }
                                }else {
                                    println!("warn: sdp accidentally dropped, race?");
                                }
                            },
                            StellarFrontendMessage::DebugInfoRequest { debug_info_request } => {
                                let mut response = format!("DEBUG: There are {} peers connected.\n", downstream_peers.len());
                                let pipeline_base = [&videoconvert, &videoflip, &debug_tee];
                                for element in pipeline_base {
                                    response.push_str(&format!("{}: {:?}\n", element.name(), element.current_state()));
                                }
                                preprocessor.for_each_element(|el| {
                                    response.push_str(&format!("{}: {:?}\n", el.name(), el.current_state()));
                                });
                                
                                // pairwise
                                for (peer_id, peer) in downstream_peers.iter() {
                                    response.push_str(&format!("Peer's queue {}: {:?}\n", peer_id, peer.queue.current_state()));
                                    response.push_str(&format!("Peer's webrtcbin {}: {:?}\n", peer_id, peer.webrtcbin.current_state()));
                                    response.push_str(&format!("Peer's may_offer {}: {:?}\n", peer_id, peer.may_offer));
                                    if let Err(err) = peer.play() {
                                        println!("Error forceplaying webrtcbin: {:?}", err);
                                    }
                                }
                                if let Err(err) = self.get_socket().emit("send_to", json!([origin_socketid, StellarFrontendMessage::DebugResponse { debug: response }])) {
                                    println!("Error sending debug info request to socket id {:?}: {:?}", origin_socketid, err);
                                }

                                // pipeline.debug_to_dot_file_with_ts(DebugGraphDetails::all(), PathBuf::from("/tmp/debug.dot"));
                                pipeline.debug_to_dot_file_with_ts(DebugGraphDetails::all(), PathBuf::from("pipeline_dump"));
                                println!("sent pipeline dump");
                                if let Err(err) = preprocessor.play() {
                                    println!("Error forceplaying preprocessor: {:?}", err);
                                }
                            },
                            StellarFrontendMessage::OfferRequest { offer_request_source } => {
                                if offer_request_source == "client" {
                                    let _ = self.streaming_command_queue.send(InternalMessage::SocketOfferGeneration(origin_socketid.clone(), OfferGenerationOriginType::Client));
                                }
                            }
                            _ => {
                                println!("Unhandled frontend message {:?}", frontend_message);
                            }
                        }
                    },
                    InternalMessage::SocketSdpAnswer(origin_socketid, desc) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                            webrtc_peer.set_remote_description(&desc);
                            println!("sending our answer sdp to socket id {:?}", origin_socketid);
                            let reply = StellarFrontendMessage::Sdp {
                                type_: "answer".to_string(),
                                sdp: desc.sdp().as_text().expect("Could not turn the session description into a string").to_string(),
                            };
                            let socket = self.get_socket();
                            if let Err(err) = socket.emit("send_to", json!([origin_socketid, reply])) {
                                println!("Error sending sdp answer to socket id {:?}: {:?}", origin_socketid, err);
                            }
                        }
                    },
                    InternalMessage::SocketSdpOffer(origin_socketid, desc) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                            webrtc_peer.may_offer = false; // just in case
                            webrtc_peer.set_local_description(&desc);
                            let reply = StellarFrontendMessage::Sdp {
                                type_: "offer".to_string(),
                                sdp: desc.sdp().as_text().expect("Could not turn the session description into a string").to_string(),
                            };
                            let socket = self.get_socket();
                            if let Err(err) = socket.emit("send_to", json!([origin_socketid, reply])) {
                                println!("Error sending sdp answer to socket id {:?}: {:?}", origin_socketid, err);
                            } else {
                                println!("Sent sdp offer to socket id {:?}", origin_socketid);
                            }
                        }
                    },
                    InternalMessage::SocketOfferGeneration(origin_socketid, origin) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                            if webrtc_peer.may_offer {
                                webrtc_peer.may_offer = false;
                                println!("generating offer for origin {:?}", origin);

                                let streaming_cmd_queue_for_reply = self.streaming_command_queue.clone();

                                let promise = gstreamer::Promise::with_change_func(move |reply| {
                                    
                                    if let Ok(Some(offer_structref)) = reply {
                                        let offer = offer_structref
                                            .value("offer")
                                            .expect("Send value")
                                            .get::<gstreamer_webrtc::WebRTCSessionDescription>()
                                            .expect("Invalid argument");
                                        println!("sending offer requested by {:?}", origin);
                                        let _ = streaming_cmd_queue_for_reply.send(InternalMessage::SocketSdpOffer(origin_socketid.clone(), offer));
                                    }else if let Err(err) = reply {
                                        println!("Error generating offer: {:?}", err);
                                    }else{
                                        println!("offer generation failed");
                                    }
                                });

                                webrtc_peer.webrtcbin.emit_by_name::<()>("create-offer", &[&None::<gstreamer::Structure>, &promise]);
                            } else {
                                println!("ignoring offer generation because we are already generating offer {}", origin_socketid);
                            }
                        } else {
                            println!("can't generate sdp offer for {:?} because it has not requested rtc", origin_socketid);
                        }
                    },
                    InternalMessage::PeerStateChange(origin_socketid, state) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {

                            let mut should_disconnect = false;

                            match state {
                                WebRTCPeerConnectionState::Connected => {
                                    webrtc_peer.may_offer = true;
                                    if let Err(err) = webrtc_peer.play() {
                                        println!("Error forceplaying webrtcbin: {:?}", err);
                                    }
                                },
                                WebRTCPeerConnectionState::Disconnected => {
                                    should_disconnect = true;
                                },
                                WebRTCPeerConnectionState::Failed => {
                                    should_disconnect = true;
                                },
                                WebRTCPeerConnectionState::Closed => {
                                    should_disconnect = true;
                                },
                                _ => {
                                    println!("unhandled webrtc peer connection state {:?}", state);
                                },
                            }

                            if should_disconnect {
                                println!("disconnecting webrtc peer {}", origin_socketid);
                                webrtc_peer.stop().expect("Error stopping webrtc peer");
                                webrtc_peer.remove_from_pipeline(&pipeline).expect("Error removing webrtc peer from pipeline");
                                if webrtc_peer.data_channels.len() > 0 {
                                    println!("{} data channels removed from peer", webrtc_peer.data_channels.len());
                                }
                                webrtc_peer.data_channels.iter().for_each(|channel| {
                                    self.data_channel_tracker.lock().unwrap().remove_data_channel(channel.id());
                                });
                                downstream_peers.remove(&origin_socketid);// drop bye bye
                                /*if let Err(err) = pipeline.set_state(gstreamer::State::Playing) {
                                    println!("Error setting pipeline state to playing: {:?}", err);
                                }*/
                            }
                        }
                    },
                    InternalMessage::SocketRtcReady(origin_socketid) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                            webrtc_peer.may_offer = true;
                            if let Err(err) = webrtc_peer.play() {
                                println!("Error forceplaying webrtcbin: {:?}", err);
                            }

                            let socket = self.get_socket();
                            if let Err(err) = socket.emit("send_to", json!([origin_socketid, StellarFrontendMessage::ProvisionWebRTCReply { provision_ok: true }])) {
                                println!("Error sending provision reply to socket id {:?}: {:?}", origin_socketid, err);
                            }
                        }
                    },
                    InternalMessage::AddDataChannelForSocket(origin_socketid, channel, originated_from_client) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&origin_socketid) {
                            channel.connect_on_message_data(|channel, data_opt| {
                                if let Some(data) = data_opt {
                                    // TODO: what do I do with binary messages?
                                }
                            });
                            let message_handler_queue = self.client_comms_command_queue.clone();
                            let streamer_message_handler_queue = self.streaming_command_queue.clone();
                            channel.connect_on_message_string(move |channel, data_opt| {
                                if let Some(data) = data_opt {
                                    // parse it
                                    let message = match serde_json::from_str::<stellar_protocol::protocol::StellarDirectControlMessage>(data) {
                                        Ok(message) => message,
                                        Err(err) => {
                                            println!("Error parsing direct control message from data channel: {:?}", err);
                                            return;
                                        }
                                    };

                                    let _ = streamer_message_handler_queue.send(InternalMessage::ProcessDirectMessage(origin_socketid.clone(), message.clone()));
                                    let _ = message_handler_queue.send(InternalMessage::ProcessDirectMessage(origin_socketid.clone(), message));
                                    
                                }
                            });
                            if originated_from_client {
                                webrtc_peer.add_data_channel(channel);
                            }
                        }
                    },
                    InternalMessage::SendDirectMessage(socket_id, channel_label, message) => {
                        if let Some(webrtc_peer) = downstream_peers.get_mut(&socket_id) {
                            let channel_opt = webrtc_peer.data_channels.iter().find(|data_channel| {
                                let label = data_channel.label();
                                if let Some(label) = label {
                                    label.as_str() == channel_label
                                }else {
                                    false
                                }
                            });
                            if let Some(channel) = channel_opt {
                                let message = serde_json::to_string(&message).expect("Could not serialize message");
                                channel.send_string(Some(&message));
                            } else {
                                println!("No data channel found for label {} on socket id {}, did it disconnect?", channel_label, socket_id);
                            }
                        }
                    },
                    InternalMessage::BroadcastDirectMessage(channel_label, message) => {
                        println!("broadcasting direct message to channel {} {:#?}", channel_label, message);
                        for (socket_id, rtc_peer) in downstream_peers.iter() {
                            rtc_peer.get_data_channels().iter().for_each(|channel| {
                                let label = channel.label();
                                if let Some(label) = label {
                                    if label.as_str() == channel_label {
                                        let serialized_message = serde_json::to_string(&message).expect("Could not serialize message");
                                        channel.send_string(Some(&serialized_message));
                                    }
                                }
                            })
                        }
                    },
                    InternalMessage::ProcessDirectMessage(source_socket_id, message) => {
                        // this recieves a copy of every message that the other thread does, it exists to allow lower level handling of events that need to be taken care of within the gstreamer pipeline loop
                        if self.config.mode == OperationMode::WaylandDesktop {
                            let mut ignored = false;
                            match &message {
                                StellarDirectControlMessage::KeyChange { key, code, composition, state, timestamp } => {
                                    let key: u32 = decode_keyevent_code_to_evdev(&code);
                                    let pressed = *state;
                                    let event_data_structure = gstreamer::Structure::builder("KeyboardKey")
                                        .field("key", key)
                                        .field("pressed", pressed)
                                        .build();


                                    capture_el.send_event(gstreamer::event::CustomUpstream::new(event_data_structure));
                                    // Date.now() - lastKeyEventTime
                                    // let delta = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() - (*timestamp) as u128;
                                    // println!("delta {} ms pressed: {}", delta, pressed);
                                },
                                StellarDirectControlMessage::MouseMoveRelative { x, y, timestamp } => {
                                    let event_data_structure = gstreamer::Structure::builder("MouseMoveRelative")
                                        .field("pointer_x", *x as f64)
                                        .field("pointer_y", *y as f64)
                                        .build();

                                    capture_el.send_event(gstreamer::event::CustomUpstream::new(event_data_structure));
                                },
                                StellarDirectControlMessage::MouseMoveAbsolute { x, y, timestamp } => {
                                    let event_data_structure = gstreamer::Structure::builder("MouseMoveAbsolute")
                                        .field("pointer_x", *x as f64)
                                        .field("pointer_y", *y as f64)
                                        .build();

                                    capture_el.send_event(gstreamer::event::CustomUpstream::new(event_data_structure));
                                },
                                StellarDirectControlMessage::MouseButton { change, buttons, state, timestamp } => {
                                    let button_index_web = change.ilog2() as u32;

                                    if let Some(button_index_linux) = WEB_BTN_TO_LINUX_BUTTON.get(button_index_web as usize) {
                                        let event_data_structure = gstreamer::Structure::builder("MouseButton")
                                            .field("button", *button_index_linux)
                                            .field("pressed", state)
                                            .build();

                                        capture_el.send_event(gstreamer::event::CustomUpstream::new(event_data_structure));
                                    } else {
                                        println!("Could not map web button index {} to linux button index (illegal mouse button?)", button_index_web);
                                    }
                                },
                                StellarDirectControlMessage::MouseScroll { delta_x, delta_y, timestamp } => {
                                    let event_data_structure = gstreamer::Structure::builder("MouseAxis")
                                        .field("x", *delta_x as f64)
                                        .field("y", *delta_y as f64)
                                        .field("timestamp", timestamp)
                                        .build();

                                    capture_el.send_event(gstreamer::event::CustomUpstream::new(event_data_structure));
                                },
                                StellarDirectControlMessage::MouseLock { state } => {
                                    // i don't think the client should be sending this?
                                },
                                _ => {
                                    // ignore
                                    ignored = true;
                                }
                               
                                /*StellarDirectControlMessage::AddGamepad { local_id, product_type, axes, buttons, hats } => todo!(),
                                StellarDirectControlMessage::AddGamepadReply { local_id, remote_id, success, message } => todo!(),
                                StellarDirectControlMessage::UpdateGamepad { remote_id, axes, buttons, hats } => todo!(),
                                StellarDirectControlMessage::RemoveGamepad { remote_id } => todo!(),
                                StellarDirectControlMessage::RemoveGamepadReply { remote_id, success, message } => todo!(),*/
                            }
                            if !ignored {
                                println!("Handled direct message {:?} from socket id {:?}", message, source_socket_id);
                            }
                        }
                    },
                    _ => { // if thi warns about unreachable code, it's very good because we implemented everything
                        // TODO: print more descriptive
                        println!("BAD: Unimplemented message {:#?}", imsg.type_id());
                    }
                };
            }
            if temp_update || should_update {
                update_frame_func(&appsrc, &video_info);
            }
            // TOOD: make this loop not thrash cpu by waiting for events, does it still/
        }

        println!("streamer thread exited cleanly.");
        Ok(())
    }

    pub fn start_stargate_client_thread(&mut self) -> anyhow::Result<()>{
        // TODO: do I need to explictly make thread since rust-socketio does this for me?
        let main_thread_cmd_queue_1 = self.streaming_command_queue.clone();
        let main_thread_cmd_queue_2 = self.streaming_command_queue.clone();
        let main_thread_cmd_queue_3 = self.streaming_command_queue.clone();

        let mut socket_builder = ClientBuilder::new(self.config.stargate_addr.clone());

        let config = self.config.clone();
        
        let local_message_handler_option = self.messaging_handler.clone();

        socket_builder = socket_builder.on("hello", move |payload, client| {
            main_thread_cmd_queue_1.send(InternalMessage::SocketConnected);
            // now we need to elevate privs
            if let Err(err) = client.emit("upgrade_privs", json!(config.secret.clone())) {
                println!("Initial privlige elevation failed: {:?}, may retry on reconnect", err);
            }
        }).on("upgraded", move |payload, client| {
            println!("Privlige upgrade accepted.");
            main_thread_cmd_queue_2.send(InternalMessage::SocketAuthenticated);
        }).on("peer_message", move |payload, client| {
            println!("peer_message {:#?}", payload);
            match payload {
                rust_socketio::Payload::Binary(bin) => {
                    // serde in js would never
                },
                rust_socketio::Payload::Text(values) => {
                    // serde json deserialize time
                    if values.len() >= 2 {
                        if let serde_json::Value::String(src_socketid) = values.get(0).unwrap() {
                            // rip 0 copy because of to_owend
                            match serde_json::from_value::<StellarFrontendMessage>(values.get(1).unwrap().to_owned()) {
                                Ok(frontend_message) => {
                                    match frontend_message {
                                        StellarFrontendMessage::Test { time } => {

                                        },
                                        StellarFrontendMessage::Ping { ping_payload } => {
                                            // TODO: 
                                        },
                                        StellarFrontendMessage::HyperwarpDebugInfoRequest { hyperwarp_debug_info_request } => {
                                            // we forward this to hyperwarp, if we have an active connection
                                            if let Some(message_handler_inner) = &local_message_handler_option {
                                                let handler = message_handler_inner.lock().unwrap();
                                                handler.signals().send(StreamerSignal::DebugInfoRequest);
                                            }
                                        },
                                        StellarFrontendMessage::EndSessionRequest { end_session_request } => {
                                            // not impl
                                        },
                                        other => {
                                            if may_mutate_pipeline(&other) {
                                                main_thread_cmd_queue_3.send(InternalMessage::SocketPeerFrontendMessageWithPipeline(src_socketid.clone(), other));
                                            } else {
                                                println!("Unhandled frontend message {:?}", other);
                                            }
                                        }
                                    }
                                },
                                Err(err) => println!("malformed frontend message {:?}", err),
                            }
                        } else {
                            println!("very malformed frontend message, missing source socket id string");
                        }
                    }
                    
                },
                rust_socketio::Payload::String(_) => {
                    // deprecated
                },
            }
        });

        println!("Connecting to Stargate server");

        let socket = socket_builder.connect()?;

        println!("Connected to Stargate server");

        let arc = Arc::new(Mutex::new(socket));
        
        let socket_arc = arc.clone();

        self.socketio_client = Some(arc);
        // send to hyperwarp client thread via signal
        let messaging_handler_option = self.messaging_handler.clone();
        if let Some(messaging_handler) = &messaging_handler_option {
            let handler = messaging_handler.lock().unwrap();
            let _ = handler.signals().send(StreamerSignal::SocketCreated(socket_arc));
        }

        Ok(())
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
        self.messaging_handler = Some(handler_wrapper_2); // this part runs before the thread is started so it always exists
        println!("Starting Hyperwarp client event thread");

        let streaming_cmd_queue = self.streaming_command_queue.clone();
        let frame = self.frame.clone();
        let is_externally_capturing = self.is_externally_capturing();

        std::thread::spawn(move || {

            let inner_run = || -> Result<()> {
                println!("Enter Hyperwarp client event processing");
                let mut shm_file: Option<std::fs::File> = None;
                let mut current_endpoint: Option<Endpoint> = None;
                let mut socket: Option<Arc<Mutex<Client>>> = None;
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
                                    current_endpoint = Some(endpoint);
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
                                                        // this doesn't actually need to happen because it's cleared and appended anyways
                                                        writable_frame.resize((4 * resolution.0 * resolution.1) as usize, 0);
                                                        println!("init streamer frame buffer {} bytes", (4 * resolution.0 * resolution.1));
                                                    }
                                                    {
                                                        shm_file = Some(std::fs::File::open(&handshake.shimg_path).expect("Failed to open shm file"));
                                                        println!("opened shm file for frame buffer");
                                                    }
                                                    streaming_cmd_queue.send(InternalMessage::HandshakeReceived(handshake));
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
                                                    // this doesn't happen enough I think to be spammy?
                                                    println!("recieving sync event on hyperwarp conn thread");
                                                    streaming_cmd_queue.send(InternalMessage::SynchornizationReceived(sync_details));
                                                },
                                                StellarMessage::DebugInfoResponseV2(debug_info, source) => {
                                                    println!("Debug info response from hyperwarp ({}): {:?}", source, debug_info);
                                                    if let Some(socket) = &socket {
                                                        let _ = socket.lock().unwrap().emit("send_to", json!([source, StellarFrontendMessage::HyperwarpDebugResponse { 
                                                            hyperwarp_debug: debug_info.message,
                                                            source: source.clone()
                                                         }]));
                                                    }
                                                },
                                                StellarMessage::ReplyDataChannelMessage(source, channel, direct_message) => {
                                                    println!("Reply data channel message from hyperwarp ({}): {:?}", source, direct_message);
                                                    streaming_cmd_queue.send(InternalMessage::SendDirectMessage(source, channel, direct_message));
                                                },
                                                StellarMessage::BroadcastDataChannelMessage(channel, direct_message) => {
                                                    println!("Broadcast data channel message from hyperwarp: {:?}", direct_message);
                                                    streaming_cmd_queue.send(InternalMessage::BroadcastDirectMessage(channel, direct_message));
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
                                    current_endpoint = None;
                                },
                            }
                        },
                        // user socket, sent something
                        NodeEvent::Signal(signal) => {
                            match signal {
                                StreamerSignal::DataChannelContent(_) => {
                                    // unused apparently
                                },
                                StreamerSignal::ProcessInput(input_event) => {
                                    if is_externally_capturing {
                                        let handler = handler_wrapper.lock().unwrap();
                                        let network = handler.network();
                                        let message = stellar_protocol::protocol::StellarMessage::UserInputEvent(input_event);
                                        // println!("sent input");
                                        if let Some(endpoint) = &current_endpoint {
                                            network.send(endpoint.clone(), &stellar_protocol::serialize(&message));
                                        }
                                    }
                                },
                                StreamerSignal::DebugInfoRequest => {
                                    let handler = handler_wrapper.lock().unwrap();
                                    let network = handler.network();
                                    let message = stellar_protocol::protocol::StellarMessage::DebugInfoRequestV2;
                                    println!("sent debug info request to hyperwarp");
                                    if let Some(endpoint) = &current_endpoint {
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&message));
                                    }
                                },
                                StreamerSignal::SocketCreated(sent_socket) => {
                                    socket = Some(sent_socket);
                                },
                                StreamerSignal::ForwardedDataChannelMessage(source_socket_id, message) => {
                                    let handler = handler_wrapper.lock().unwrap();
                                    let network = handler.network();
                                    let message = stellar_protocol::protocol::StellarMessage::ForwardedDataChannelMessage(source_socket_id, message);
                                    // println!("sent forwarded data channel message to hyperwarp");
                                    if let Some(endpoint) = &current_endpoint {
                                        network.send(endpoint.clone(), &stellar_protocol::serialize(&message));
                                    }
                                }
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
