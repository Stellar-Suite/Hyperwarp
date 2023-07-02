use std::{
    io::{Read, Write},
    os::unix::net::UnixStream, sync::Mutex,
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
        // self.stream.set_nonblocking(true)?;
        Ok(())
    }

    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.stream.write(data)?;
        Ok(())
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<(), std::io::Error> {
        self.stream.read(data)?;
        Ok(())
    }
}

pub struct UnixTransporter {
    pub transports: Vec<Box<Mutex<dyn Transport + Send + Sync>>>,
}

impl Transporter for UnixTransporter {
    fn get_transports(&self) -> &Vec<Box<Mutex<dyn Transport + Send + Sync>>> {
        &self.transports
    }
}

impl UnixTransporter {
    pub fn new_with_unix_transport(ut: UnixTransport) -> UnixTransporter {
        UnixTransporter {
            transports: vec![Box::new(Mutex::new(ut))],
        }
    }
}