use std::{net::{SocketAddr, UdpSocket}, sync::{Arc, Mutex}, thread::{spawn, JoinHandle}};
use chrono::{DateTime, Duration, TimeZone, Utc};
use shared::{requests::{AvailabilityRequest, BookRequest, MonitorFacilityRequest, OffsetBookingRequest, RawRequest, RequestType}, responses::RawResponse, Byteable};
use crate::{duplicates::DuplicatesCache, facilities::{Booking, Facility}};

/// Handles messages.
pub struct Handler {
    socket: UdpSocket,
    duplicates_cache: DuplicatesCache,
    facilities: Vec<Facility>,
    monitoring_addresses: Vec<(SocketAddr, String, DateTime<Utc>)>, // note: String is the facility name, DateTime is the expiry date
}

impl Handler {
    pub fn new(socket: UdpSocket) -> Self {
        let duplicates_cache = DuplicatesCache::new();
        let facilities = Vec::new();
        let monitoring_addresses = Vec::new();
        Self {
            socket,
            duplicates_cache,
            facilities,
            monitoring_addresses,
        }
    }

    /// Handles a message, returning the response as bytes.
    pub fn handle_message(&mut self, data: &mut Vec<u8>, source_addr: &SocketAddr) -> Result<Vec<u8>, String> 
    {
        match RawRequest::from_bytes(data) {
            Ok(req) => {
                match self.duplicates_cache.check(source_addr, &req.request_id) {
                    Some(res) => Ok(res),
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
                                self.handle_monitor_request(req, source_addr)
                            },
                        };
                        match result {
                            Ok(res) => {
                                let response = RawResponse {
                                    request_id: req.request_id,
                                    is_error: false,
                                    message: res
                                };
                                return Ok(response.to_bytes());
                            },
                            Err(res) => {
                                let response = RawResponse {
                                    request_id: req.request_id,
                                    is_error: true,
                                    message: res
                                };
                                return Ok(response.to_bytes());
                            }
                        }
                    }
                }
            },
            Err(err) => {
                return Err(format!("Error deserializing message: {err}"));
            }
        }
    }

    fn handle_availability_request(&self, req: AvailabilityRequest) -> Result<String, String> {
        match self.facilities
            .iter()
            .find(|&facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                let availabilities = req.days
                    .into_iter()
                    .map(|day| format!("{day}:\n {}\n", facility.get_availabilities(day)))
                    .collect();
                return Ok(availabilities);
            },
            None => {
                return Err("No such facility found".to_string());
            }
        }
    }

    fn handle_booking_request(&mut self, req: BookRequest) -> Result<String, String> {
        match self.facilities
            .iter_mut()
            .find(|facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                let new_booking = Booking::new(req.start_time, req.end_time)?;
                let new_id = facility.add_new_booking(new_booking)?;
                return Ok(format!("Successfully added new booking with ID: {new_id}"));
            },
            None => {
                return Err("No such facility found".to_string());
            }
        }
    }

    fn handle_offset_request(&mut self, req: OffsetBookingRequest) -> Result<String, String> {
        for facility in &mut self.facilities {
            if facility.check_booking_id(&req.booking_id) {
                facility.offset_booking(
                    req.booking_id, 
                    req.offset_hours, 
                    req.offset_min, 
                    req.negative
                )?;
            }
        }
        Err(format!("No booking ID {} found in any facility", req.booking_id))
    }

    fn handle_monitor_request(&mut self, req: MonitorFacilityRequest, source_addr: &SocketAddr) -> Result<String, String> {
        match self.facilities
            .iter()
            .find(|&facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                let expiry = Utc::now() + Duration::seconds(req.seconds_to_monitor);
                self.monitoring_addresses.push((source_addr.clone(), req.facility_name, expiry));
                // TODO: send a monitor message
            },
            None => {

            }
        }
    }

    fn send_monitor_message()
}