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

impl RawResponse {
    /// Build a simple response from just a string.
    pub fn from_string(request_id: Uuid, is_error: u8, string: String) -> Self {
        let length = string.len() as u16;
        let data = string.bytes().collect();
        Self {
            request_id,
            is_error,
            length,
            data    
        }
    }
}