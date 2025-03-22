use derive::ByteableDerive;
use crate::Byteable;
use uuid::Uuid;

/// Structure of a raw response from the server.
#[derive(ByteableDerive, Debug, Clone)]
pub struct RawResponse {
    pub request_id: Uuid,
    pub is_error: bool,
    pub message: String
}