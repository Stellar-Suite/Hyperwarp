use message_io::adapters::unix_socket::{create_null_socketaddr, UnixSocketListenConfig};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};
use message_io::network::{NetEvent, Transport, TransportListen};

use std::path::PathBuf;
use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex, MutexGuard}, fs::File, io::Write, collections::HashMap,
};

use std::thread; // for test func

use crate::utils::{config::Config, pointer::Pointer};
use lazy_static::lazy_static;

use super::{
    feature_flags::FeatureFlags, host_behavior::{HostBehavior, DefaultHostBehavior},
};

pub struct CaptureHelper {
    pub frameFile: Option<Mutex<File>>,
}


pub struct ApplicationHost {
    pub config: Config,
    pub features: Mutex<FeatureFlags>,
    pub behavior: Arc<Mutex<Box<dyn HostBehavior + Send>>>,
    pub func_pointers: Mutex<HashMap<String, Pointer>>,
    pub capture_helper: Option<CaptureHelper>,
    pub messaging_handler: Option<Arc<Mutex<NodeHandler<()>>>>,
}

pub enum InternalSignals {
    TestSignal,
    TracingSignal,
    NewFrameSignal,
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
            config,
            features: Mutex::new(FeatureFlags::new()),
            behavior: Arc::new(Mutex::new(Box::new(default_behavior))),
            func_pointers: Mutex::new(HashMap::new()),
            capture_helper: None,
            messaging_handler: None,
        };
        return host;
    }

    pub fn get_unix_socket_path(&self) -> PathBuf {
        let mut path = PathBuf::from("/tmp");
        path.push(format!("hw-{}.sock", self.config.session_id));
        path
    }

    pub fn start_server(&mut self){

        let (handler, listener) = node::split::<()>();
        
        // bind unix always
        let unix_socket_path = match self.config.unix_socket_path.clone() {
            Some(path) => path.into(),
            None => self.get_unix_socket_path(),
        };

        if self.config.debug_mode {
            println!("Listening on unix socket: {}", unix_socket_path.display());
        }

        handler.network().listen_with(TransportListen::UnixSocket(UnixSocketListenConfig::new(unix_socket_path)), create_null_socketaddr()).expect("Opening unix control socket failed.");

        if let Some(bind_type) = &self.config.bind_type {
            let addr = self.config.bind_addr.expect("bind address not set");
            if self.config.debug_mode {
                println!("Binding to address: {} (proto: {})", addr, bind_type);
            }
            match bind_type.as_str() {
                "udp" => {
                    handler.network().listen(Transport::Udp, addr).unwrap();
                },
                "tcp" => {
                    handler.network().listen(Transport::Tcp, addr).unwrap();
                },
                _ => {
                    panic!("unknown bind type");
                }
            }
           
        }

        let is_debug = self.config.debug_mode;
        let is_tracing = self.config.tracing_mode;

        let handler_wrapper = Arc::new(Mutex::new(handler));
        let handler_wrapper_2 = handler_wrapper.clone();

        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => {
                
            },
            NetEvent::Accepted(_endpoint, _listener) => {

            },
            NetEvent::Message(endpoint, data) => {
                // ex. reply
                handler_wrapper.lock().unwrap().network().send(endpoint, data);
            },
            NetEvent::Disconnected(_endpoint) => {
                if is_debug {
                    println!("One client disconnected. {}", _endpoint.addr());
                }
            }
        });

        self.messaging_handler = Some(handler_wrapper_2);

        // handler.signals().send(event)
        // self.messaging_pair = Some((handler, listener));
        
    }

    pub fn start(&mut self) {
        // TODO: start server
        self.start_server();
        if self.config.capture_mode {
            self.capture_helper = Some(CaptureHelper {
                frameFile: None,
            });
        }
    }

    pub fn log(&self){
        // println!("Running debug");
        let file_create_result = File::create(format!("/tmp/hw_debug_{}", std::process::id()));
        // println!("File create result: {:?}", file_create_result);
        if let Ok(mut file) = file_create_result {
            let log_data = format!("thread id: {:?}, cwd: {:?}, args: {:?}\r\n",thread::current().id(),std::env::current_dir().unwrap(), std::env::args());
            file.write_all(log_data.as_bytes()).unwrap();
        }
    }

    pub fn get_behavior(&self) -> MutexGuard<Box<dyn HostBehavior + Send>> {
        self.behavior.lock().unwrap()
    }

    pub fn test(&self){
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
    let host =  {
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
