//! Market data handler: full UDP feed pipeline with tick-to-trade latency measurement.

pub mod itch;
pub mod itch_to_book;
pub mod mold_udp64;
pub mod util;
pub mod outbound;
pub mod pipeline;
pub mod reorder;
pub mod udp_receiver;
pub mod strategy;

pub use itch::{DecodeError, IngestError, ItchDecoder};
pub use itch_to_book::{FeedBookAction, ItchToBookAdapter, FeedBookError};
pub use mold_udp64::{
    DecodedPacket, MoldDecodeError, PacketHeader, PacketKind, decode_packet, encode_packet,
    parse_header,
};
pub use util::latency::LatencyRecorder;
pub use outbound::OutboundBuf;
pub use pipeline::{MarketHandlerPipeline, PipelineConfig, PipelineError, PipelineResult};
pub use reorder::{PushError, ReorderStats, ReorderWindowExceeded};
pub use udp_receiver::UdpReceiver;
pub use strategy::QuoterState;
