use backtrace::Backtrace;
use message_io::adapters::unix_socket::{create_null_socketaddr, UnixSocketListenConfig};
use message_io::network::{Endpoint, NetEvent, Transport, TransportListen};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};

use stellar_protocol::protocol::{get_all_channels, Handshake, StellarChannel, StellarMessage, Synchornization};

use crossbeam_queue::SegQueue;

use std::any::Any;
use std::path::PathBuf;
use std::sync::RwLock;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    os::unix::net::UnixStream,
    sync::{Arc, Mutex, MutexGuard},
};

use std::thread; // for test func

use crate::shim;
use crate::utils::{config::Config, pointer::Pointer};
use lazy_static::lazy_static;

use super::window::Window;
use super::{
    feature_flags::FeatureFlags,
    host_behavior::{DefaultHostBehavior, HostBehavior},
};

pub struct CaptureHelper {
    pub frameFile: Option<Mutex<File>>,
}

pub enum MainTickMessage {
    RequestResolutionBroadcast(Endpoint),
    RequestShImgPath(Endpoint),
    RequestHandshake(Endpoint),
}

pub struct LastSentState {
    pub resolution: (u32, u32),
}

pub struct ApplicationHost {
    pub config: Arc<Config>,
    pub features: Mutex<FeatureFlags>,
    pub behavior: Arc<Mutex<Box<dyn HostBehavior + Send>>>,
    pub func_pointers: Mutex<HashMap<String, Pointer>>,
    pub capture_helper: Option<CaptureHelper>,
    pub messaging_handler: Option<Arc<Mutex<NodeHandler<InternalSignals>>>>,
    pub command_queue: Arc<SegQueue<MainTickMessage>>,
    pub last_sent_state: Arc<RwLock<LastSentState>>, // TODO: remove this arc rwlock if perf is hit hard enough here, may be able to unsafe it
}

#[derive(Debug)]
pub enum InternalSignals {
    TestSignal,
    TracingSignal,
    NewFrameSignal,
    SendToChannelSignal(StellarChannel, StellarMessage),
}

impl ApplicationHost {
    pub fn new(config: Config) -> Self {
        let mut default_behavior = DefaultHostBehavior::new();
        if config.debug_mode {
            println!("Spawning writer thread...");
        }
        let handle = default_behavior.spawn_writer_thread(&config);
        if config.debug_mode {
            println!("Default behavior thread handle: {:?}", handle);
        }
        let host = ApplicationHost {
            config: Arc::new(config),
            features: Mutex::new(FeatureFlags::new()),
            behavior: Arc::new(Mutex::new(Box::new(default_behavior))),
            func_pointers: Mutex::new(HashMap::new()),
            capture_helper: None,
            messaging_handler: None,
            command_queue: Arc::new(SegQueue::new()),
            last_sent_state: Arc::new(RwLock::new(LastSentState { resolution: (0, 0) })),
        };
        return host;
    }

    pub fn get_unix_socket_path(&self) -> PathBuf {
        let mut path = PathBuf::from("/tmp");
        path.push(format!("hw-{}.sock", self.config.session_id));
        path
    }

    pub fn get_handshake(&self) -> Handshake {
        let resolution = self.get_behavior().get_fb_size().or_else(|| Some((0,0))).unwrap();
        let shimg_path = self.get_behavior().get_shimg_path(&self.config);
        let handshake = Handshake {
            resolution: resolution,
            shimg_path: shimg_path,
        };
        handshake
    }

    pub fn tick(&self) {
        self.get_behavior().tick();

        // process commands from queue
        if self.config.tracing_mode {
            println!("tick()");
        }

        // compute changes in state
        let mut state_changed = false;
        {
            let mut last_sent_state = self.last_sent_state.write().unwrap();
            if last_sent_state.resolution != self.get_behavior().get_fb_size().unwrap() {
                state_changed = true;
                last_sent_state.resolution = self.get_behavior().get_fb_size().unwrap();
            }


        }

        if state_changed {
            // send a sync message
            println!("changes detected, doing sync signal");
            self.sync();
        }

        match self.command_queue.pop() {
            Some(command) => match command {
                MainTickMessage::RequestResolutionBroadcast(endpoint) => {
                    if self.config.debug_mode {
                        println!("Responding to resolution request from {:?} with {:?}", endpoint.addr(), self.get_behavior().get_fb_size());
                    }
                    self.send_to(endpoint, &StellarMessage::ResolutionBroadcastResponse(self.get_behavior().get_fb_size()));
                    if self.config.debug_mode {
                        println!("Resolution response sent!");
                    }
                },
                MainTickMessage::RequestShImgPath(endpoint) => {
                    let path = self.get_behavior().get_shimg_path(&self.config);
                    if self.config.debug_mode {
                        println!("Responding to shimg path request from {:?} with {:?}", endpoint.addr(), path);
                    }
                    let path_copy = path.clone();
                    self.send_to(endpoint.clone(), &StellarMessage::ShImgPathResponseStruct(path));
                    self.send_to(endpoint.clone(), &StellarMessage::ShImgPathResponse(path_copy.display().to_string()));
                },
                MainTickMessage::RequestHandshake(endpoint) => {
                    if self.config.debug_mode {
                        println!("Responding to handshake request from {:?}", endpoint.addr());
                    }
                    let handshake = self.get_handshake();
                    self.send_to(endpoint, &StellarMessage::HandshakeResponse(handshake));
                }
            },
            None => {}
        }
    }

