use shared::{requests::{AvailabilityRequest, BookRequest, MonitorFacilityRequest, OffsetBookingRequest, RawRequest, RequestType}, responses::RawResponse, Byteable};

use crate::{duplicates::DuplicatesCache, facilities::Facility};

/// Handles messages.
pub struct Handler {
    duplicates_cache: DuplicatesCache,
    facilities: Vec<Facility>,

}

impl Handler {
    pub fn new() -> Self {
        Self {
            duplicates_cache: DuplicatesCache::new(),
            facilities: Vec::new()
        }
    }

    /// Handles a message, returning the response as bytes.
    pub fn handle_message(&self, data: &mut Vec<u8>) -> Vec<u8> 
    {
        match RawRequest::from_bytes(data) {
            Ok(req) => {
                match self.duplicates_cache.check(&req.request_id) {
                    Some(res) => res,
                    None => {
                        let result = match req.request_type {
                            RequestType::Availability(req) => {
                                self.handle_availability_request(req)
                            },
                            RequestType::Book(req) => {
                                self.handle_booking_request(req)
                            },
                            RequestType::Offset(req) => {
                                self.handle_offset_request(req)
                            },
                            RequestType::Monitor(req) => {
                                self.handle_monitor_request(req)
                            },
                        };
                        match result {
                            Ok(res) => {
                                let response = RawResponse::from_string(
                                    req.request_id,
                                    0,
                                    res
                                );
                                return response.to_bytes();
                            },
                            Err(res) => {
                                let response: RawResponse = RawResponse::from_string(
                                    req.request_id,
                                    1,
                                    res
                                );
                                return response.to_bytes();
                            }
                        }
                    }
                }
            },
            Err(err) => {

            }
        }
    }

    fn handle_availability_request(&self, req: AvailabilityRequest) -> Result<String, String> {

    }

    fn handle_booking_request(&self, req: BookRequest) -> Result<String, String> {

    }

    fn handle_offset_request(&self, req: OffsetBookingRequest) -> Result<String, String> {

    }

    fn handle_monitor_request(&self, req: MonitorFacilityRequest) -> Result<String, String> {
        
    }
}