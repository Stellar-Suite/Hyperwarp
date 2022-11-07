// use std::error::Error;
use std::io::Error;
use std::sync::mpsc::{self, channel, Receiver, Sender};
use std::thread;

use super::message::Message;

pub trait Transport {
    fn send(&mut self, data: &[u8]) -> Result<(), Error>; // block
    fn recv(&mut self, data: &mut [u8]) -> Result<(), Error>; // block
}

pub struct Connection<T: Transport> {
    transport: T,
    // message output queue
    outgoing: (Sender<Message>, Receiver<Message>),
    // message input queue
    ingoing: (Sender<Message>, Receiver<Message>),
}

/*fn test() {
    let a = mpsc::channel();
    let b = mpsc::channel();
}*/

impl<T: Transport> Connection<T> {
    pub fn new(transport: T) -> Self {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();
        let conn = Connection::<T> {
            transport,
            outgoing: (tx1, rx1),
            ingoing: (tx2, rx2),
        };

        conn
    }

    fn start() {
        thread::spawn(move || {});
    }
}
