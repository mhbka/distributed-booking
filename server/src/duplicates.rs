use std::{collections::HashMap, net::SocketAddr};

use uuid::Uuid;

/// Caches responses for requests sent to the server.
pub struct DuplicatesCache {
    duplicates: HashMap<Uuid, Vec<u8>>
}

impl DuplicatesCache {
    pub fn new() -> Self {
        Self {
            duplicates: HashMap::new()
        }
    }

    /// Returns the last response for the given request ID.
    pub fn check(&mut self, request_id: Uuid) -> Option<Vec<u8>> {
        self.duplicates.get(&request_id).cloned()
    }

    /// Inserts a response under the request ID.
    pub fn insert_entry(&mut self, request_id: Uuid, response: Vec<u8>) {
        self.duplicates.insert(request_id, response);
    }
}

