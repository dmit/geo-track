# geo-track -- Toy platform for real-time tracking of geopositional events.

**Note**: Intended as a simple demo rather than an actual product.

The project emulates a web service that tracks current geographic locations of a number of beacons
in real time based on incoming pings. As a real life example, consider tracking fleets of delivery
vehicles, ships, airplanes, etc.

## Build instructions

Run unit tests:
```sh
cargo test
```

Build release binaries:
```sh
cargo build --release
```

## Features

- [ ] Event ingestion
  - [ ] JSON
  - [ ] binary protocols
- [ ] Random event generation
- [ ] REST API for current state
- [ ] Streaming WebSocket API to export push updates in real time
- [ ] Web UI to view live data on a map
