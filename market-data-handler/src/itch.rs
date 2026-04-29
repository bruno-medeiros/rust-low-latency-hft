//! Decode ITCH-style length-prefixed binary messages.
//!
//! Format: 2-byte big-endian length (payload only) + 1-byte type + payload.
//! Numeric fields are little-endian.

use thiserror::Error;
use zerocopy::byteorder::big_endian::U16 as U16Be;
use zerocopy::byteorder::little_endian::{U32, U64};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// ITCH-style decoded message (borrows from input buffer).
#[derive(Debug, PartialEq)]
pub enum ItchMessage<'a> {
    SystemEvent { text: &'a str },
    AddOrder {
        oid: u64,
        side: Side,
        qty: u32,
        price: u32,
        symbol: &'a str,
    },
    OrderExecuted { oid: u64, qty: u32 },
    OrderCanceled { oid: u64, qty: u32 },
}

const MSG_SYSTEM_EVENT: u8 = 0;
const MSG_ADD_ORDER: u8 = 1;
const MSG_ORDER_EXECUTED: u8 = 2;
const MSG_ORDER_CANCELED: u8 = 3;

/// Fixed-size head of an `AddOrder` payload (everything before the variable-length symbol).
#[derive(FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C, packed)]
struct AddOrderFixed {
    oid: U64,
    side: u8,
    qty: U32,
    price: U32,
    sym_len: U16Be,
}
const _: () = assert!(core::mem::size_of::<AddOrderFixed>() == 19);

/// Shared layout for `OrderExecuted` and `OrderCanceled` payloads.
#[derive(FromBytes, IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C, packed)]
struct OidQty {
    oid: U64,
    qty: U32,
}
const _: () = assert!(core::mem::size_of::<OidQty>() == 12);

/// Decoder for ITCH-style messages (length-prefixed, big-endian length).
pub struct ItchDecoder;

impl ItchDecoder {
    pub fn new() -> Self {
        Self
    }

    /// Parse the next length-prefixed ITCH-style message from `buf`.
    /// Returns `(message, consumed_bytes)` if a full message was present, `None` if `buf` is empty or needs more data.
    pub fn pop_message<'a>(
        &mut self,
        buf: &'a [u8],
    ) -> Result<Option<(ItchMessage<'a>, usize)>, DecodeError> {
        if buf.len() < 3 {
            return Ok(None);
        }
        let payload_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let total_len = 2 + payload_len;
        if buf.len() < total_len {
            return Ok(None); // need more data (message may span datagrams)
        }
        let msg_type = buf[2];
        let payload = &buf[3..total_len];
        let msg = match msg_type {
            MSG_SYSTEM_EVENT => decode_system_event(payload)?,
            MSG_ADD_ORDER => decode_add_order(payload)?,
            MSG_ORDER_EXECUTED => decode_order_executed(payload)?,
            MSG_ORDER_CANCELED => decode_order_canceled(payload)?,
            t => return Err(DecodeError::InvalidMessageType(t)),
        };
        Ok(Some((msg, total_len)))
    }
}

impl Default for ItchDecoder {
    fn default() -> Self {
        Self::new()
    }
}

fn decode_system_event(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    if payload.len() < 2 {
        return Err(DecodeError::truncated(2, payload.len()));
    }
    let text_len = u16::from_be_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + text_len {
        return Err(DecodeError::truncated(2 + text_len, payload.len()));
    }
    let text =
        std::str::from_utf8(&payload[2..2 + text_len]).map_err(|_| DecodeError::InvalidUtf8)?;
    Ok(ItchMessage::SystemEvent { text })
}

fn decode_add_order(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    let (fixed, rest) = AddOrderFixed::ref_from_prefix(payload)
        .map_err(|_| DecodeError::truncated(size_of::<AddOrderFixed>(), payload.len()))?;
    let side = match fixed.side {
        0 => Side::Buy,
        1 => Side::Sell,
        b => return Err(DecodeError::InvalidMessageType(b)),
    };
    let sym_len = fixed.sym_len.get() as usize;
    if rest.len() < sym_len {
        return Err(DecodeError::truncated(
            size_of::<AddOrderFixed>() + sym_len,
            payload.len(),
        ));
    }
    let symbol = std::str::from_utf8(&rest[..sym_len]).map_err(|_| DecodeError::InvalidUtf8)?;
    Ok(ItchMessage::AddOrder {
        oid: fixed.oid.get(),
        side,
        qty: fixed.qty.get(),
        price: fixed.price.get(),
        symbol,
    })
}

