use std::net::UdpSocket;

use duplicates::DuplicatesCache;
use handler::Handler;

mod facilities;
mod handler;
mod duplicates;

const MAX_SIZE: usize = 1000;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34524").unwrap();
    
    let handler = Handler {};
    let mut duplicates_cache = DuplicatesCache::new();
    let mut facilities = Vec::new();
    
    let mut recv_buffer = [0; MAX_SIZE];

    loop {
        match socket
            .recv_from(&mut recv_buffer)
            .map_err(|err| format!("Error while receiving: {err}"))
        { 
            Ok((size, source_addr)) => {
                let response = handler.handle_message(
                    &mut duplicates_cache,
                    &mut facilities, 
                    &mut recv_buffer[0..size]
                );
                match socket.send_to(response.as_slice(), source_addr) {
                    Ok(ok) => {

                    },
                    Err(err) => {

                    }
                }
                
            },
            Err(err) => {
                println!("Got an error receiving message: {err}");
            }
        }
    }
}
