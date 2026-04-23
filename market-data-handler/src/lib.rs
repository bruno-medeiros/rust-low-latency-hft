//! Market data handler: full UDP feed pipeline with tick-to-trade latency measurement.
//!
//! # Modules
//!
//! | Module | Role |
//! |---|---|
//! | [`decode`] | ITCH-style length-prefixed message decoder (zero-copy) |
//! | [`message`] | ITCH message types |
//! | [`ingest`] | In-memory datagram reassembly (no sockets) |
//! | [`feed_book`] | Adapter: ITCH message → limit-order-book commands |
//! | [`mold_udp64`] | MoldUDP64-lite packet framing (encode + zero-copy decode) |
//! | [`reorder`] | Sequence-numbered reorder ring with gap detection |
//! | [`rx`] | Batched UDP receiver via `recvmmsg(2)` |
//! | [`outbound`] | Fixed-size outbound order buffer (stack-allocated) |
//! | [`strategy`] | Top-of-book cross-spread quoter stub |
//! | [`latency`] | TSC-backed tick-to-trade latency recorder |
//! | [`pipeline`] | Full hot-loop wiring: RX → reorder → decode → book → strategy → timestamp |

pub mod decode;
pub mod error;
pub mod feed_book;
pub mod mold_udp64;
pub mod ingest;
pub mod latency;
pub mod message;
pub mod outbound;
pub mod pipeline;
pub mod reorder;
pub mod rx;
pub mod strategy;

pub use decode::ItchDecoder;
pub use error::{DecodeError, IngestError};
pub use feed_book::{FeedBookAction, FeedBookAdapter, FeedBookError};
pub use mold_udp64::{
    DecodedPacket, PacketHeader, PacketKind, decode_packet,
    encode_packet,
};
pub use ingest::DatagramIngestor;
pub use latency::LatencyRecorder;
pub use message::{ItchMessage, Side};
pub use outbound::OutboundBuf;
pub use pipeline::{PipelineConfig, PipelineResult, run};
pub use reorder::ReorderRing;
pub use rx::UdpReceiver;
pub use strategy::QuoterState;
