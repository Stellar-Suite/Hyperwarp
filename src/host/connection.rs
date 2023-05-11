// use std::error::Error;
use std::io::Error;
use std::rc::Rc;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::message::Message;

pub const MAX_PAYLOAD: usize = 1024 * 1024 * 10; // 10 MB

pub trait Transport {
    fn send(&mut self, data: &[u8]) -> Result<(), Error>; // block
    fn recv(&mut self, data: &mut [u8]) -> Result<(), Error>; // block
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    } // this should block until we establish a connection
}

pub struct Connection {
    pub transport: Arc<Mutex<dyn Transport + Send + Sync>>, // super nesting lol
                                                            // message output queue
                                                            // outgoing: (Sender<Message>, Receiver<Message>),
                                                            // message input queue
                                                            // ingoing: (Sender<Message>, Receiver<Message>),
}

/*fn test() {
    let a = mpsc::channel();
    let b = mpsc::channel();
}*/

impl Connection {
    pub fn new(transport: impl Transport + Send + Sync + 'static) -> Self {
        // let (tx1, rx1) = mpsc::channel();
        // let (tx2, rx2) = mpsc::channel();
        let conn = Connection {
            transport: Arc::new(Mutex::new(transport)),
            // outgoing: mpsc::channel(), // (tx1, rx1),
            // ingoing: mpsc::channel(),  // (tx2, rx2),
        };

        conn
    }

    pub fn start(&mut self) {
        let transport = &mut self.transport;
        transport
            .lock()
            .unwrap()
            .init()
            .expect("Transport initalization failure. ");
        /* thread::spawn(move || {
            transport.init().expect("Transport initalization fail. ");
            // read/write queue here
        }); */
        // rw threads
        /*let recv_transport = transport.clone();
        thread::spawn(move || {
            let buf = [0; MAX_PAYLOAD];
            loop {
                transport.recv(&mut buf).expect("Transport recv fail. ");
            }
        });*/
    }
}
