use std::net::UdpSocket;
use shared::{requests::{AvailabilityRequest, RawRequest, RequestType}, responses::RawResponse, time::Day, Byteable};
use uuid::Uuid;

const BUF_SIZE: usize = u16::MAX as usize;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34523").unwrap();
    let mut buffer = vec![0; BUF_SIZE];

    let request = RequestType::Availability(
        AvailabilityRequest {
            facility_name: "MR1".into(),
            days: vec![
                Day::Monday, 
                Day::Tuesday,
                Day::Wednesday,
                Day::Thursday,
                Day::Friday
                ]
        }
    );
    let data = RawRequest {
        request_id: Uuid::new_v4(),
        request_type: request
    }.to_bytes();

    let bytes_written = socket.send_to(&data, "127.0.0.1:34524").unwrap();
    println!("Written: {bytes_written}");

    let _ = socket.recv(&mut buffer).unwrap();
    let response = RawResponse::from_bytes(&mut buffer).unwrap();
    println!("Response:\n {}", response.message);
}
