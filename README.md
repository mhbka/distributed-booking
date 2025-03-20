## Info
This is the course project for NTU's SC4051 Distributed Systems course.
The objective is to implement a distributed facilities booking system based on client-server architecture,
using only UDP sockets.

The project is implemented in Rust.

## Requirements
### Facilities
Each facility has the following properties:
- Name
- Availability (only considering a singular week, ie Mon-Sun, with day/hour/minute granularity) 
- Bookings by clients

Each booking must be exclusive (no overlaps in timing) for that facility.

### Services
The following services are made available by the server:
1. Querying a facility's availability
    - Parameters: Facility name, queried day(s)
    - Returns: Availability of each day; error message if the request is badly formatted, or the facility doesn't exist
2. Booking a facility 
    - Parameters: Start and end time
    - Returns: A unique booking ID; error message if the request is badly formatted, the facility doesn't exist, or there's an overlap
3. Shifting a booking forward/backward in time
    - Parameters: A booking ID and a positive/negative time offset
    - Returns: An acknowledgement; error message if the request is badly formatted, the booking doesn't exist, or there's a new overlap
4. Realtime facility monitoring
    - Parameters: Facility name, monitor interval (ie, period of time to monitor for)
    - Returns: Nothing immediate. The client address + port is recorded, and all changes to the facility are sent to the client up to the monitor
    - Note: For simplicity, a client cannot send requests while it is monitoring a facility.
5. A non-idempotent service (WIP)
6. An idempotent service (WIP)

### Implementation requirements


## Running the project
### Note: WIP
You need Rust installed. Run these on different terminals:

```Powershell
# server
cd server
cargo run --release
```

```Powershell
# client
cd client
cargo run --release
```