fn decode_order_executed(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    let (h, _) = OidQty::ref_from_prefix(payload)
        .map_err(|_| DecodeError::truncated(size_of::<OidQty>(), payload.len()))?;
    Ok(ItchMessage::OrderExecuted {
        oid: h.oid.get(),
        qty: h.qty.get(),
    })
}

fn decode_order_canceled(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    let (h, _) = OidQty::ref_from_prefix(payload)
        .map_err(|_| DecodeError::truncated(size_of::<OidQty>(), payload.len()))?;
    Ok(ItchMessage::OrderCanceled {
        oid: h.oid.get(),
        qty: h.qty.get(),
    })
}

/// Encode ITCH-style messages for testing and replay.
pub mod encode {
    use super::{AddOrderFixed, OidQty};
    use crate::itch::Side;
    use zerocopy::IntoBytes;
    use zerocopy::byteorder::big_endian::U16 as U16Be;
    use zerocopy::byteorder::little_endian::{U32, U64};

    const MSG_SYSTEM_EVENT: u8 = 0;
    const MSG_ADD_ORDER: u8 = 1;
    const MSG_ORDER_CANCELED: u8 = 3;

    pub fn encode_system_event(text: &str) -> Vec<u8> {
        let text_bytes = text.as_bytes();
        let payload_len = 1 + 2 + text_bytes.len();
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_SYSTEM_EVENT);
        buf.extend_from_slice(&(text_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(text_bytes);
        buf
    }

    pub fn encode_add_order(oid: u64, side: Side, qty: u32, price: u32, symbol: &str) -> Vec<u8> {
        let sym_bytes = symbol.as_bytes();
        let payload_len = 1 + size_of::<AddOrderFixed>() + sym_bytes.len();
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_ADD_ORDER);
        let fixed = AddOrderFixed {
            oid: U64::new(oid),
            side: match side {
                Side::Buy => 0,
                Side::Sell => 1,
            },
            qty: U32::new(qty),
            price: U32::new(price),
            sym_len: U16Be::new(sym_bytes.len() as u16),
        };
        buf.extend_from_slice(fixed.as_bytes());
        buf.extend_from_slice(sym_bytes);
        buf
    }

    pub fn encode_order_canceled(oid: u64, qty: u32) -> Vec<u8> {
        let payload_len = 1 + size_of::<OidQty>();
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_ORDER_CANCELED);
        let body = OidQty {
            oid: U64::new(oid),
            qty: U32::new(qty),
        };
        buf.extend_from_slice(body.as_bytes());
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_roundtrip_system_event() {
        let encoded = encode::encode_system_event("START");
        let mut decoder = ItchDecoder::new();
        let (msg, consumed) = decoder.pop_message(&encoded).unwrap().unwrap();
        assert_eq!(consumed, encoded.len());
        assert!(matches!(msg, ItchMessage::SystemEvent { text } if text == "START"));
    }

    #[test]
    fn decoder_roundtrip_add_order() {
        let encoded = encode::encode_add_order(1, Side::Buy, 100, 5000, "AAPL");
        let mut decoder = ItchDecoder::new();
        let (msg, consumed) = decoder.pop_message(&encoded).unwrap().unwrap();
        assert_eq!(consumed, encoded.len());
        assert!(matches!(
            msg,
            ItchMessage::AddOrder {
                oid: 1,
                side: Side::Buy,
                qty: 100,
                price: 5000,
                symbol
            } if symbol == "AAPL"
        ));
    }

    #[test]
    fn decoder_roundtrip_order_canceled() {
        let encoded = encode::encode_order_canceled(42, 500);
        let mut decoder = ItchDecoder::new();
        let (msg, consumed) = decoder.pop_message(&encoded).unwrap().unwrap();
        assert_eq!(consumed, encoded.len());
        assert!(matches!(
            msg,
            ItchMessage::OrderCanceled { oid: 42, qty: 500 }
        ));
    }
}
