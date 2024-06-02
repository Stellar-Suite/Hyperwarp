use message_io::adapters::unix_socket::{create_null_socketaddr, UnixSocketListenConfig};
use message_io::network::{Endpoint, NetEvent, Transport, TransportListen};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};

use stellar_protocol::protocol::{get_all_channels, StellarChannel, StellarMessage};

use crossbeam_queue::SegQueue;

use std::any::Any;
use std::path::PathBuf;
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    os::unix::net::UnixStream,
    sync::{Arc, Mutex, MutexGuard},
};

use std::thread; // for test func

use crate::utils::{config::Config, pointer::Pointer};
use lazy_static::lazy_static;

use super::{
    feature_flags::FeatureFlags,
    host_behavior::{DefaultHostBehavior, HostBehavior},
};

pub struct CaptureHelper {
    pub frameFile: Option<Mutex<File>>,
}

pub enum MainTickMessage {
    RequestResolutionBroadcast(Endpoint),
}

pub struct ApplicationHost {
    pub config: Arc<Config>,
    pub features: Mutex<FeatureFlags>,
    pub behavior: Arc<Mutex<Box<dyn HostBehavior + Send>>>,
    pub func_pointers: Mutex<HashMap<String, Pointer>>,
    pub capture_helper: Option<CaptureHelper>,
    pub messaging_handler: Option<Arc<Mutex<NodeHandler<InternalSignals>>>>,
    pub command_queue: Arc<SegQueue<MainTickMessage>>,
}

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
        };
        return host;
    }

    pub fn get_unix_socket_path(&self) -> PathBuf {
        let mut path = PathBuf::from("/tmp");
        path.push(format!("hw-{}.sock", self.config.session_id));
        path
    }

    pub fn tick(&self) {
        self.get_behavior().tick();

        match self.command_queue.pop() {
            Some(command) => match command {
                MainTickMessage::RequestResolutionBroadcast(endpoint) => {
                    self.send_to(endpoint, &StellarMessage::ResolutionBroadcastResponse(self.get_behavior().get_fb_size()));
                },
            },
            None => {}
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
                TransportListen::UnixSocket(UnixSocketListenConfig::new(unix_socket_path)),
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

                                match stellar_protocol::deserialize_safe(data) { 
                                    Some(message) => {
                                        match message {
                                            StellarMessage::RequestResolutionBroadcast => {
                                                command_queue.push(MainTickMessage::RequestResolutionBroadcast(endpoint));
                                            },
                                            StellarMessage::Hello => {
                                                if config.debug_mode {
                                                    println!("Hello message received from {:?}", endpoint.addr());
                                                }
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
                            InternalSignals::TestSignal => {}
                            InternalSignals::TracingSignal => {}
                            InternalSignals::NewFrameSignal => {}
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
        self.start_server();
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
}

fn create_host() -> ApplicationHost {
    let config = Config::from_env();
    if config.debug_mode {
        // println!("Selected Connection type: {}", config.connection_type);
        println!("Host config: {:?}", config);
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