    pub fn sync(&self){
        if let Some(handler) = &self.messaging_handler {
            let handler = handler.lock().unwrap();
            let sync_details = self.get_sync(); // this is here so it is more up to date
            handler.signals().send(InternalSignals::SendToChannelSignal(StellarChannel::Synchornizations, StellarMessage::SynchronizationEvent(sync_details)));
        }
    }

    pub fn get_sync(&self) -> Synchornization {
        Synchornization {
            resolution: self.get_behavior().get_fb_size(),
        }
    }

    pub fn start_server(&mut self) {
        let (handler, listener) = node::split::<InternalSignals>();

        // bind unix always
        let unix_socket_path = match self.config.unix_socket_path.clone() {
            Some(path) => path.into(),
            None => self.get_unix_socket_path(),
        };

        if self.config.debug_mode {
            println!("Listening on unix socket: {}", unix_socket_path.display());
        }

        handler
            .network()
            .listen_with(
                TransportListen::UnixDatagramSocket(UnixSocketListenConfig::new(unix_socket_path)),
                create_null_socketaddr(),
            )
            .expect("Opening unix control socket failed.");

        if let Some(bind_type) = &self.config.bind_type {
            let addr = self.config.bind_addr.expect("bind address not set");
            if self.config.debug_mode {
                println!("Binding to address: {} (proto: {})", addr, bind_type);
            }
            match bind_type.as_str() {
                "udp" => {
                    handler.network().listen(Transport::Udp, addr).unwrap();
                }
                "tcp" => {
                    handler.network().listen(Transport::Tcp, addr).unwrap();
                }
                _ => {
                    panic!("unknown bind type");
                }
            }
        }

        let config = self.config.clone();

        let handler_wrapper = Arc::new(Mutex::new(handler));
        let handler_wrapper_2 = handler_wrapper.clone();
        let handler_wrapper_3 = handler_wrapper.clone();
        let handler_signals = handler_wrapper.clone();

        let command_queue = self.command_queue.clone();

        std::thread::spawn(move || {
            // let mut frame_sub_endpoints: Vec<Endpoint> = vec![];
            let mut pubsub: HashMap<StellarChannel, Vec<Endpoint>> = HashMap::new();
            for channel in stellar_protocol::protocol::get_all_channels() {
                pubsub.insert(channel, vec![]);
            }

            let check_subscribers = |subscribers: &mut Vec<Endpoint>| {
                let handler_locked = handler_wrapper_3.lock().unwrap();
                subscribers.retain_mut(|target_endpoint| {
                    handler_locked
                        .network()
                        .is_ready(target_endpoint.resource_id())
                        .is_some()
                });
            };

            let check_all = || {
                for channel in stellar_protocol::protocol::get_all_channels() {
                    check_subscribers(pubsub.get_mut(&channel).unwrap());
                }
            };

            let send_main_tick_request = |message: MainTickMessage| {
                command_queue.push(message);
            };

            listener.for_each(move |event| {
                // println!("got event: {:?}", event);
                match event {
                    NodeEvent::Network(netevent) => {
                        match netevent {
                            NetEvent::Connected(endpoint, ready) => {
                                if !ready {
                                    if config.debug_mode {
                                        println!("One client did not successfully ready. {}", endpoint.addr());
                                    }
                                }
                            }
                            NetEvent::Accepted(_endpoint, _listener) => {}
                            NetEvent::Message(endpoint, data) => {
                                if config.debug_mode {
                                    println!("I got a message from {:?}", endpoint.addr());
                                }
                                match stellar_protocol::deserialize_safe(data) { 
                                    Some(message) => {
                                        match message {
                                            StellarMessage::ResolutionRequest => {
                                                if config.debug_mode {
                                                    println!("Attempting to fufill resolution request from {:?}", endpoint.addr());
                                                }
                                                send_main_tick_request(MainTickMessage::RequestResolutionBroadcast(endpoint));
                                            },
                                            StellarMessage::ShImgPathRequest => {
                                                if config.debug_mode {
                                                    println!("Attempting to fufill shimg path request from {:?}", endpoint.addr());
                                                }
                                                send_main_tick_request(MainTickMessage::RequestShImgPath(endpoint));
                                            },
                                            StellarMessage::HandshakeRequest => {
                                                if config.debug_mode {
                                                    println!("Attempting to fufill handshake request from {:?}", endpoint.addr());
                                                }
                                                send_main_tick_request(MainTickMessage::RequestHandshake(endpoint));
                                            },
                                            StellarMessage::Hello => {
                                                if config.debug_mode {
                                                    println!("Hello message received from {:?}", endpoint.addr());
                                                }
                                            },
                                            StellarMessage::SubscribeChannel(channel) => {
                                                if config.debug_mode {
                                                    println!("Subscribing to channel {:?} from {:?}", channel, endpoint.addr());
                                                }
                                                pubsub.get_mut(&channel).unwrap().push(endpoint.clone());
                                            },
                                            _ => {
                                                if config.debug_mode {
                                                    println!("Unhandled message: {:?}", message);
                                                }
                                            }
                                        }
                                    },
                                    None => {
                                        if config.debug_mode {
                                            println!("Error deserializing message (malformed?):");
                                        }
                                    }
                                }

                                // ex. reply
                                // old echo test
                                /*handler_wrapper
                                    .lock()
                                    .unwrap()
                                    .network()
                                    .send(endpoint, data);*/
                            }
                            NetEvent::Disconnected(_endpoint) => {
                                if config.debug_mode {
                                    println!("One client disconnected. {}", _endpoint.addr());
                                }
                            }
                        }
                    }
                    NodeEvent::Signal(signal) => {
                        match signal {
                            // WARNING: DO NOT SEND A SIGNAL HERE CAUSE IT'LL BE DEADLOCKED I THINK
                            // APPARENTLY NOT
                            InternalSignals::TestSignal => {}
                            InternalSignals::TracingSignal => {}
                            InternalSignals::NewFrameSignal => {
                                check_subscribers(&mut pubsub.get_mut(&StellarChannel::Frame).unwrap());
                                for subscriber in pubsub.get_mut(&StellarChannel::Frame).unwrap() {
                                    handler_wrapper
                                        .lock()
                                        .unwrap()
                                        .network()
                                        .send(subscriber.clone(), &stellar_protocol::serialize(&StellarMessage::NewFrame));
                                }
                            },
                            InternalSignals::SendToChannelSignal(channel, message) => {
                                if let Some(subscribers) = pubsub.get_mut(&channel) {
                                    for subscriber in subscribers {
                                        handler_wrapper
                                            .lock()
                                            .unwrap()
                                            .network()
                                            .send(subscriber.clone(), &stellar_protocol::serialize(&message));
                                    }
                                }
                            },
                            _ => {
                                // log unhandled signal
                                if config.debug_mode {
                                    println!("Unhandled signal: {:?}", signal.type_id());
                                }
                            }
                        }
                    }
                }
            });
        });

        self.messaging_handler = Some(handler_wrapper_2);

        // handler.signals().send(event)
        // self.messaging_pair = Some((handler, listener));
    }

