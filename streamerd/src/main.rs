use clap::Parser;

use crate::streamerd::{Streamer, StreamerConfig};
pub mod streamerd;
pub mod test;

fn main() {
    println!("Starting streamer daemon v{}",env!("CARGO_PKG_VERSION"));
    let config = StreamerConfig::parse();
    println!("Loaded config: {:?}", config);
    let mut streamerd = Streamer::new(config);
    streamerd.run();
}