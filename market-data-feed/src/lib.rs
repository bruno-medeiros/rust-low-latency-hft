//! ITCH-style market data feed: ingest and decode (skeleton).

pub mod decode;
pub mod error;
pub mod ingest;
pub mod message;

pub use decode::ItchDecoder;
pub use error::{DecodeError, IngestError};
pub use ingest::DatagramIngestor;
pub use message::{ItchMessage, Side};
