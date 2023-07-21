// use std::error::Error;
use std::io::Error;
use std::rc::Rc;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::message::Message;

pub const MAX_PAYLOAD: usize = 1024 * 1024 * 10; // 10 MB

pub trait TransportLink {
    fn send(&mut self, data: &[u8]) -> Result<bool, Error>; // first is would block
    fn send_vec(&mut self, data: &Vec<u8>) -> Result<bool, Error>;
    fn recv(&mut self, data: &mut [u8]) -> Result<bool, Error>; // first is would block
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    } // this should block until we establish a connection
    fn close(&mut self) -> Result<(), Error> {
        Ok(())
    } // this should block until we close the connection
    fn is_connected(&self) -> bool {
        true
    }
}

static nextId: Mutex<u64> = Mutex::new(1);
static CAP: u64 = 0xFFFFFFFFFFFFFFF;

// cycling id generator
pub fn gen_id() -> u64{
    let mut locked = nextId.lock().unwrap();
    let id = *locked;
    *locked += 1;
    if(*locked > CAP){
        *locked = 1;
    }

    id
}


pub struct TransportIdentity {
    pub id: u64,
    pub flags: u64,
}

impl TransportIdentity {
    pub fn new() -> Self {
        TransportIdentity {
            id: gen_id(),
            flags: 0,
        }
    }
}

pub struct Transport {
    pub link: Box<dyn TransportLink + Send + Sync>,
    pub identity: TransportIdentity,
}

impl Transport {
    pub fn new(link: Box<dyn TransportLink + Send + Sync>) -> Self {
        Transport {
            link: link,
            identity: TransportIdentity::new(),
        }
    }
}

// handles multiple transports
pub trait Transporter {
    fn get_transports(&self) -> Arc<Mutex<Vec<Transport>>>;

    fn init(&mut self) -> Result<(), Error>{
        // TODO: impl
        Ok(())
    }

    fn tick(&mut self) -> Result<(), Error>{
        // TODO: impl
        Ok(())
    }


    fn close(&mut self) -> Result<(), Error>{
        // TODO: impl
        Ok(())
    }
}

pub struct ConnectionManager {
    pub transporter: Arc<Mutex<dyn Transporter + Send + Sync>>, // super nesting lol
}

/*fn test() {
    let a = mpsc::channel();
    let b = mpsc::channel();
}*/

impl ConnectionManager {
    // 'static mem leak?
    pub fn new(transporter: impl Transporter + Send + Sync + 'static) -> Self {
        let conn = ConnectionManager {
            transporter: Arc::new(Mutex::new(transporter))
        };

        conn
    }

    pub fn start(&mut self) {
        let transport = &mut self.transporter;
        transport
            .lock()
            .unwrap()
            .init()
            .expect("Transporter initalization failure. ");
    }
}

pub fn get_empty_transports_vec() -> Vec<Transport> {
    return vec![];
}