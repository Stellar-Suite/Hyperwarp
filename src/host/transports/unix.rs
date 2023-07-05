use std::{
    io::{Read, Write, Error},
    os::unix::net::{UnixStream, UnixListener}, sync::{Mutex, Arc},
};

use crate::host::connection::Transporter;

use super::super::connection::{Connection, Transport};

pub struct UnixTransport {
    pub stream: UnixStream,
}

impl Clone for UnixTransport {
    fn clone(&self) -> Self {
        Self {
            stream: self
                .stream
                .try_clone()
                .expect("Unix stream clone failure. "),
        }
    }
}

impl Transport for UnixTransport {
    fn init(&mut self) -> Result<(), std::io::Error> {
        self.stream.set_nonblocking(true)?;
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<bool, std::io::Error> {
        match self.stream.write(data) {
            Ok(_) => return Ok(false),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(true);
                }
                return Err(e);
            }
        }
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<bool, std::io::Error> {
        match self.stream.read(data) {
            Ok(_) => return Ok(false),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(true);
                }
                return Err(e);
            }
        }
    }
}

pub struct UnixTransporter {
    pub transports: Arc<Mutex<Vec<Box<dyn Transport + Send + Sync>>>>,
}

impl Transporter for UnixTransporter {
    fn get_transports(&self) -> Arc<Mutex<Vec<Box<dyn Transport + Send + Sync>>>> {
        self.transports.clone()
    }
}

impl UnixTransporter {
    pub fn new_with_unix_transport(ut: UnixTransport) -> UnixTransporter {
        UnixTransporter {
            transports: Arc::new(Mutex::new(vec![Box::new(ut)])),
        }
    }
}

pub struct UnixListenerTransporter {
    pub transports: Arc<Mutex<Vec<Box<dyn Transport + Send + Sync>>>>,
    pub listener: UnixListener,
}

impl Transporter for UnixListenerTransporter {
    fn get_transports(&self) -> Arc<Mutex<Vec<Box<dyn Transport + Send + Sync>>>> {
        self.transports.clone()
    }

    fn init(&mut self) -> Result<(), Error>{
        self.listener.set_nonblocking(true)
    }
}

impl UnixListenerTransporter {
    pub fn new_with_path(path: &str) -> UnixListenerTransporter {
        UnixListenerTransporter {
            transports: Arc::new(Mutex::new(vec![])),
            listener: UnixListener::bind(path).expect("Unix listener bind failure. "),
        }
    }

    pub fn new_with_listener(listener: UnixListener) -> UnixListenerTransporter {
        UnixListenerTransporter {
            transports: Arc::new(Mutex::new(vec![])),
            listener: listener,
        }
    }

    fn test(&mut self){

    }
}