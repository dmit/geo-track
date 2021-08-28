## Where to start

For the tracking server start with the `main` function in `crates/server/bin`.

## Q&A

### Why Axum for HTTP?

For a production-grade project I would go with a more mature library (likely,
[Actix Web](https://actix.rs/)), but since this is more of a showcase, Axum won
out because it's smaller (most importantly, in terms of compilation time), yet
also surprisingly full-featured for such a new library.

### Why CBOR for binary encoding?

Since the goal of this project isn't solving any actual problems, but rather to
showcase a minimized replica of a real-world product would look like in Rust,
the actual encoding doesn't really matter. In reality, `Status` messages would
be arriving to the server in all kinds of (mostly binary, mostly proprietary)
formats. CBOR was chosen because it's binary and because it is ubiquitous.

### Why Sled for persistence?

Because it's simple, it's embedded (so no need for external processes), and it
is extremely promising as a viable production storage engine in the future. At
least as long as you're not veering into petabyte-scale territory.

### Why expose TCP and UDP ingest endpoints?

By default, all endpoints (including HTTP, which is intended to be the
public-facing API) are bound to `localhost`. In reality, the HTTP endpoint would
most likely end up being proxied by Nginx or similar, while the TCP and UDP
ingest endpoints would only be reachable through a VPN tunnel from trusted
partners' networks, which in turn would proxy the status updates from their
corresponding fleets of sensors.

In cases where ingestion directly from sensors is necessary, the TCP/UDP
endpoints would be sitting behind a public-facing load balancer.
