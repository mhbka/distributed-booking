use std::{collections::{HashMap, VecDeque}, net::SocketAddr};
use uuid::Uuid;

const MAX_LOG_LENGTH: usize = 50;

/// Caches previous requests.
pub struct Log {
    log: VecDeque<(Uuid, Vec<u8>)>
}

impl Log {
    pub fn new() -> Self {
        Self {
            log: VecDeque::new()
        }
    }

    /// Returns the last response's data for a request.
    /// 
    /// Returns `None` if the request wasn't found.
    pub fn check(&mut self, request_id: &Uuid) -> Option<&Vec<u8>> {
        self.log
            .iter()
            .find(|(id, _)| id == request_id)
            .map(|(_, response)| response)
    }

    /// Inserts a response under the request ID.
    /// 
    /// Pops the oldest record if the log has reached capacity.
    pub fn insert(&mut self, request_id: &Uuid, response: &Vec<u8>) {
        if self.log.len() >= MAX_LOG_LENGTH {
            self.log.pop_front();
        }
        self.log.push_back((request_id.clone(), response.clone()));
    }
}

