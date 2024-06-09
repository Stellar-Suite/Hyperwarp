use std::net::SocketAddr;
use std::{path::PathBuf, thread};

use message_io::network::adapter::NetworkAddr;
use stellar_protocol::protocol::{StellarMessage};
use message_io::node;
use message_io::{adapters::unix_socket::{create_null_socketaddr, UnixSocketConnectConfig, UnixSocketListenConfig}, node::{NodeEvent, NodeHandler}};

pub fn test_networking() {
    
    let test_path = PathBuf::from("/tmp/testing_socket");
    let test_path_2 = test_path.clone();


    let a = thread::spawn(move || {
        // server
        let (handler, listener) = message_io::node::split::<StellarMessage>();
        handler.network().listen_with(message_io::network::TransportListen::UnixDatagramSocket(UnixSocketListenConfig::new(test_path)), create_null_socketaddr());
        handler.network().listen(message_io::network::Transport::Udp, "0.0.0.0:1234");
        handler.network().listen(message_io::network::Transport::FramedTcp, "0.0.0.0:1235");
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
        // let mut config = UnixSocketConnectConfig::new(test_path_2);
        // config.flush_on_send = true;
        // let config_enum = message_io::network::TransportConnect::UnixSocketDatagram(config);
        // let (endpoint, addr) = handler.network().connect_with(config_enum, create_null_socketaddr()).expect("client setup failed");
        let (endpoint, addr) = handler.network().connect_with(message_io::network::TransportConnect::UnixSocketDatagram(UnixSocketConnectConfig::new("/tmp/rand_socket".into())), NetworkAddr::Path(test_path_2)).expect("client setup failed");
        let socket_addr = "0.0.0.0:1235".parse::<SocketAddr>().unwrap();
        let addr_enum = NetworkAddr::IP(socket_addr);
        // let (endpoint, addr) = handler.network().connect(message_io::network::Transport::FramedTcp, addr_enum).expect("udp client setup failed");
        for _ in 0..10 {
            handler.network().send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HelloName("Kitty".to_string())));
            handler.network().send(endpoint.clone(), &stellar_protocol::serialize(&StellarMessage::HelloName("Very Large Kitty II".to_string())));
            
            thread::sleep(std::time::Duration::from_millis(133));
        }

        println!("handling other events");

        listener.for_each(move |event| {
            if let NodeEvent::Network(netevent) = event {
                if let message_io::network::NetEvent::Message(endpoint, message) = netevent {
                    match stellar_protocol::deserialize_safe(&message) {
                        Some(message) => {
                            match message {
                                StellarMessage::HelloName(name) => {
                                    println!("Received hello from server: {}", name);
                                },
                                _ => {
                                    println!("Received other type {:#?} from server...", message);
                                }
                            }
                        },
                        _ => {
                            println!("Received invalid message from socket...");
                        }
                    }
                }
            }
        });


    });

    b.join().expect("client thread panicked");
}