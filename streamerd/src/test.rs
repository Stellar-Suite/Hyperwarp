use std::{path::PathBuf, thread};

use stellar_protocol::protocol::{StellarMessage};
use message_io::node;
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig, UnixSocketListenConfig}, node::{NodeEvent, NodeHandler}};

pub fn test_networking() {
    
    let test_path = PathBuf::from("/tmp/testing_socket");
    let test_path_2 = test_path.clone();

    let a = thread::spawn(move || {
        // server
        let (handler, listener) = message_io::node::split::<StellarMessage>();
        handler.network().listen_with(message_io::network::TransportListen::UnixSocket(UnixSocketListenConfig::new(test_path)), create_null_socketaddr());
        listener.for_each(move |event| {
            match event {
                NodeEvent::Network(netevent) => {
                    match netevent {
                        message_io::network::NetEvent::Connected(_, _) => {

                        },
                        message_io::network::NetEvent::Accepted(_, _) => {

                        },
                        message_io::network::NetEvent::Message(endpoint, message) => {
                            match stellar_protocol::deserialize_safe(&message) {
                                Some(message) => {
                                    match message {
                                        StellarMessage::HelloName(name) => {
                                            println!("Received hello from client: {}", name);
                                            handler.network().send(endpoint, &stellar_protocol::serialize(&StellarMessage::HelloName("Kitten".to_string())));
                                        },
                                        _ => {
                                            println!("Received invalid message from Hyperwarp socket...");
                                        }
                                    }
                                },
                                None => {
                                    println!("Received invalid message from Hyperwarp socket...");
                                }
                            }
                        },
                        message_io::network::NetEvent::Disconnected(_) => {

                        },
                    };
                },
                _ => {
                    println!("meh other event");
                }
            }
        });
    });

    let b = thread::spawn(move || {
        // client
        thread::sleep(std::time::Duration::from_secs(1)); // Give server time to init
        let (handler, listener) = message_io::node::split::<StellarMessage>();
        let (endpoint, addr) = handler.network().connect_with(message_io::network::TransportConnect::UnixSocket(UnixSocketConnectConfig::new(test_path_2)), create_null_socketaddr()).expect("client setup failed");
        for _ in 0..10 {
            handler.network().send(endpoint, &stellar_protocol::serialize(&StellarMessage::HelloName("Kitty".to_string())));
            thread::sleep(std::time::Duration::from_millis(133));
        }
    });

    b.join().expect("client thread panicked");
}