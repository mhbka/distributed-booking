use std::io::{self, Write};
use std::net::UdpSocket;
use std::str::FromStr;
use clap::{command, Parser};
use shared::requests::{AvailabilityRequest, BookRequest, CancelBookingRequest, ExtendBookingRequest, MonitorFacilityRequest, OffsetBookingRequest, RawRequest, RequestType};
use shared::time::{Day, Hour, Minute, Time};
use socket::SenderReceiver;
use uuid::Uuid;

mod socket;

/// The client for the project.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The address to bind to
    #[arg(short, long, default_value_t = String::from("0.0.0.0:34523"))]
    addr: String,
    /// The address of the server
    #[arg(short, long, default_value_t = String::from("0.0.0.0:34524"))]
    server_addr: String,
    /// Whether to enable retries (DEFAULTS TO FALSE)
    #[arg(short, long)]
    use_reliability: bool,
    /// The proportion of packets to duplicate (only if retries are enabled)
    #[arg(short, long, default_value_t = 0.0)]
    duplicate_packet_rate: f64
}

fn main() {
    let args = Args::parse();
    
    println!("======================");
    println!("Arguments: {args:#?}");
    println!("======================");

    let socket = UdpSocket::bind(args.addr).unwrap();
    let mut sender_receiver = SenderReceiver::new(socket, args.use_reliability, args.duplicate_packet_rate);

    loop {  
        let request = get_user_request();
        println!("Request created: {:?}", request);

        let seconds_to_monitor = if let RequestType::Monitor(req) = &request.request_type {
            Some(req.seconds_to_monitor)
        } else {
            None
        };

        match sender_receiver.send(request, &args.server_addr) {
            Ok(response) => {
                println!("--- Response ---");
                println!("{}", response.message);
                println!("----------------");
            }
            Err(err) => {
                println!("---- Error ----");
                println!("{err}");
                println!("---------------");
            }
        }

        if let Some(seconds) = seconds_to_monitor {
            sender_receiver.monitor(&args.server_addr, seconds);
        }
    }
    
    
}

fn get_user_request() -> RawRequest {
    println!("Facility Booking System");
    println!("======================");
    
    let request_id = Uuid::new_v4();
    let request_type = get_request_type();

    RawRequest {
        request_id,
        request_type,
    }
}

fn get_request_type() -> RequestType {
    println!("Please select a request type:");
    println!("1. Check facility availability");
    println!("2. Book a facility");
    println!("3. Offset an existing booking");
    println!("4. Monitor a facility");
    println!("5. Cancel a booking");
    println!("6. Extend a booking");
    
    let choice = get_input_with_prompt("Enter your choice (1-6): ");
    
    match choice.trim() {
        "1" => RequestType::Availability(get_availability_request()),
        "2" => RequestType::Book(get_book_request()),
        "3" => RequestType::Offset(get_offset_booking_request()),
        "4" => RequestType::Monitor(get_monitor_facility_request()),
        "5" => RequestType::Cancel(get_cancel_booking_request()),
        "6" => RequestType::Extend(get_extend_booking_request()),
        _ => {
            println!("Invalid choice. Please try again.");
            get_request_type()
        }
    }
}

fn get_availability_request() -> AvailabilityRequest {
    println!("\n-- Checking Facility Availability --");
    
    let facility_name = get_input_with_prompt("Enter facility name: ");
    
    println!("Enter days to check (comma-separated, e.g., Mon,Tue,Wed):");
    let days_input = get_input_with_prompt("Days: ");
    
    let days = days_input
        .split(',')
        .map(|day| day.trim())
        .filter(|day| !day.is_empty())
        .map(|day| Day::from_str(day).unwrap_or_else(|_| {
            println!("Warning: Invalid day '{}', defaulting to Monday", day);
            Day::Monday
        }))
        .collect();
    
    AvailabilityRequest {
        facility_name,
        days,
    }
}

fn get_book_request() -> BookRequest {
    println!("\n-- Booking a Facility --");
    
    let facility_name = get_input_with_prompt("Enter facility name: ");
    
    println!("- Start time -");
    let start_time = get_time_input();
    println!("- End time -");
    let end_time = get_time_input();
    
    BookRequest {
        facility_name,
        start_time,
        end_time,
    }
}

fn get_offset_booking_request() -> OffsetBookingRequest {
    println!("\n-- Modifying a Booking --");
    
    let booking_id = get_uuid_input("Enter booking ID: ");
    
    println!("Enter time offset:");
    let offset_hours = get_number_input::<Hour>("Hours: ");
    let offset_min = get_number_input::<Minute>("Minutes: ");
    
    let negative_input = get_input_with_prompt("Move booking earlier? (y/n): ");
    let negative = negative_input.trim().to_lowercase() == "y";
    
    OffsetBookingRequest {
        booking_id,
        offset_hours,
        offset_min,
        negative,
    }
}

fn get_monitor_facility_request() -> MonitorFacilityRequest {
    println!("\n-- Monitoring a Facility --");
    
    let facility_name = get_input_with_prompt("Enter facility name: ");
    let seconds = get_number_input::<u8>("Enter seconds to monitor (max 255): ");
    
    MonitorFacilityRequest {
        facility_name,
        seconds_to_monitor: seconds,
    }
}

fn get_cancel_booking_request() -> CancelBookingRequest {
    println!("\n-- Cancelling a Booking --");
    
    let booking_id = get_uuid_input("Enter booking ID to cancel: ");
    
    CancelBookingRequest {
        booking_id,
    }
}

fn get_extend_booking_request() -> ExtendBookingRequest {
    println!("\n-- Extending a Booking --");
    
    let booking_id = get_uuid_input("Enter booking ID: ");
    
    println!("Enter extension time:");
    let extend_hours = get_number_input::<Hour>("Hours: ");
    let extend_min = get_number_input::<Minute>("Minutes: ");
    
    ExtendBookingRequest {
        booking_id,
        extend_hours,
        extend_min,
    }
}

// Helper functions

fn get_input_with_prompt(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn get_number_input<T: FromStr>(prompt: &str) -> T {
    loop {
        let input = get_input_with_prompt(prompt);
        match input.parse::<T>() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Please enter a valid number."),
        }
    }
}

fn get_uuid_input(prompt: &str) -> Uuid {
    loop {
        let input = get_input_with_prompt(prompt);
        match Uuid::parse_str(&input) {
            Ok(uuid) => return uuid,
            Err(_) => println!("Invalid UUID format. Please try again."),
        }
    }
}

fn get_time_input() -> Time {
    // First get the day
    println!("Enter day (e.g., Mon, Tue, Wed):");
    let day = loop {
        let day_str = get_input_with_prompt("Day: ");
        match Day::from_str(&day_str) {
            Ok(day) => break day,
            Err(_) => println!("Invalid day format. Please try again."),
        }
    };
    
    // Then get the time (hours and minutes)
    loop {
        let time_str = get_input_with_prompt("Time (HH:MM): ");
        let parts: Vec<&str> = time_str.split(':').collect();
        
        if parts.len() == 2 {
            if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) {
                let hour = Hour::new(hours);
                let minute = Minute::new(minutes);
                if let Ok(hour) = hour {
                    if let Ok(minute) = minute {
                        return Time { 
                            day, 
                            hour,
                            minute 
                        };
                    }
                }
            }
        }
        
        println!("Invalid time format. Please use HH:MM format.");
    }
}