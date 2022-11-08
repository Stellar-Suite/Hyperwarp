use std::os::unix::net::UnixStream;

use crate::utils::config::Config;
use lazy_static::lazy_static;

use super::{
    connection::{Connection, Transport},
    transports::{null::NullTransport, unix::UnixTransport},
};

pub struct ApplicationHost {
    pub config: Config,
    pub connection: Option<Box<dyn Transport + Send + Sync>>,
}

impl ApplicationHost {
    pub fn new(config: Config) -> Self {
        let mut host = ApplicationHost {
            config,
            connection: None,
        };
        return host;
    }
}

fn create_host() -> ApplicationHost {
    let config = Config::from_env();
    let conn;
    match config.connection_type.as_ref() {
        "unix_client" => {
            conn = Connection::<UnixTransport>::new(UnixTransport {
                stream: UnixStream::connect("/tmp/test").expect("Unix socket connect fail. "),
            });
            let host = ApplicationHost::new(config);
            host
        }
        _ => ApplicationHost::new(config),
    }
}

lazy_static! {
    pub static ref HOST: ApplicationHost = create_host();
}
