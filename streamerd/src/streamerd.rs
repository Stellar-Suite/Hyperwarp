use core::prelude::rust_2015;
use std::{any::Any, cmp, collections::HashMap, io::{Read, Seek}, path::PathBuf, sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard, RwLock}, thread::JoinHandle};

use clap::{Parser, ValueEnum, command};

use anyhow::{bail, Result};

use crossbeam_channel::{Receiver, Sender};
use crossbeam_queue::SegQueue;
use gio::glib::{self, bitflags::Flags};
use gstreamer::{prelude::*, Buffer, BufferFlags, DebugGraphDetails, Element, ErrorMessage};
use gstreamer_app::AppSrc;
use gstreamer_video::{prelude::*, VideoColorimetry, VideoFlags, VideoInfo};
use gstreamer_webrtc::WebRTCSessionDescription;
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig}, network::adapter::NetworkAddr, node::{self, NodeEvent, NodeHandler}};

use rust_socketio::{client::Client, ClientBuilder};
use serde_json::json;
use stellar_protocol::protocol::{may_mutate_pipeline, streamer_state_to_u8, EncodingPreset, GraphicsAPI, PipelineOptimization, StellarChannel, StellarFrontendMessage, StellarMessage, StreamerState};

use std::time::Instant;

use crate::webrtc::{self, WebRTCPeer, WebRTCPreprocessor};

// https://docs.rs/clap/latest/clap/_derive/_cookbook/git_derive/index.html

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperationMode {
    Hyperwarp,
    Ingest
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
    SocketOfferGeneration(String),
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
    #[arg(short, long = "fps", default_value = "60", help = "fps to use in streaming pipeline")]
    fps: Option<u32>,
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
    pub streaming_command_queue: Sender<InternalMessage>,
    pub streaming_command_recv: Receiver<InternalMessage>,
    pub frame: Arc<RwLock<Vec<u8>>>,
    pub socketio_client: Option<Arc<Mutex<Client>>>,
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
        .name("capsfilter")
        .build()?;
    capsfilter.set_property("caps", &caps);
    Ok(capsfilter)
}

