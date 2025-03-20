use std::{collections::HashMap, net::SocketAddr};

use uuid::Uuid;

/// Caches responses for requests sent to the server.
/// 
/// Each `SocketAddr` only holds the latest request ID and response data.
pub struct DuplicatesCache {
    duplicates: HashMap<SocketAddr, (Uuid, Vec<u8>)>
}

impl DuplicatesCache {
    pub fn new() -> Self {
        Self {
            duplicates: HashMap::new()
        }
    }

    /// Returns the last response's data for an address.
    /// 
    /// Returns `None` if the request ID doesn't match.
    pub fn check(&mut self, addr: &SocketAddr, request_id: &Uuid) -> Option<Vec<u8>> {
        match self.duplicates.get(addr) {
            Some((latest_id, data)) => {
                if latest_id == request_id {
                    return Some(data.clone())
                } else {
                    return None;
                }
            },
            None => None
        }
    }

    /// Inserts a response under the request ID.
    pub fn insert_entry(&mut self, addr: &SocketAddr, request_id: &Uuid, response: &Vec<u8>) {
        self.duplicates.insert(addr.clone(), (request_id.clone(), response.clone()));
    }
}

