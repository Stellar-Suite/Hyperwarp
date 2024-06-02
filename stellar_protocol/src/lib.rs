use lazy_static::lazy_static;
use bincode;
use crate::protocol::StellarMessage;

pub mod protocol;

// for eventual bincode 2 migration
// lazy_static! {
    
// }

pub fn serialize(msg: StellarMessage) -> Vec<u8> {
    bincode::serialize(&msg).unwrap()
}

pub fn deserialize(data: &[u8]) -> StellarMessage {
    bincode::deserialize(data).unwrap()
}