impl Streamer {
    pub fn new(config: StreamerConfig) -> Self {

        let (sender, receiver) = crossbeam_channel::unbounded::<InternalMessage>();

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
        self.start_stargate_client_thread().expect("Stargate connection failed.");
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

    pub fn run_gstreamer(&mut self) -> anyhow::Result<()> {
        let config = self.config.clone();
        let stopper = self.stop.clone();
        

        // println!("grabbing a main loop");
        // let main_loop = glib::MainLoop::new(None, false);

        println!("initalizing streaming");
        gstreamer::init().expect("library load failed");

        print!("loaded streaming library");

        let pipeline = gstreamer::Pipeline::default();

        println!("pipeline initalizing");

        // requires --gst-enable-gst-debug at build time for gstreamer
        pipeline.debug_to_dot_data(DebugGraphDetails::all());

        // let videoupload = gstreamer::ElementFactory::make("cudaupload").build().expect("could not create video processor");
        let ximagesrc = gstreamer::ElementFactory::make("ximagesrc").name("ximagesrc").build().expect("could not create ximagesrc element");
        let videoconvert = gstreamer::ElementFactory::make("videoconvert").build().expect("could not create video processor");
        let videoflip = gstreamer::ElementFactory::make("videoflip").build().expect("could not create optional video flipper");
        let debug_tee = gstreamer::ElementFactory::make("tee").name("debug_tee").build().expect("could not create debugtee");
        let sink = gstreamer::ElementFactory::make("autovideosink").build().expect("could not create output");
        
        pipeline.add(&ximagesrc).expect("adding debug ximagesrc to pipeline failed");

        // let caps_filter_1 = build_capsfilter(gstreamer::Caps::builder("video/x-raw").field("format", "NV12").build()).expect("could not create capsfilter");

        let mut running = true;
        let streaming_cmd_queue_2 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_1 = self.streaming_command_queue.clone();
        let streaming_cmd_queue_for_cb_2 = self.streaming_command_queue.clone();
        let self_frame = self.frame.clone();

        videoconvert.set_property_from_str("qos", "true");

        let mut downstream_peers: HashMap<String, WebRTCPeer> = HashMap::new();

        let mut video_info =
        // default to 100x100
        gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, 100, 100)
    //         .fps(gst::Fraction::new(2, 1))
           // .flags(VideoFlags::VARIABLE_FPS)
           .fps(gstreamer::Fraction::new(self.config.fps.unwrap_or(60) as i32, 1))
            .build()
            .expect("Failed to create video info on demand for source");

        let appsrc = gstreamer_app::AppSrc::builder()
        .caps(&video_info.to_caps().expect("Cap generation failed"))
        // .is_live(true)
        .block(false)
        .is_live(true)
        .do_timestamp(true)
        .format(gstreamer::Format::Time)
        .build();

        let mut intiial_link = match INTERNAL_DEBUG {
            true => vec![&ximagesrc], //vec![appsrc.upcast_ref::<Element>()];
            false => vec![appsrc.upcast_ref::<Element>()],
        };
        // intiial_link.push(&videoupload);
        intiial_link.push(&videoconvert);
        intiial_link.push(&videoflip);
        intiial_link.push(&debug_tee);
        if config.debug || true {
            intiial_link.push(&sink);
        }

        // link
        // pipeline.add(&videoupload).expect("adding upload element to pipeline failed");
        // pipeline.add(&caps_filter_1).expect("adding capsfilter to pipeline failed");
        if !INTERNAL_DEBUG {
            pipeline.add(appsrc.upcast_ref::<Element>()).expect("adding frames source element to pipeline failed");
        }
        pipeline.add_many([&videoconvert, &videoflip, &debug_tee, &sink]).expect("adding els failed");
        gstreamer::Element::link_many(intiial_link).expect("linking failed");

        

        println!("create queue before preprocessor");
        let queue = gstreamer::ElementFactory::make("queue").build().expect("could not create queue element");
        pipeline.add(&queue).expect("adding elements to pipeline failed");
        gstreamer::Element::link_many([&debug_tee, &queue]).expect("linking failed");

        println!("initing preprocessor");

        let mut preprocessor = WebRTCPreprocessor::new_preset(self.config.encoder, self.config.optimizations);
        preprocessor.set_config(config.clone());
        preprocessor.set_default_settings();
        preprocessor.attach_to_pipeline(&pipeline, &queue);

        println!("setting up second tee element");

        let video_tee = gstreamer::ElementFactory::make("tee").name("video_tee").build().expect("could not create video tee");
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
                // if !matches!(msg.view(), MessageView::Qos(..)) {
                    println!("{:?}", msg);
                // }

                if INTERNAL_DEBUG {
                    // pipeline.debug_to_dot_file(DebugGraphDetails::all(), PathBuf::from("pipeline.dump.dot"));
                }

                match msg.view() {
                    MessageView::Eos(..) => {
                        println!("Exiting at end of stream.");
                        break;
                    },
                    MessageView::Error(err) => {
                        pipeline.set_state(gstreamer::State::Null).expect("could not reset pipeline state");
                        running = false;
                        println!("Error: {} {:?}", err.error(), err.debug());
                        return Ok(());
                    }
                    _ => (),
                }
            }
            // main_loop.context().iteration(true);

