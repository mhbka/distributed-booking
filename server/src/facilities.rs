use std::ops::{Add, Sub};
use uuid::Uuid;

pub struct Facility {
    pub name: String,
    bookings: Vec<(BookingId, Booking)>
}

impl Facility {
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

    /// Offset the booking by given hours and minutes.
    /// 
    /// Errors if the booking ID doesn't exist, or the offset'd booking overlaps with current ones.
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

        if let Err(err) = self.add_booking_with_id(booking_id, offset_booking) {
            self.add_booking_with_id(booking_id, booking)?;
            return Err(err);
        }
        Ok(())
    } 
}

/// The booking ID, which is just a Uuid (which is just 16 bytes).
pub type BookingId = Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Booking {
    start_time: BookingTime,
    end_time: BookingTime
}

impl Booking {
    /// Create the booking.
    /// 
    /// Errors if `start_time` is equal or after `end_time`.
    pub fn new(start_time: BookingTime, end_time: BookingTime) -> Result<Self, String> {
        if start_time >= end_time {
            return Err(format!("Start time ({start_time:?}) is equal or after end time ({end_time:?})"));
        }
        Ok(
            Self { start_time, end_time }
        )
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

    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct BookingTime {
    pub day: Day,
    pub hour: Hour,
    pub minute: Minute
}

impl BookingTime {
    /// Offsets by the given time. 
    pub fn offset(
        &mut self, 
        hours: Hour,
        minutes: Minute,
        negative: bool
    ) {
        if negative {
            let mut new_minute = self.minute.clone();
            let mut new_hour = self.hour.clone();
            let mut new_day = self.day.clone();
            
            if self.minute.0 < minutes.0 {
                new_minute = Minute((new_minute.0
                     - minutes.0) % 60);
                new_hour = Hour((new_hour.0 + 24 - 1) % 24);
            } else {
                new_minute = Minute((new_minute.0 - minutes.0) % 60);
            }
            if self.hour.0 < hours.0 {
                new_hour = Hour((new_hour.0 + 24 - hours.0) % 24);
                new_day = match new_day {
                    Day::Monday => Day::Sunday,
                    Day::Tuesday => Day::Monday,
                    Day::Wednesday => Day::Tuesday,
                    Day::Thursday => Day::Wednesday,
                    Day::Friday => Day::Thursday,
                    Day::Saturday => Day::Friday,
                    Day::Sunday => Day::Saturday,
                };
            } else {
                new_hour = Hour((new_hour.0 - hours.0) % 24);
            }
            self.minute = new_minute;
            self.hour = new_hour;
            self.day = new_day;
        }
        else {
            let mut new_minute = self.minute.clone();
            let mut new_hour = self.hour.clone();
            let mut new_day = self.day.clone();
            
            new_minute = Minute((new_minute.0 + minutes.0) % 60);
            if self.minute.0 + minutes.0 >= 60 {
                new_hour = Hour((new_hour.0 + 1) % 24);
            }
        
            let total_hours = new_hour.0 + hours.0;
            new_hour = Hour(total_hours % 24);
            if total_hours >= 24 {
                new_day = match new_day {
                    Day::Monday => Day::Tuesday,
                    Day::Tuesday => Day::Wednesday,
                    Day::Wednesday => Day::Thursday,
                    Day::Thursday => Day::Friday,
                    Day::Friday => Day::Saturday,
                    Day::Saturday => Day::Sunday,
                    Day::Sunday => Day::Monday,
                };
            }

            // Update the BookingTime
            self.minute = new_minute;
            self.hour = new_hour;
            self.day = new_day;
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday
}

impl Day {
    pub fn from_u8(val: u8) -> Self {
        let val = val % 7;
        match val {
            0 => Day::Monday,
            1 => Day::Tuesday,
            2 => Day::Wednesday,
            3 => Day::Thursday,
            4 => Day::Friday,
            5 => Day::Saturday,
            6 => Day::Sunday,
            _ => unreachable!(), // This is unreachable because of the modulo operation
        }
    }
}

/// A u8 between 0 and 24.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Hour(u8);

impl Hour {
    pub fn new(hour: u8) -> Result<Self, String> {
        if hour >= 24 {
            return Err(format!("Got an invalid hour value: {hour}"));
        }
        Ok(
            Self(hour)
        )
    }
}

impl Add<Hour> for Hour {
    type Output = Hour;

    fn add(self, rhs: Hour) -> Self::Output {
        Hour((self.0 + rhs.0) % 24)
    }
}

impl Sub<Hour> for Hour {
    type Output = Hour;

    fn sub(self, rhs: Hour) -> Self::Output {
        Hour((self.0 - rhs.0) % 24)
    }
}

/// A u8 between 0 and 60.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Minute(u8);

impl Minute {
    pub fn new(min: u8) -> Result<Self, String> {
        if min >= 60 {
            return Err(format!("Got an invalid minute value: {min}"));
        }
        Ok(
            Self(min)
        )
    }
}

impl Add<Minute> for Minute {
    type Output = Minute;

    fn add(self, rhs: Minute) -> Self::Output {
        Minute((self.0 + rhs.0) % 60)
    }
}

impl Sub<Minute> for Minute {
    type Output = Minute;

    fn sub(self, rhs: Minute) -> Self::Output {
        Minute((self.0 - rhs.0) % 60)
    }
}

