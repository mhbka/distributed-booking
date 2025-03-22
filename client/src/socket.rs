use std::{io::ErrorKind, net::UdpSocket, thread::sleep, time::Duration};
use shared::{requests::RawRequest, responses::RawResponse, Byteable};

const BUF_SIZE: usize = u16::MAX as usize;
const TIMEOUT_MS: u64 = 50;
const MAX_RETRIES: usize = 5;

/// Wraps a `UdpSocket` and provides (de)serialization and (if enabled) retries.
pub struct SenderReceiver {
    socket: UdpSocket,
    use_reliability: bool
}

impl SenderReceiver {
    /// Create the `SenderReceiver`.
    pub fn new(socket: UdpSocket, use_reliability: bool) -> Self {
        Self {
            socket,
            use_reliability
        }
    }

    /// Send a message and receive a response.
    pub fn send(&mut self, request: RawRequest, addr: String) -> Result<RawResponse, String> {
        let request_bytes = request.to_bytes();
        let mut recv_buffer = vec![0; BUF_SIZE];

        if self.use_reliability {
            self.socket
                .set_read_timeout(Some(Duration::from_millis(TIMEOUT_MS)))
                .expect("Should not have issues setting timeout");
            for retry in 0..MAX_RETRIES {
                self.socket
                    .send_to(&request_bytes, &addr)
                    .map_err(|err| format!("Error while sending request on retry {retry}: {err}"))?;
                match self.socket.recv_from(&mut recv_buffer) {
                    Ok(ok) => {
                        let response = RawResponse::from_bytes(&mut recv_buffer)?;
                        return Ok(response);
                    },
                    Err(err) => {
                        if err.kind() == ErrorKind::TimedOut || err.kind() == ErrorKind::WouldBlock {
                            if retry < MAX_RETRIES-1 {
                                let backoff = Duration::from_millis(TIMEOUT_MS * (retry as u64 + 1));
                                sleep(backoff);
                            }
                        }
                        else {
                            return Err(format!("Got a non-timeout error while receiving message: {err}"));
                        }
                    }
                }
            }
            return Err(format!("Timeout occurred; maxed out at {} retries", MAX_RETRIES));
        }
        else {
            self.socket
                .send_to(&request_bytes, addr)
                .map_err(|err| format!("Error while sending request: {err}"))?;
            match self.socket.recv(&mut recv_buffer) {
                Ok(ok) => {
                    let response = RawResponse::from_bytes(&mut recv_buffer)?;
                    return Ok(response);
                },
                Err(err) => {
                    return Err(format!("Got an error while receiving message: {err}"));
                }
            }
        }
    }   
}