            let imsg = self.streaming_command_recv.recv()?;
            { // this block here is remenant of the old if statement
                match imsg {
                    // TODO: deduplicate code between handshake and sync, but closure does not currently work because it needs to mutate video_info
                    InternalMessage::HandshakeReceived(handshake) => {
                        println!("handshake details {:#?}", handshake);
                        let res=  handshake.resolution;
                        println!("updating to {:?}", res);
                        video_info =
                            gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                            //         .fps(gst::Fraction::new(2, 1))
                            .fps(gstreamer::Fraction::new(self.config.fps.unwrap_or(60) as i32, 1))
                                .build()
                                .expect("Failed to create video info on demand for source");
                        println!("video info {:#?}",video_info);
                        if !INTERNAL_DEBUG {
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
                        println!("syncing {:#?}", sync_details);
                        if let Some(res) = sync_details.resolution {
                            video_info =
                            gstreamer_video::VideoInfo::builder(gstreamer_video::VideoFormat::Rgba, res.0, res.1)
                            //         .fps(gst::Fraction::new(2, 1))
                                .build()
                                .expect("Failed to create video info on demand for source");
                            println!("video info {:#?}",video_info);
                            if !INTERNAL_DEBUG {
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
                        let socket =  self.get_socket();
                        // this tells the ui to switch out of the loading screen
                        if let Err(err) = socket.emit("set_session_state", json!(streamer_state_to_u8(streamer_state))) {
                            println!("Error setting session state on remote Stargate server: {:?}", err);
                        }else{
                            println!("Request to set session state on remote Stargate server.");
                        }
                    },
                    InternalMessage::SocketPeerFrontendMessageWithPipeline(origin_socketid, frontend_message) => {
                        match frontend_message {
                            StellarFrontendMessage::ProvisionWebRTC { rtc_provision_start } => {
                                println!("Provisioning webrtc for socket id {:?} client claim start: {:?}", origin_socketid, rtc_provision_start);
                                if let Err(err) = preprocessor.play() {
                                    println!("Error forceplaying preprocessor: {:?}", err);
                                }
                                let downstream_peer_el_group = webrtc::WebRTCPeer::new(origin_socketid.clone());
                                downstream_peer_el_group.setup_with_pipeline(&pipeline, &video_tee);
                                if let Ok(_) = downstream_peer_el_group.play() {

                                    let streaming_cmd_queue_for_negotiation = self.streaming_command_queue.clone();
                                    let origin_socketid_for_negotiation = origin_socketid.clone();

                                    downstream_peer_el_group.webrtcbin.connect_closure("on-negotiation-needed", false, glib::closure!(move |_webrtcbin: &gstreamer::Element| {
                                        println!("element prompted negotiation");
                                        streaming_cmd_queue_for_negotiation.send(InternalMessage::SocketOfferGeneration(origin_socketid_for_negotiation.clone()));
                                    }));

                                    let socket_arc = self.socketio_client.clone().unwrap();
                                    let origin_socketid_for_ice_sending = origin_socketid.clone();

                                    downstream_peer_el_group.webrtcbin.connect_closure("on-ice-candidate", false, glib::closure!(move |_webrtcbin: &gstreamer::Element, mlineindex: u32, candidate: &str| {
                                        println!("element got an ice candidate");
                                        let socket = socket_arc.lock().unwrap();
                                        if let Err(err) = socket.emit("send_to", json!([origin_socketid_for_ice_sending, StellarFrontendMessage::Ice { candidate: candidate.to_string(), sdp_mline_index: mlineindex }])) {
                                            println!("Error sending ice candidate to socket id {:?}: {:?}", origin_socketid_for_ice_sending, err);
                                        }
                                    }));

                                    downstream_peers.insert(origin_socketid.clone(), downstream_peer_el_group);
                                    println!("Added downstream peer to pipeline");

                                    let socket = self.get_socket();
                                    if let Err(err) = socket.emit("send_to", json!([origin_socketid, StellarFrontendMessage::ProvisionWebRTCReply { provision_ok: true }])) {
                                        println!("Error sending provision reply to socket id {:?}: {:?}", origin_socketid, err);
                                    }

                                } else {
                                    println!("Failed to play downstream peer");
                                    downstream_peer_el_group.remove_from_pipeline(&pipeline, &video_tee);
                                }
                                

                            },
                            StellarFrontendMessage::Ice { candidate, sdp_mline_index } => {
                                if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {
                                    webrtc_peer.webrtcbin.emit_by_name::<()>("add-ice-candidate", &[&sdp_mline_index, &candidate]);
                                }
                            },
                            StellarFrontendMessage::Sdp { type_, sdp } => {
                                if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {
                                    if type_ == "answer" {
                                        println!("processing sdp answer");
                                        if let Err(err) = webrtc_peer.process_sdp_answer(&sdp) {
                                            self.complain_to_socket(&origin_socketid, &format!("Error processing sdp answer from socket id {:?}", err));
                                            println!("Error processing sdp answer from socket id {:?}: {:?}", origin_socketid, err);
                                        }
                                    }else if type_ == "offer" {
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
                                    }else{
                                        println!("Unhandled sdp type {:?} from socket id {:?}", type_, origin_socketid);
                                    }
                                }
                            },
                            StellarFrontendMessage::DebugInfoRequest { debug_info_request } => {
                                let mut response = format!("DEBUG: There are {} peers connected.\n", downstream_peers.len());
                                let pipeline_base = [&videoconvert, &videoflip, &debug_tee, &queue];
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
                                }

                                if let Err(err) = self.get_socket().emit("send_to", json!([origin_socketid, StellarFrontendMessage::DebugResponse { debug: response }])) {
                                    println!("Error sending debug info request to socket id {:?}: {:?}", origin_socketid, err);
                                }

                                pipeline.debug_to_dot_file_with_ts(DebugGraphDetails::all(), PathBuf::from("/tmp/debug.dot"));
                                pipeline.debug_to_dot_file_with_ts(DebugGraphDetails::all(), PathBuf::from("pipeline.dump.dot"));
                                println!("sent pipeline dump");
                            },
                            StellarFrontendMessage::OfferRequest { offer_request_source } => {
                                if offer_request_source == "client" {
                                    self.streaming_command_queue.send(InternalMessage::SocketOfferGeneration(origin_socketid.clone()));
                                }
                            }
                            _ => {
                                println!("Unhandled frontend message {:?}", frontend_message);
                            }
                        }
                    },
                    InternalMessage::SocketSdpAnswer(origin_socketid, desc) => {
                        if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {
                            webrtc_peer.set_remote_description(&desc);
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
                        if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {
                            webrtc_peer.set_local_description(&desc);
                            let reply = StellarFrontendMessage::Sdp {
                                type_: "offer".to_string(),
                                sdp: desc.sdp().as_text().expect("Could not turn the session description into a string").to_string(),
                            };
                            let socket = self.get_socket();
                            if let Err(err) = socket.emit("send_to", json!([origin_socketid, reply])) {
                                println!("Error sending sdp answer to socket id {:?}: {:?}", origin_socketid, err);
                            }
                        }
                    },
                    InternalMessage::SocketOfferGeneration(origin_socketid) => {

                        if let Some(webrtc_peer) = downstream_peers.get(&origin_socketid) {

                            let streaming_cmd_queue_for_reply = self.streaming_command_queue.clone();

                            let promise = gstreamer::Promise::with_change_func(move |reply| {
                                
                                if let Ok(Some(offer_structref)) = reply {
                                    let offer = offer_structref
                                        .value("offer")
                                        .unwrap()
                                        .get::<gstreamer_webrtc::WebRTCSessionDescription>()
                                        .expect("Invalid argument");
                                    streaming_cmd_queue_for_reply.send(InternalMessage::SocketSdpOffer(origin_socketid.clone(), offer));
                                }else if let Err(err) = reply {
                                    println!("Error generating offer: {:?}", err);
                                }else{
                                    println!("offer generation failed");
                                }
                            });

                            webrtc_peer.webrtcbin.emit_by_name::<()>("create-offer", &[&None::<gstreamer::Structure>, &promise]);
                        }

                        

                    },
                    _ => {
                        // TODO: print more descriptive
                        println!("Unimplemented message {:#?}", imsg.type_id());
                    }
                };
            }
            if temp_update || should_update {
                update_frame_func(&appsrc, &video_info);
            }
            // TOOD: make this loop not thrash cpu by waiting for events
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
                                        StellarFrontendMessage::Test { time } => todo!(),
                                        StellarFrontendMessage::Ping { ping_payload } => todo!(),
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
