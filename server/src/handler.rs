use std::net::SocketAddr;
use chrono::{DateTime, Duration, Utc};
use shared::{requests::{AvailabilityRequest, BookRequest, MonitorFacilityRequest, OffsetBookingRequest, RawRequest, RequestType}, responses::RawResponse, time::Day, Byteable};
use uuid::Uuid;
use crate::{facilities::{Booking, Facility}, socket::SenderReceiver};

/// Handles messages.
pub struct Handler {
    sender_receiver: SenderReceiver,
    facilities: Vec<Facility>,
    monitoring_addresses: Vec<(SocketAddr, String, DateTime<Utc>)>, // note: String is the facility name, DateTime is the expiry date
}

impl Handler {
    /// Instantiate the handler.
    pub fn new(sender_receiver: SenderReceiver) -> Self {
        let facilities = vec![ // initial facilities
            Facility::new("MR1".into()),
            Facility::new("MR2".into()),
            Facility::new("MR3".into()),
            Facility::new("MR4".into()),
            Facility::new("MR5".into()),
        ];
        let monitoring_addresses = Vec::new();
        Self {
            sender_receiver,
            facilities,
            monitoring_addresses,
        }
    }

    /// Infinitely receives and handles messages.
    pub fn run(&mut self) {
        loop {
            match self.sender_receiver.receive() { 
                Ok((req, source_addr)) => {
                    let response = self.handle_message(req, &source_addr);
                    match response {
                        Ok(res) => {
                            match self.sender_receiver.send(&res, &source_addr) {
                                Ok(ok) => {
                                    tracing::debug!("Successfully sent response to {} (length: {})", source_addr, res.len());
                                },
                                Err(err) => {
                                    tracing::warn!("Error sending response to {}: {} (length: {})", source_addr, err, res.len());
                                }
                            }
                        },
                        Err(err) => tracing::warn!("Error while handling message: {err}")
                    }
                },
                Err(err) => tracing::warn!("Error receiving message: {err}")
            }
        }
    }

    /// Handles a message, returning the response as bytes.
    pub fn handle_message(&mut self, req: RawRequest, source_addr: &SocketAddr) -> Result<Vec<u8>, String> 
    {
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

    fn handle_availability_request(&self, mut req: AvailabilityRequest) -> Result<String, String> {
        match self.facilities
            .iter()
            .find(|&facility| facility.name == req.facility_name)
        {
            Some(facility) => {
                tracing::trace!("check this: {:?}", req.days);
                req.days.sort();
                req.days.dedup(); // in case >1 of the same day
                let availabilities = req.days
                    .into_iter()
                    .map(|day| format!("-----\n {}\n -----\n", facility.get_availabilities(day)))
                    .collect();
                tracing::trace!("{availabilities}");
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