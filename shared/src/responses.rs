use derive::ByteableDerive;
use crate::Byteable;
use uuid::Uuid;

/// Structure of a raw response from the server.
#[derive(ByteableDerive)]
pub struct RawResponse {
    pub request_id: Uuid,
    pub is_error: u8,
    pub length: u16,
    pub data: Vec<u8>,
}

/// Any response from the server (just a string), since the client doesn't any structure anyway.
#[derive(ByteableDerive)]
pub struct MessageResponse {
    message: String
}