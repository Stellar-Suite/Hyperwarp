use std::{
    io::{Read, Write, Error},
    os::unix::net::{UnixStream, UnixListener}, sync::{Mutex, Arc},
};

use crate::host::{connection::Transporter, hosting::HOST};

use super::super::connection::{Connection, Transport};

pub struct UnixTransport {
    pub stream: UnixStream,
    pub closed: bool,
}

impl Clone for UnixTransport {
    fn clone(&self) -> Self {
        Self {
            stream: self
                .stream
                .try_clone()
                .expect("Unix stream clone failure. "),
            closed: self.closed,
        }
    }
}

impl Transport for UnixTransport {
    fn init(&mut self) -> Result<(), std::io::Error> {
        self.stream.set_nonblocking(true)?;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        !self.closed
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
        // normal read allows partial data reads and we don't want that when we always know how big our data is going to be. 
        match self.stream.read_exact(data) {
            Ok(_old_bytes_now_nothing) => {
                // println!("read {:?}", _bytes);
                /*if bytes == 0 {
                    self.closed = true;
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        "Connection closed"
                    ));
                }*/
                return Ok(false);
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(true);
                }
                // oh no something went wrong
                // TODO: handle errors cooler
                self.closed = true;
                self.stream.shutdown(std::net::Shutdown::Both)?;
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

    fn tick(&mut self) -> Result<(), Error>{
        match self.listener.accept() {
            Ok(stream_and_addr) => {
                let (stream, addr) = stream_and_addr;
                if HOST.config.debug_mode {
                    println!("New connection from {:?}", addr);
                }
                let mut transport = UnixTransport {
                    stream: stream,
                    closed: false
                };
                transport.init()?; // shouldnt cause issues as it only sets nonblocking on that too
                self.transports.lock().unwrap().push(Box::new(transport));
                Ok(())
            },
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    return Ok(());
                }
                return Err(e);
            }
        }
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
    
}