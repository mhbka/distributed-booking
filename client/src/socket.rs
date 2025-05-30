use std::{error::Error, io::ErrorKind, net::UdpSocket, thread::sleep, time::{Duration, SystemTime}};
use rand::{rngs::ThreadRng, Rng};
use shared::{requests::RawRequest, responses::RawResponse, Byteable};

const BUF_SIZE: usize = u16::MAX as usize;
const TIMEOUT_MS: u64 = 500;
const MAX_RETRIES: usize = 10;

/// Wraps a `UdpSocket` and provides (de)serialization and (if enabled) retries.
pub struct SenderReceiver {
    socket: UdpSocket,
    rng: ThreadRng,
    use_reliability: bool,
    duplicate_packet_rate: f64
}

impl SenderReceiver {
    /// Create the `SenderReceiver`.
    pub fn new(socket: UdpSocket, use_reliability: bool, duplicate_packet_rate: f64) -> Self {
        socket
            .set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
            .expect("Should not have issues setting timeout");
        Self {
            socket,
            rng: rand::rng(),
            use_reliability,
            duplicate_packet_rate
        }
    }

    /// Send a message and receive a response.
    pub fn send(&mut self, request: RawRequest, addr: &String) -> Result<RawResponse, String> {
        let request_id = request.request_id;
        let request_bytes = request.to_bytes();
        let mut recv_buffer = vec![0; BUF_SIZE];

        if self.use_reliability {
            for retry in 0..MAX_RETRIES {
                self.socket
                    .send_to(&request_bytes, &addr)
                    .map_err(|err| format!("Error while sending request on retry {retry}: {err} (source: {:?})", err.source()))?;

                let roll = self.rng.random_range(0.0..1.0);
                if roll < self.duplicate_packet_rate {
                    println!("Intentionally duplicating packet...");
                    continue;
                }
                
                loop {
                    match self.socket.recv_from(&mut recv_buffer) {
                        Ok(ok) => {
                            let response = RawResponse::from_bytes(&mut recv_buffer)?;

                            if response.request_id != request_id {
                                println!("Response ID {} doesn't match request ID {}; continuing...", response.request_id, request_id);
                                continue;
                            }

                            return Ok(response);
                        },
                        Err(err) => {
                            if err.kind() == ErrorKind::TimedOut || err.kind() == ErrorKind::WouldBlock {
                                if retry < MAX_RETRIES-1 {
                                    let backoff_ms = TIMEOUT_MS * (retry as u64 + 1);
                                    let backoff = Duration::from_millis(backoff_ms);
                                    println!("Attempt {}: Failed to send packet; waiting {}ms before retrying", retry+1, backoff_ms);
                                    sleep(backoff);
                                }
                            }
                            else {
                                return Err(format!("Got a non-timeout error while receiving message: {err} (source: {:?})", err.source()));
                            }
                        }
                    }
                }
            }
            return Err(format!("Timeout occurred; maxed out at {} retries", MAX_RETRIES));
        }
        else {
            self.socket
                .send_to(&request_bytes, addr)
                .map_err(|err| format!("Error while sending request: {err} (source: {:?})", err.source()))?;
            match self.socket.recv(&mut recv_buffer) {
                Ok(ok) => {
                    let response = RawResponse::from_bytes(&mut recv_buffer)?;
                    return Ok(response);
                },
                Err(err) => {
                    return Err(format!("Got an error while receiving message: {err}, (source: {:?})", err.source()));
                }
            }
        }
    }

    /// Monitors messages from `addr` and prints them for the specified number of seconds.
    /// 
    /// Call this after sending a monitor request.
    pub fn monitor(&mut self, addr: &String, seconds: u8) {
        self.socket // don't need to timeout so often while monitoring
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("Should not have issues setting timeout");

        let expiry_time = SystemTime::now()
            .checked_add(Duration::from_secs(seconds as u64))
            .expect("Should be valid");
        let mut recv_buffer = vec![0; BUF_SIZE];

        println!("------");
        println!("Now monitoring address {addr}...");

        while SystemTime::now() < expiry_time {
            match self.socket.recv_from(&mut recv_buffer) {
                Ok((size, source_addr)) => {
                    match RawResponse::from_bytes(&mut recv_buffer) {
                        Ok(response) => {
                            println!("------");
                            if &source_addr.to_string() != addr {
                                println!("NOTE: Following message came from an unexpected address ({source_addr})");
                            }
                            println!("{}", response.message);
                        },
                        Err(err) => {
                            println!("------");
                            println!("Error parsing message: {err}");
                        }
                    }
                },
                Err(err) => {
                    if err.kind() != ErrorKind::TimedOut {
                        println!("------");
                        println!("Error receiving message: {err} (kind: {})", err.kind());
                    }
                }
            }
        }

        self.socket // after monitoring, we set back to the normal timeout value
            .set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
            .expect("Should not have issues setting timeout");

        println!("------");
        println!("Ending monitoring...");
        println!("------");
    }
}