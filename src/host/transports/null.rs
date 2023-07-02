use std::{
    io::{Read, Write},
    os::unix::net::UnixStream, sync::Mutex,
};

use crate::host::connection::{Transporter, empty_transports};

use super::super::connection::{Connection, Transport};

#[derive(Copy, Clone, Debug)] // debug should be easy
pub struct NullTransport {}

impl Transport for NullTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        /*Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))*/
        Ok(())
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<(), std::io::Error> {
        /*Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))*/
        Ok(())
    }
}

pub struct NullTransporter {}

impl Transporter for NullTransporter {
    fn get_transports(&self) -> &Vec<Box<Mutex<dyn Transport + Send + Sync>>> {
        &empty_transports // vec![Box::new(NullTransport {})]
    }
}