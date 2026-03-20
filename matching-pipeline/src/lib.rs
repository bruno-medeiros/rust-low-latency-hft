//! Matching-engine pipeline: parse order flow, queue commands, and drive the LOB.

pub mod command;
mod consumer;
pub mod lobster;
pub mod pipeline;

pub use command::OrderCommand;
pub use limit_order_book::types::{OrderId, Price, Qty, Side};
pub use lobster::{LobsterEventType, LobsterParseError, LobsterParser, LobsterRow};
pub use pipeline::{Pipeline, PipelineConfig, PipelineResult};