    pub fn notify_frame(&self) {
        if let Some(handler) = &self.messaging_handler {
            let handler = handler.lock().unwrap();
            handler.signals().send(InternalSignals::NewFrameSignal);
        }
    }

    pub fn send_to(&self, endpoint: Endpoint, message: &StellarMessage) -> bool {
        if let Some(handler) = &self.messaging_handler {
            let handler = handler.lock().unwrap();
            handler
                .network()
                .send(endpoint, &stellar_protocol::serialize(message));
            return true;
        }
        false
    }

    pub fn start(&mut self) {
        if !self.config.disable_control {
            self.start_server();
        } else {
            if self.config.debug_mode {
                println!("Control disabled. Not starting server.");
            }
        }
        if self.config.capture_mode {
            self.capture_helper = Some(CaptureHelper { frameFile: None });
        }
    }

    pub fn log(&self) {
        // println!("Running debug");
        let file_create_result = File::create(format!("/tmp/hw_debug_{}", std::process::id()));
        // println!("File create result: {:?}", file_create_result);
        if let Ok(mut file) = file_create_result {
            let log_data = format!(
                "thread id: {:?}, cwd: {:?}, args: {:?}\r\n",
                thread::current().id(),
                std::env::current_dir().unwrap(),
                std::env::args()
            );
            file.write_all(log_data.as_bytes()).unwrap();
        }
    }

    pub fn get_behavior(&self) -> MutexGuard<Box<dyn HostBehavior + Send>> {
        self.behavior.lock().unwrap()
    }

    pub fn test(&self) {
        // TODO: supress test func in non-debug mode
        println!("test func called on thread {:?}", thread::current().id());
    }

    pub fn onFrameSwapBegin(&self) {
        self.get_behavior().onFrameSwapBegin();
        self.tick();
    }

    pub fn onFrameSwapEnd(&self) {
        self.get_behavior().onFrameSwapEnd();
        self.tick();
        self.notify_frame();
        self.tick();
    }

    pub fn onWindowCreate(
        &self,
        win: Window,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) {
        self.get_behavior().onWindowCreate(win, x, y, width, height);
        self.tick();
    }
}

fn create_host() -> ApplicationHost {
    let config = Config::from_env();
    if config.debug_mode {
        // println!("Selected Connection type: {}", config.connection_type);
        println!("Host config: {:?}", config);
        let bt = Backtrace::new();
        println!("Startup backtrace: {:?}", bt);
    }
    let host = {
        let mut host = ApplicationHost::new(config);
        host.start();
        host
    };

    host.log();
    host
}

lazy_static! {
    // so look here, this might be unsafe yk, but all the important things are behind mutexes
    pub static ref HOST: ApplicationHost = create_host();
}
