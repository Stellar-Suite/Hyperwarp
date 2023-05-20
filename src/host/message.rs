// specs in protocol.md

pub struct Message {
    type_id: u32,
    payload_size: u32, // max 4gb
}
