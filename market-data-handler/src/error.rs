use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("decode error: {0}")]
    Decode(#[from] DecodeError),
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("truncated: need {needed} bytes, have {have}")]
    Truncated { needed: usize, have: usize },

    #[error("invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("invalid UTF-8 in string field")]
    InvalidUtf8,
}

impl DecodeError {
    pub fn truncated(needed: usize, have: usize) -> Self {
        Self::Truncated { needed, have }
    }
}

#[derive(Debug, Error)]
pub enum SeqOrderError {
    #[error("packet sequence out of order: expected {expected}, got {got}")]
    OutOfOrder { expected: u64, got: u64 },
}
