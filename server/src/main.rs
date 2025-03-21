use std::net::UdpSocket;
use handler::Handler;

mod facilities;
mod handler;
mod duplicates;
mod socket;

const INIT_SIZE: usize = 1000;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34524").unwrap();
    let mut handler = Handler::new(socket.try_clone().unwrap());
    let mut recv_buffer = Vec::with_capacity(INIT_SIZE);

    loop {
        match socket
            .recv_from(&mut recv_buffer)
            .map_err(|err| format!("Error while receiving: {err}"))
        { 
            Ok((size, source_addr)) => {
                let response = handler.handle_message(&mut recv_buffer, &source_addr);
                match response {
                    Ok(res) => {
                        match socket.send_to(res.as_slice(), source_addr) {
                            Ok(ok) => {
                                println!("Successfully sent response to {source_addr}\n Data: {res:#?}");
                            },
                            Err(err) => {
                                println!("Error sending response: {err}\n Data: {res:#?}");
                            }
                        }
                    },
                    Err(err) => println!("{err}")
                }
            },
            Err(err) => {
                println!("Error receiving message: {err}");
            }
        }
    }
}
