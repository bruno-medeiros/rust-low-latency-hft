//! Decode ITCH-style length-prefixed binary messages.
//!
//! Format: 2-byte big-endian length (payload only) + 1-byte type + payload.
//! Numeric fields are little-endian.

use crate::error::DecodeError;
use crate::message::{ItchMessage, Side};

const MSG_SYSTEM_EVENT: u8 = 0;
const MSG_ADD_ORDER: u8 = 1;
const MSG_ORDER_EXECUTED: u8 = 2;
const MSG_ORDER_CANCELED: u8 = 3;
const MSG_TRADE: u8 = 4;

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
            MSG_TRADE => decode_trade(payload)?,
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
    // oid:8 + side:1 + qty:4 + price:4 + sym_len:2 = 19
    if payload.len() < 19 {
        return Err(DecodeError::truncated(19, payload.len()));
    }
    let oid = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let side = match payload[8] {
        0 => Side::Buy,
        1 => Side::Sell,
        _ => return Err(DecodeError::InvalidMessageType(payload[8])),
    };
    let qty = u32::from_le_bytes(payload[9..13].try_into().unwrap());
    let price = u32::from_le_bytes(payload[13..17].try_into().unwrap());
    let sym_len = u16::from_be_bytes([payload[17], payload[18]]) as usize;
    if payload.len() < 19 + sym_len {
        return Err(DecodeError::truncated(19 + sym_len, payload.len()));
    }
    // REVIEW: allocation here needed?
    let symbol =
        std::str::from_utf8(&payload[19..19 + sym_len]).map_err(|_| DecodeError::InvalidUtf8)?;
    Ok(ItchMessage::AddOrder {
        oid,
        side,
        qty,
        price,
        symbol,
    })
}

fn decode_order_executed(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    if payload.len() < 12 {
        return Err(DecodeError::truncated(12, payload.len()));
    }
    let oid = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let qty = u32::from_le_bytes(payload[8..12].try_into().unwrap());
    Ok(ItchMessage::OrderExecuted { oid, qty })
}

fn decode_order_canceled(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    if payload.len() < 12 {
        return Err(DecodeError::truncated(12, payload.len()));
    }
    let oid = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let qty = u32::from_le_bytes(payload[8..12].try_into().unwrap());
    Ok(ItchMessage::OrderCanceled { oid, qty })
}

fn decode_trade(payload: &[u8]) -> Result<ItchMessage<'_>, DecodeError> {
    if payload.len() < 17 {
        return Err(DecodeError::truncated(17, payload.len()));
    }
    let oid = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    let side = match payload[8] {
        0 => Side::Buy,
        1 => Side::Sell,
        _ => return Err(DecodeError::InvalidMessageType(payload[8])),
    };
    let qty = u32::from_le_bytes(payload[9..13].try_into().unwrap());
    let price = u32::from_le_bytes(payload[13..17].try_into().unwrap());
    Ok(ItchMessage::Trade {
        oid,
        side,
        qty,
        price,
    })
}

/// Encode ITCH-style messages for testing and replay.
pub mod encode {
    use crate::message::Side;

    const MSG_SYSTEM_EVENT: u8 = 0;
    const MSG_ADD_ORDER: u8 = 1;
    const MSG_ORDER_CANCELED: u8 = 3;

    pub fn encode_system_event(text: &str) -> Vec<u8> {
        let text_bytes = text.as_bytes();
        let payload_len = 1 + 2 + text_bytes.len(); // type + text_len + text
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_SYSTEM_EVENT);
        buf.extend_from_slice(&(text_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(text_bytes);
        buf
    }

    pub fn encode_add_order(oid: u64, side: Side, qty: u32, price: u32, symbol: &str) -> Vec<u8> {
        let sym_bytes = symbol.as_bytes();
        let payload_len = 1 + 19 + sym_bytes.len(); // type + fields + symbol
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_ADD_ORDER);
        buf.extend_from_slice(&oid.to_le_bytes());
        buf.push(match side {
            Side::Buy => 0,
            Side::Sell => 1,
        });
        buf.extend_from_slice(&qty.to_le_bytes());
        buf.extend_from_slice(&price.to_le_bytes());
        buf.extend_from_slice(&(sym_bytes.len() as u16).to_be_bytes());
        buf.extend_from_slice(sym_bytes);
        buf
    }

    pub fn encode_order_canceled(oid: u64, qty: u32) -> Vec<u8> {
        let payload_len = 1 + 12; // type + oid:8 + qty:4
        let mut buf = Vec::with_capacity(2 + payload_len);
        buf.extend_from_slice(&(payload_len as u16).to_be_bytes());
        buf.push(MSG_ORDER_CANCELED);
        buf.extend_from_slice(&oid.to_le_bytes());
        buf.extend_from_slice(&qty.to_le_bytes());
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Side;

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
