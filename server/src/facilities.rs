use shared::time::{Day, Hour, Minute, Time};
use uuid::Uuid;

pub struct Facility {
    pub name: String,
    bookings: Vec<(BookingId, Booking)>
}

impl Facility {
    /// Create a new facility.
    pub fn new(name: String) -> Self {
        Self {
            name,
            bookings: Vec::new()
        }
    }

    /// Add a new booking for the facility.
    /// 
    /// Errors if the booking overlaps with current ones.
    pub fn add_new_booking(&mut self, new_booking: Booking) -> Result<BookingId, String> {
        if self.bookings
            .iter()
            .any(|(_, booking)| booking.overlaps(&new_booking))
        {
            return Err(format!("New booking ({new_booking:?}) overlaps with at least 1 current booking"));
        }
        let new_id = Uuid::new_v4();
        self.bookings.push((new_id.clone(), new_booking));
        Ok(new_id)
    }

    /// Add a booking with the given ID.
    /// 
    /// Errors if the ID already exists or there's overlap with current bookings.
    pub fn add_booking_with_id(&mut self, booking_id: BookingId, booking: Booking) -> Result<(), String> {
        if self.bookings
            .iter()
            .any(|(id, _)| id == &booking_id) 
        {
            return Err(format!("Booking {booking_id} already exists"));
        }
        if self.bookings
            .iter()
            .any(|(_, cur_booking)| cur_booking.overlaps(&booking))
        {
            return Err(format!("New booking ({booking:?}) overlaps with at least 1 current booking"));
        }
        self.bookings.push((booking_id, booking));
        Ok(())
    }

    /// Returns the booking details of a given booking ID, if it exists.
    pub fn get_booking_details(&self, booking_id: &BookingId) -> Option<&(Uuid, Booking)> {
        self.bookings
            .iter()
            .find(|(id, _)| id == booking_id)
    }

    /// Remove the booking given by its ID.
    /// 
    /// Errors if the booking ID doesn't exist.
    pub fn remove_booking(&mut self, booking_id: BookingId) -> Result<Booking, String> {
        if let Some(pos) = self.bookings
            .iter()
            .position(|(id, _)| id == &booking_id)
        {
            let booking = self.bookings.remove(pos);
            return Ok(booking.1);
        }
        Err(format!("Booking {booking_id} could not be found"))
    }

    /// Get the available times for the day, as a string.
    pub fn get_availabilities(&self, day: Day) -> String {
        let mut day_bookings: Vec<&Booking> = self.bookings
            .iter()
            .filter_map(|(_, booking)| {
                if booking.start_time.day == day && booking.end_time.day == day {
                    Some(booking)
                } else {
                    None
                }
            })
            .collect();

        day_bookings.sort();
        
        let day_start = Time {
            day: day.clone(),
            hour: Hour::new(0).unwrap(),
            minute: Minute::new(0).unwrap(),
        };
        let day_end = Time {
            day: day.clone(),
            hour: Hour::new(23).unwrap(),
            minute: Minute::new(59).unwrap(),
        };
        
        let mut open_slots = Vec::new();
        let mut current_time = day_start;
        
        for booking in day_bookings {
            if current_time < booking.start_time {
                open_slots.push((current_time, booking.start_time.clone()));
            }
            current_time = booking.end_time.clone();
        }
        if current_time <= day_end {
            open_slots.push((current_time, day_end));
        }

        let mut result = String::new();
        
        for (i, (start, end)) in open_slots.iter().enumerate() {
            result.push_str(&format!("{}. {} - {}", i + 1, start, end));
        }
        
        result
    }

    /// Offset the booking by given hours and minutes.
    /// 
    /// Errors if the booking ID doesn't exist, the offset'd booking overlaps with current ones, 
    /// or the offset pushes the booking into a different day.
    pub fn offset_booking(
        &mut self, 
        booking_id: BookingId, 
        hours: Hour, 
        minutes: Minute, 
        negative: bool
    ) -> Result<(), String> 
    {
        let booking = self.remove_booking(booking_id)?;

        let mut offset_booking = booking.clone();
        offset_booking.offset(hours, minutes, negative);

        if offset_booking.start_time.day != booking.start_time.day
        || offset_booking.end_time.day != booking.start_time.day {
            self.add_booking_with_id(booking_id, booking)?;
            return Err("Offset pushes the booking into a different day; not allowed".to_string());
        }
        else if let Err(err) = self.add_booking_with_id(booking_id, offset_booking) {
            self.add_booking_with_id(booking_id, booking)?;
            return Err(err);
        }
        Ok(())
    } 
}

/// The booking ID, which is just a Uuid (which is just 16 bytes).
pub type BookingId = Uuid;

/// A booking, marked by a start and end time.
/// 
/// As a rule, all bookings must start and end on the same day.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Booking {
    start_time: Time,
    end_time: Time
}

impl Booking {
    /// Create the booking.
    /// 
    /// Errors if `start_time` is equal or after `end_time`.
    pub fn new(start_time: Time, end_time: Time) -> Result<Self, String> {
        if start_time >= end_time {
            return Err(format!("Start time ({start_time:?}) is equal or after end time ({end_time:?})"));
        }
        if start_time.day != end_time.day {
            return Err(format!("Start time day {} must match end time day {}", start_time.day, end_time.day));
        }
        Ok(
            Self { start_time, end_time }
        )
    }

    /// Returns the start and end times of the booking.
    pub fn time(&self) -> (&Time, &Time) {
        (&self.start_time, &self.end_time)
    }

    /// Returns if the 2 bookings overlap.
    pub fn overlaps(&self, other_booking: &Booking) -> bool {
        (self.start_time <= other_booking.start_time && self.end_time >= other_booking.start_time)
        || (other_booking.start_time <= self.start_time && other_booking.end_time >= self.start_time)
    }

    /// Offsets the booking by the given time. 
    pub fn offset(
        &mut self, 
        hours: Hour,
        minutes: Minute,
        negative: bool
    ) {
        self.start_time.offset(hours, minutes, negative);
        self.end_time.offset(hours, minutes, negative);
    }
}
