use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use super::super::connection::{Connection, Transport};

#[derive(Copy, Clone, Debug)] // debug should be easy
pub struct NullTransport {}

impl Transport for NullTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }
}
