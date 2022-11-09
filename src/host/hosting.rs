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
        let mut host = ApplicationHost {
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
    let mut host = match config.connection_type.as_ref() {
        "unix_client" => {
            let conn = Connection::new(UnixTransport {
                stream: UnixStream::connect("/tmp/test").expect("Unix socket connect fail. "),
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
        host
    }
}

lazy_static! {
    pub static ref HOST: ApplicationHost = create_host();
}
