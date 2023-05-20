use std::{
    os::unix::net::UnixStream,
    sync::{Arc, Mutex},
};

use crate::utils::config::Config;
use lazy_static::lazy_static;

use super::{
    connection::{Connection, Transport},
    transports::{null::NullTransport, unix::UnixTransport},
};

#[derive(Clone)]
pub struct ApplicationHost {
    pub config: Config,
    pub connection: Option<Arc<Mutex<Connection>>>,
}

impl ApplicationHost {
    pub fn new(config: Config) -> Self {
        let host = ApplicationHost {
            config,
            connection: None,
        };
        return host;
    }

    pub fn start(&mut self) {
        if let Some(conn) = &self.connection {
            conn.lock().unwrap().start();
        }
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
    pub static ref HOST: ApplicationHost = create_host();
}
