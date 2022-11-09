// use std::error::Error;
use std::io::Error;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;

use super::message::Message;

pub trait Transport {
    fn send(&mut self, data: &[u8]) -> Result<(), Error>; // block
    fn recv(&mut self, data: &mut [u8]) -> Result<(), Error>; // block
    fn init(&mut self) -> Result<(), Error> {
        Ok(())
    } // this should block until we establish a connection
}

pub struct Connection {
    pub transport: Box<dyn Transport + Send + Sync>,
    // message output queue
    outgoing: (Sender<Message>, Receiver<Message>),
    // message input queue
    ingoing: (Sender<Message>, Receiver<Message>),
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
            transport: Box::new(transport),
            outgoing: mpsc::channel(), // (tx1, rx1),
            ingoing: mpsc::channel(),  // (tx2, rx2),
        };

        conn
    }

    pub fn start(&mut self) {
        let transport = &mut self.transport;
        transport.init().expect("Transport initalization failure. ");
        /*thread::spawn(move || {
            transport.init().expect("Transport initalization fail. ");
            // read/write queue here
        });*/
    }
}
