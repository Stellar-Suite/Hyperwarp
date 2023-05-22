use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex, MutexGuard}, fs::File, io::Write,
};

use std::thread; // for test func

use crate::utils::config::Config;
use lazy_static::lazy_static;

use super::{
    connection::{Connection, Transport},
    transports::{null::NullTransport, unix::UnixTransport},
    feature_flags::FeatureFlags, host_behavior::{HostBehavior, DefaultHostBehavior},
};

pub struct ApplicationHost {
    pub config: Config,
    pub connection: Option<Arc<Mutex<Connection>>>,
    pub features: Mutex<FeatureFlags>,
    pub behavior: Arc<Mutex<Box<dyn HostBehavior + Send>>>,
}

impl ApplicationHost {
    pub fn new(config: Config) -> Self {
        let host = ApplicationHost {
            config,
            connection: None,
            features: Mutex::new(FeatureFlags::new()),
            behavior: Arc::new(Mutex::new(Box::new(DefaultHostBehavior::new())))
        };
        return host;
    }

    pub fn start(&mut self) {
        if let Some(conn) = &self.connection {
            conn.lock().unwrap().start();
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
        println!("test func called on thread {:?}", thread::current().id());
    }
}

fn create_host() -> ApplicationHost {
    let config = Config::from_env();
    if config.debug_mode {
        println!("Selected Connection type: {}", config.connection_type);
        println!("Host config: {:?}", config);
    }
    let host = match config.connection_type.as_ref() {
        "unix_client" => {
            let unix_socket_path = config.unix_socket_path.as_ref().expect("Unix socket path should be set in config");
            let conn = Connection::new(UnixTransport {
                stream: UnixStream::connect(unix_socket_path).expect("Unix socket connect fail. "),
            });
            let mut host = ApplicationHost::new(config);
            host.connection = Some(Arc::new(Mutex::new(conn)));
            host.start();
            host
        }
        _ => ApplicationHost::new(config),
    };
    host.log();
    if let Some(ref conn_arc) = host.connection {
        host
    } else {
        if host.config.debug_mode {
            println!("No connection type specified. ");
        }
        host
    }
}

lazy_static! {
    // so look here, this might be unsafe yk, but all the important things are behind mutexes
    pub static ref HOST: ApplicationHost = create_host();
}
