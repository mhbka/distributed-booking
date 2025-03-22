use std::net::{SocketAddr, UdpSocket};
use rand::{rngs::ThreadRng, Rng};
use shared::{requests::RawRequest, responses::RawResponse, Byteable};
use crate::log::Log;

const BUF_SIZE: usize = u16::MAX as usize;

/// Wraps the `UdpSocket` and provides serialization and logging mechanisms.
pub struct SenderReceiver {
    socket: UdpSocket,
    log: Log,
    rng: ThreadRng,
    use_reliability: bool,
    packet_drop_rate: f64
}

impl SenderReceiver {
    pub fn new(socket: UdpSocket, use_reliability: bool, packet_drop_rate: f64) -> Self {
        Self {
            socket,
            log: Log::new(),
            rng: rand::rng(),
            use_reliability,
            packet_drop_rate
        }
    }

    /// Attempt to receive a request from the socket.
    /// 
    /// If the request's ID and address is found in log, the logd response is sent back
    /// and the function waits for the next message instead.
    /// 
    /// Errors if there's an issue receiving the message or decoding it into a `RawRequest`.
    pub fn receive(&mut self) -> Result<(RawRequest, SocketAddr), String> {
        let mut buf = vec![0; BUF_SIZE];
        loop {
            let (size, source_addr) = self.socket
                .recv_from(&mut buf)
                .map_err(|err| format!("Failed to receive UDP data: {err}"))?;

            let roll = self.rng.random_range(0.0..1.0);
            if roll < self.packet_drop_rate {
                tracing::debug!("Intentionally dropping a packet from {source_addr}...");
                continue;
            }

            let request = RawRequest::from_bytes(&mut buf)?;
            tracing::trace!("Received following message from {source_addr}: {request:?}");
            
            if self.use_reliability {
                match self.log.check(&request.request_id) {
                    Some(response) => {
                        tracing::debug!("Found logged response for {}, request ID: {}; returning cached response", source_addr, request.request_id);
                        if let Err(err) = self.socket.send_to(&response, source_addr) {
                            tracing::warn!("Unable to send UDP message for logged response: {err}");
                        };
                    },
                    None => {
                        tracing::debug!("No logged response for {}, request ID: {}; returning with request", source_addr, request.request_id);
                        return Ok((request, source_addr));
                    }
                }
            }
            else {
                tracing::debug!("Logging turned off; returning with request for {}, request ID: {}", source_addr, request.request_id);
                return Ok((request, source_addr));
            }
        }
    }

    /// Sends the response to the given address.
    /// 
    /// If enabled, also adds the response to the internal log.
    pub fn send(&mut self, response: &RawResponse, addr: &SocketAddr) -> Result<(), String> {
        let response_bytes = response.clone().to_bytes();

        if self.use_reliability {
            let id = response.request_id.clone();
            self.log.insert(&id, &response_bytes);
        }   

        match self.socket
            .send_to(&response_bytes, addr)
            .map(|bytes| ())
            .map_err(|err| format!("Unable to send UDP message: {err}"))
        {
            Ok(ok) => {
                tracing::debug!("Successfully sent following message to {addr}: {response:?}");
                Ok(())
            },
            Err(err) => {
                tracing::warn!("Error while sending message to {addr}: {err}");
                Err(err)
            }
        }
    }
}