# geo-track -- Toy platform for real-time tracking of geopositional events.

**Note**: Intended as a simple demo rather than an actual product.

The project emulates a web service that tracks current geographic locations of a number of beacons
in real time based on incoming pings. As a real life example, consider tracking fleets of delivery
vehicles, ships, airplanes, etc.

## Building

Run the server locally in dev mode:
```sh
export RUST_LOG=info,server=debug,tower_http=debug
cargo run --bin server
```

Build release binaries:
```sh
cargo build --release
```

## Development

Run unit tests:
```sh
cargo test
```

Format code and run linter:
```sh
cargo fmt
cargo clippy
```

## Features

- [x] Event ingestion
  - [ ] JSON
  - [x] Binary protocol (Bincode)
  - [ ] Persistence
- [ ] Basic event processing
  - [ ] Metrics
    - [ ] Total mileage
    - [ ] Average movement speed
  - [ ] Alerts
    - [ ] Entering/exiting pre-specified zone
- [ ] Random event generation
- [ ] REST API for current state
- [ ] Streaming WebSocket API to export push updates in real time
- [ ] Web UI to view live data on a map
- [x] Structured logging
  - [ ] Export in OpenTelemetry format
