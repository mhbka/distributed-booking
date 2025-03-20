use std::{fmt::Display, ops::{Add, Sub}};
use derive::ByteableDerive;
use strum::{Display, EnumIter};
use crate::Byteable;

/// Representation of time for a booking.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, ByteableDerive)]
pub struct Time {
    pub day: Day,
    pub hour: Hour,
    pub minute: Minute
}

impl Time {
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

            self.minute = new_minute;
            self.hour = new_hour;
            self.day = new_day;
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}:{}", self.day, self.hour.0, self.minute.0)
    }
}

/// A day of the week.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Display, EnumIter)]
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
    /// Parses a `u8` from 0-6 into a `Day`.
    /// 
    /// Errors if `>=6`.
    pub fn from_u8(val: u8) -> Result<Self, String> {
        let day = match val {
            0 => Day::Monday,
            1 => Day::Tuesday,
            2 => Day::Wednesday,
            3 => Day::Thursday,
            4 => Day::Friday,
            5 => Day::Saturday,
            6 => Day::Sunday,
            _ => return Err(format!("Can only be between 0-6 (got {val})"))
        };
        Ok(day)
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Day::Monday => 0,
            Day::Tuesday => 1,
            Day::Wednesday => 2,
            Day::Thursday => 3,
            Day::Friday => 4,
            Day::Saturday => 5,
            Day::Sunday => 6,
        }
    }
}

impl Byteable for Day {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> where Self: Sized {
        let val = u8::from_bytes(data)?;
        Ok(Day::from_u8(val)?)
    }

    fn to_bytes(self) -> Vec<u8> {
        self.to_u8().to_bytes()
    }
}

/// A u8 between 0 and 24.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
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

impl Byteable for Hour {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> where Self: Sized {
        let val = u8::from_bytes(data)?;
        Ok(Self(val))
    }

    fn to_bytes(self) -> Vec<u8> {
        u8::to_bytes(self.0)
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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq)]
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

impl Byteable for Minute {
    fn from_bytes(data: &mut Vec<u8>) -> Result<Self, String> where Self: Sized {
        let val = u8::from_bytes(data)?;
        Ok(Self(val))
    }

    fn to_bytes(self) -> Vec<u8> {
        u8::to_bytes(self.0)
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