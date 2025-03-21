use std::{net::{SocketAddr, UdpSocket}, sync::{Arc, Mutex}, thread::{spawn, JoinHandle}};
use chrono::{DateTime, Duration, TimeZone, Utc};
use shared::{requests::{AvailabilityRequest, BookRequest, MonitorFacilityRequest, OffsetBookingRequest, RawRequest, RequestType}, responses::RawResponse, time::Day, Byteable};
use uuid::Uuid;
use crate::{duplicates::DuplicatesCache, facilities::{Booking, Facility}, socket::SenderReceiver};

/// Handles messages.
pub struct Handler {
    sender_receiver: SenderReceiver,
    duplicates_cache: DuplicatesCache,
    facilities: Vec<Facility>,
    monitoring_addresses: Vec<(SocketAddr, String, DateTime<Utc>)>, // note: String is the facility name, DateTime is the expiry date
}

impl Handler {
    pub fn new(sender_receiver: SenderReceiver) -> Self {
        let duplicates_cache = DuplicatesCache::new();
        let facilities = Vec::new();
        let monitoring_addresses = Vec::new();
        Self {
            sender_receiver,
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

    /// Attempts to add a new booking.
    /// 
    /// If successful, also sends a message to monitoring addresses for updated availability on the booked day.
    fn handle_booking_request(&mut self, req: BookRequest) -> Result<String, String> {
        match self.facilities
            .iter_mut()
            .find(|facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                let booking_day = req.start_time.day;
                let new_booking = Booking::new(req.start_time, req.end_time)?;
                let new_id = facility.add_new_booking(new_booking)?;

                self.send_monitor_message(&req.facility_name, booking_day);

                return Ok(format!("Successfully added new booking with ID: {new_id}"));
            },
            None => {
                return Err("No such facility found".to_string());
            }
        }
    }

    /// Attempts to offset a booking.
    /// 
    /// If successful, also sends a message to monitoring addresses for updated availability on the offsetted day.
    fn handle_offset_request(&mut self, req: OffsetBookingRequest) -> Result<String, String> {
        for facility in &mut self.facilities {
            if let Some(&(ref id, ref booking)) = facility.get_booking_details(&req.booking_id) {
                let booking_day = booking.time().0.day;
                let facility_name = facility.name.clone();
                facility.offset_booking(
                    req.booking_id, 
                    req.offset_hours, 
                    req.offset_min, 
                    req.negative
                )?;
                self.send_monitor_message(&facility_name, booking_day);
                return Ok(format!("Facility {facility_name} successfully offsetted"));
            }
        }
        Err(format!("No booking ID {} found in any facility", req.booking_id))
    }

    /// Attempts to register a monitoring address.
    fn handle_monitor_request(&mut self, req: MonitorFacilityRequest, source_addr: &SocketAddr) -> Result<String, String> {
        match self.facilities
            .iter()
            .find(|&facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                let expiry = Utc::now() + Duration::seconds(req.seconds_to_monitor.into());
                self.monitoring_addresses.push((
                    source_addr.clone(), 
                    req.facility_name.clone(), 
                    expiry
                ));
                return Ok(format!("Successfully registered {source_addr} for monitoring facility {}", req.facility_name));
            },
            None => {
                return Err(format!("No facility {} found", req.facility_name));
            }
        }
    }
    
    /// Send a message to all addresses monitoring the given facility, 
    /// with the availability for the updated day.
    /// 
    /// Also filters out any expired monitoring addresses.
    fn send_monitor_message(
        &mut self, 
        facility_name: &String,
        updated_day: Day
    ) {
        self.monitoring_addresses
            .retain(|(_, _, expiry)| expiry < &Utc::now());

        if let Some(facility) = self.facilities
            .iter()
            .find(|&f| &f.name == facility_name)
        {   
            tracing::debug!("Sending monitor message for facility {facility_name}");

            let availabilities = facility.get_availabilities(updated_day);
            let monitoring_message = format!("-----\n A booking was updated on {updated_day}; new availabilities:\n {availabilities}\n -----");
            let response = RawResponse {
                request_id: Uuid::new_v4(), // doesn't really matter I think
                is_error: false,
                message: monitoring_message
            }.to_bytes();

            self.monitoring_addresses
            .iter()
            .filter(|(_, name, _)| name == facility_name)
            .for_each(|(addr, facility_name, expiry)| {
                self.sender_receiver.send(&response, &addr);
                tracing::debug!("Sent {addr} a monitoring message for facility {facility_name} (expiry: {expiry})");
            });
        }
    }
}