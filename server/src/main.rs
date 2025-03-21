use std::net::UdpSocket;
use handler::Handler;
use socket::SenderReceiver;
use tracing::Level;

mod facilities;
mod handler;
mod duplicates;
mod socket;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let socket = UdpSocket::bind("127.0.0.1:34524").unwrap();
    let sender_receiver = SenderReceiver::new(socket);
    let mut handler = Handler::new(sender_receiver);

    handler.run();
}
