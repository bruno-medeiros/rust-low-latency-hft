//! Market data handler: full UDP feed pipeline with tick-to-trade latency measurement.

pub mod itch;
pub mod error;
pub mod itch_to_book;
pub mod mold_udp64;
pub mod latency;
pub mod outbound;
pub mod pipeline;
pub mod rx;
pub mod strategy;

pub use itch::ItchDecoder;
pub use error::{DecodeError, IngestError, SeqOrderError};
pub use itch_to_book::{FeedBookAction, ItchToBookAdapter, FeedBookError};
pub use mold_udp64::{
    DecodedPacket, PacketHeader, PacketKind, decode_packet,
    encode_packet,
};
pub use latency::LatencyRecorder;
pub use outbound::OutboundBuf;
pub use pipeline::{MarketHandlerPipeline, PipelineConfig, PipelineResult};
pub use rx::UdpReceiver;
pub use strategy::QuoterState;
