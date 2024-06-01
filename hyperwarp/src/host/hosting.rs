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
    pub capture_helper: Option<CaptureHelper>
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
        };
        return host;
    }

    pub fn start(&mut self) {
        // TODO: start server
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
        println!("Selected Connection type: {}", config.connection_type);
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
