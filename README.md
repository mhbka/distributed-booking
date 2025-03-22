## Info
This is the project for NTU's SC4051 Distributed Systems.
The objective is to implement a facilities booking system based on client-server architecture, using UDP sockets.

## Running
You need Rust installed.

### Server
```Powershell
# build it
cd server
cargo build --release
cd ../target/release

# see definition for args
.\server -h

# ...for eg, run with caching + 30% packet drops
.\server -u -p 0.3
```

### Client
```Powershell
# build it
cd client
cargo build --release
cd ../target/release

# see definition for args
.\client -h

# ...for eg, run with retries + 60% packet dupes
.\client -u -d 0.6
```