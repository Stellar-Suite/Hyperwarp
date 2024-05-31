// specs in protocol.md

use serde::{Deserialize, Serialize};

pub struct Message {
    type_id: u32,
    payload_size: u32, // max 4gb
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    RenderedFrame {
        width: u32,
        height: u32,
    },
    ForceResize {
        width: u32,
        height: u32,
    },
    EnableFlag {
        flag: u32
    },
    DisableFlag {
        flag: u32
    },
    SetId {
        id: u32
    }
}

pub const BROADCAST_TYPE: u32 = 0;
pub const CONTROL_TYPE: u32 = 1;

// type id 4 bytes
// payload_size 4 bytes
pub fn allocate_header_buffer() -> [u8; 8] {
    [0; 8]
}