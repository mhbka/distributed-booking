use uuid::Uuid;
use crate::{time::{Day, Hour, Minute, Time}, Byteable};
use derive::ByteableDerive;

/// Structure of a raw request to the server.
#[derive(ByteableDerive, Debug, Clone)]
pub struct RawRequest {
    pub request_id: Uuid,
    pub request_type: RequestType,
}

/// For requesting facility availability.
#[derive(ByteableDerive, Debug, Clone)]
pub struct AvailabilityRequest {
    pub facility_name: String,
    pub days: Vec<Day>
}

/// For booking a facility.
#[derive(ByteableDerive, Debug, Clone)]
pub struct BookRequest {
    pub facility_name: String,
    pub start_time: Time,
    pub end_time: Time
}

/// For modifying a booking.
#[derive(ByteableDerive, Debug, Clone)]
pub struct OffsetBookingRequest {
    pub booking_id: Uuid,
    pub offset_hours: Hour,
    pub offset_min: Minute,
    pub negative: bool
}

/// For cancelling a booking.
#[derive(ByteableDerive, Debug, Clone)]
pub struct CancelBookingRequest {
    pub booking_id: Uuid
}

/// For extending a booking.
#[derive(ByteableDerive, Debug, Clone)]
pub struct ExtendBookingRequest {
    pub booking_id: Uuid,
    pub extend_hours: Hour,
    pub extend_min: Minute
}

/// For registering a monitor callback.
#[derive(ByteableDerive, Debug, Clone)]
pub struct MonitorFacilityRequest {
    pub facility_name: String,
    pub seconds_to_monitor: u8
}

/// The possible requests to the server.
#[derive(Debug, Clone)]
pub enum RequestType {
    Availability(AvailabilityRequest),
    Book(BookRequest),
    Offset(OffsetBookingRequest),
    Monitor(MonitorFacilityRequest),
    Cancel(CancelBookingRequest),
    Extend(ExtendBookingRequest)
}

impl Byteable for RequestType {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> {
        let discriminant = u8::from_bytes(data)?;
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
            4 => {
                let request = CancelBookingRequest::from_bytes(data)?;
                Self::Cancel(request)
            },
            5 => {
                let request = ExtendBookingRequest::from_bytes(data)?;
                Self::Extend(request)
            }
            other => Err(format!("Unsupported request type discriminant: {other}"))?
        };
        Ok(val)
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
            RequestType::Cancel(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 4);
                request_bytes
            },
            RequestType::Extend(request) => {
                let mut request_bytes = request.to_bytes();
                request_bytes.insert(0, 5);
                request_bytes
            },
        }
    }
}