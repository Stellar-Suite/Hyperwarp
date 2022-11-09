use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

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
