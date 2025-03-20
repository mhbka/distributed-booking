use uuid::Uuid;
use crate::{time::{Day, Hour, Minute, Time}, Byteable};
use derive::ByteableDerive;

/// Structure of a raw request to the server.
#[derive(ByteableDerive)]
pub struct RawRequest {
    pub request_id: Uuid,
    pub request_type: RequestType,
}

/// For requesting facility availability.
#[derive(ByteableDerive)]
pub struct AvailabilityRequest {
    pub facility_name: String,
    pub days: Vec<Day>
}

/// For booking a facility.
#[derive(ByteableDerive)]
pub struct BookRequest {
    pub facility_name: String,
    pub start_time: Time,
    pub end_time: Time
}

/// For modifying a booking.
#[derive(ByteableDerive)]
pub struct OffsetBookingRequest {
    pub booking_id: Uuid,
    pub offset_hours: Hour,
    pub offset_min: Minute,
    pub negative: bool
}

/// For registering a monitor callback.
#[derive(ByteableDerive)]
pub struct MonitorFacilityRequest {
    pub facility_name: String,
    pub seconds_to_monitor: u8
}

/// The possible requests to the server.
pub enum RequestType {
    Availability(AvailabilityRequest),
    Book(BookRequest),
    Offset(OffsetBookingRequest),
    Monitor(MonitorFacilityRequest)
}

impl Byteable for RequestType {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        if data.len() >= 1 {
            let discriminant = data.remove(0);
            let val = match discriminant {
                0 => {
                    let request = AvailabilityRequest::from_bytes(data)?;
                    Self::Availability(request)
                },
                1 => {
                    let request = BookRequest::from_bytes(data)?;
                    Self::Book(request)
                },
                2 => {
                    let request = OffsetBookingRequest::from_bytes(data)?;
                    Self::Offset(request)
                },
                3 => {
                    let request = MonitorFacilityRequest::from_bytes(data)?;
                    Self::Monitor(request)
                },
                other => Err(format!("Unsupported request type discriminant: {other}"))?
            };
            return Ok(val);
        }
        Err(format!("Not enough bytes (len: {})", data.len()))
    }

    fn to_bytes(self) -> Vec<u8> {
        match self {
            RequestType::Availability(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 0);
                request_bytes
            },
            RequestType::Book(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 1);
                request_bytes
            },
            RequestType::Offset(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 2);
                request_bytes
            },
            RequestType::Monitor(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 3);
                request_bytes
            },
        }
    }
}