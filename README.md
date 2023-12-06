# geo-track — Toy platform for real-time tracking of geopositional events.

**Note**: Intended as a simple demo rather than an actual product.

The project emulates a web service that tracks current geographic locations of a
number of beacons in real time based on incoming pings. As a real life example,
consider tracking fleets of delivery vehicles, ships, airplanes, etc.

## Building

Run the server locally in dev mode:

```console
export RUST_LOG=info,server=debug,tower_http=debug
cargo run --bin server --features bin,sled
```

Build release binaries:

```console
cargo build --release --features bin,sled
```

## Development

The project is being developed and tested using the latest stable version of the
Rust compiler.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for information about high-level
technical decisions as well as where to start if you're looking to learn more
about the code base.

Run unit tests:

```console
cargo test
```

or using [cargo-nextest](https://nexte.st):

```console
cargo nextest run
```

Format code and run linter:

```console
cargo fmt
cargo clippy --features bin,sled
```

## Feature roadmap

- [x] Event ingestion
  - [ ] JSON over HTTP
  - [x] Binary over TCP and UDP (CBOR)
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
