//! Lock-free SPSC and MPSC queue crate.

pub mod mpsc;
pub mod spsc;

pub use mpsc::{MpscConsumer, MpscProducer, MpscQueue};
pub use spsc::{SpscConsumer, SpscProducer, SpscQueue};
