use std::net::{SocketAddr, UdpSocket};
use shared::{requests::RawRequest, Byteable};
use crate::duplicates::DuplicatesCache;

const BUF_SIZE: usize = u16::MAX as usize;

/// Wraps the `UdpSocket` and provides serialization and caching/retry mechanisms.
pub struct SenderReceiver {
    socket: UdpSocket,
    cache: DuplicatesCache
}

impl SenderReceiver {
    pub fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            cache: DuplicatesCache::new()
        }
    }

    /// Attempt to receive a request from the socket.
    /// 
    /// If the request's ID and address is found in cache, the cached response is sent back
    /// and the function waits for the next message instead.
    /// 
    /// Errors if there's an issue receiving the message or decoding it into a `RawRequest`.
    pub fn receive(&mut self) -> Result<(RawRequest, SocketAddr), String> {
        let mut buf = vec![0; BUF_SIZE];
        loop {
            let (size, source_addr) = self.socket
                .recv_from(&mut buf)
                .map_err(|err| format!("Failed to receive UDP data: {err}"))?;

            let request = RawRequest::from_bytes(&mut buf)?;
            
            match self.cache.check(&source_addr, &request.request_id) {
                Some(response) => {
                    tracing::debug!("Found cached response for {}, request ID: {}", source_addr, request.request_id);
                    self.socket.send_to(&response, source_addr);
                    // TODO: retry/timeout
                },
                None => {
                    tracing::debug!("No cached response for {}, request ID: {}; returning with request", source_addr, request.request_id);
                    return Ok((request, source_addr));
                }
            }
        }
    }

    /// Sends the response to the given address.
    pub fn send(&mut self, response: &Vec<u8>, addr: &SocketAddr) -> Result<(), String> {
        tracing::trace!("Sending response with length {} to address {}", response.len(), addr);
        self.socket
            .send_to(response, addr)
            .map(|bytes| ())
            .map_err(|err| format!("Unable to send UDP message: {err}"))
        // TODO: retry/timeout here
    }
}