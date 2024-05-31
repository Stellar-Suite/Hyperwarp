use std::{
    io::{Read, Write},
    os::unix::net::UnixStream, sync::{Mutex, Arc},
};

use crate::host::connection::{Transporter, get_empty_transports_vec, Transport};

use super::super::connection::{ConnectionManager, TransportLink};

#[derive(Copy, Clone, Debug)] // debug should be easy
pub struct NullTransport {}

impl TransportLink for NullTransport {
    fn send(&mut self, data: &[u8]) -> Result<bool, std::io::Error> {
        /*Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))*/
        Ok(false)
    }

    fn send_vec(&mut self, data: &Vec<u8>) -> Result<bool, std::io::Error> {
        /*Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))*/
        Ok(false)
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<bool, std::io::Error> {
        /*Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))*/
        Ok(false)
    }
}

pub struct NullTransporter {}

impl Transporter for NullTransporter {
    fn get_transports(&self) -> Arc<Mutex<Vec<Transport>>> {
        Arc::new(Mutex::new(get_empty_transports_vec())) // vec![Box::new(NullTransport {})]
    }
}