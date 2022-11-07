use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use super::super::connection::{Connection, Transport};

struct UnixTransport {
    stream: UnixStream,
}

impl Transport for UnixTransport {
    fn send(&mut self, data: &[u8]) -> Result<(), std::io::Error> {
        self.stream.write(data)?;
        Ok(())
    }

    fn recv(&mut self, data: &mut [u8]) -> Result<(), std::io::Error> {
        self.stream.read(data)?;
        Ok(())
    }
}
