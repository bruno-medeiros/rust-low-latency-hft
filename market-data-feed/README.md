# market-data-feed

ITCH-style binary market data feed handler: **ingest** and **decode** raw network payloads into structured events.

## Goals

- **Ingest** — Accept raw datagrams (e.g. UDP multicast payloads or MoldUDP-style framing) and buffer/reassemble for decoding. Minimal copies on the hot path.

- **Decode** — Parse ITCH-style length-prefixed binary messages into a typed event enum. Zero-copy where possible; validated bounds and no panics on malformed input.
