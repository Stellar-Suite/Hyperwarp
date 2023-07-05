// specs in protocol.md

pub struct Message {
    type_id: u32,
    payload_size: u32, // max 4gb
}

// type id 4 bytes
// payload_size 4 bytes
pub fn allocate_header_buffer() -> [u8; 8] {
    [0; 8]
}