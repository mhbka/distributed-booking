use std::net::UdpSocket;

use duplicates::DuplicatesCache;
use handler::Handler;

mod facilities;
mod handler;
mod duplicates;

const INIT_SIZE: usize = 1000;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34524").unwrap();
    
    let handler = Handler::new();
    
    let mut recv_buffer = Vec::with_capacity(INIT_SIZE);

    loop {
        match socket
            .recv_from(&mut recv_buffer)
            .map_err(|err| format!("Error while receiving: {err}"))
        { 
            Ok((size, source_addr)) => {
                let response = handler.handle_message(&mut recv_buffer);
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
