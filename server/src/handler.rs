use crate::{duplicates::DuplicatesCache, facilities::Facility};

/// Handles messages.
pub struct Handler {
    
}

impl Handler {
    /// Handles a message, returning the response as bytes.
    pub fn handle_message(
        &self, 
        duplicates_cache: &mut DuplicatesCache,
        facilities: &mut Vec<Facility>,
        data: &mut [u8]
    ) -> Vec<u8> 
    {
        match RawRequest
    }
}