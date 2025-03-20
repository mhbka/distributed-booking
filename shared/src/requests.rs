use uuid::Uuid;
use crate::{time::{Day, Hour, Minute, Time}, Byteable};
use derive::ByteableDerive;

/// Structure of a raw request to the server.
#[derive(ByteableDerive)]
pub struct RawRequest {
    pub request_id: Uuid,
    pub request_type: RequestType,
    pub length: u16,
    pub data: Vec<u8>,
}

/// For requesting facility availability.
#[derive(ByteableDerive)]
pub struct AvailabilityRequest {
    facility_name: String,
    days: Vec<Day>
}

/// For booking a facility.
#[derive(ByteableDerive)]
pub struct BookRequest {
    facility_name: String,
    start_time: Time,
    end_time: Time
}

/// For modifying a booking.
#[derive(ByteableDerive)]
pub struct OffsetBookingRequest {
    booking_id: Uuid,
    offset_hours: Hour,
    offset_min: Minute,
    negative: bool
}

/// For registering a monitor callback.
#[derive(ByteableDerive)]
pub struct MonitorFacilityRequest {
    facility_name: String,
    seconds_to_monitor: u8
}

/// The types of requests to the server.
pub enum RequestType {
    Availability,
    Book,
    Offset,
    Monitor
}

impl Byteable for RequestType {
    /// Deserializes from a single `u8`.
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> where Self: Sized {
        if data.len() >= 1 {
            let discriminant = data.remove(0);
            let val = match discriminant {
                0 => Self::Availability,
                1 => Self::Book,
                2 => Self::Offset,
                3 => Self::Monitor,
                other => Err(format!("Unsupported request type discriminant: {other}"))?
            };
            return Ok(val);
        }
        Err(format!("Not enough bytes (len: {})", data.len()))
    }

    fn to_bytes(self) -> Vec<u8> {
        let val = match self {
            RequestType::Availability => 0,
            RequestType::Book => 1,
            RequestType::Offset => 2,
            RequestType::Monitor => 3,
        };
        vec![val]
    }
}