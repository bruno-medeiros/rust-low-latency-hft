//! Outbound order message encoding.
//!
//! `OutboundBuf` is a fixed-size stack buffer representing the bytes that would be
//! sent to an exchange gateway. Zero heap allocation on the hot path.
//!
//! Wire format (all LE):
//!   [msg_type: u8][oid: u64][side: u8][price: u32][qty: u32] = 18 bytes

use limit_order_book::types::Side;
use zerocopy::byteorder::little_endian;
use zerocopy::{Immutable, IntoBytes, KnownLayout, Unaligned};

pub const OUTBOUND_LEN: usize = 18;

const MSG_NEW_ORDER: u8 = 0x01;
const MSG_CANCEL_ORDER: u8 = 0x02;

#[derive(IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C, packed)]
struct NewOrderWire {
    msg_type: u8,
    oid: little_endian::U64,
    side: u8,
    price: little_endian::U32,
    qty: little_endian::U32,
}
const _: () = assert!(core::mem::size_of::<NewOrderWire>() == OUTBOUND_LEN);

#[derive(IntoBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C, packed)]
struct CancelOrderWire {
    msg_type: u8,
    oid: little_endian::U64,
}
const CANCEL_LEN: usize = size_of::<CancelOrderWire>();

/// A fixed-size stack buffer holding one encoded outbound order message.
#[derive(Clone, Copy)]
pub struct OutboundBuf {
    pub bytes: [u8; OUTBOUND_LEN],
    pub len: usize,
}

impl Default for OutboundBuf {
    fn default() -> Self {
        Self { bytes: [0u8; OUTBOUND_LEN], len: 0 }
    }
}

impl OutboundBuf {
    /// Encode a new-order message into `self`. Returns the filled byte slice.
    #[inline]
    pub fn encode_new_order(&mut self, oid: u64, side: Side, price: u32, qty: u32) -> &[u8] {
        let wire = NewOrderWire {
            msg_type: MSG_NEW_ORDER,
            oid: little_endian::U64::new(oid),
            side: match side {
                Side::Buy => 0,
                Side::Sell => 1,
            },
            price: little_endian::U32::new(price),
            qty: little_endian::U32::new(qty),
        };
        self.bytes[..OUTBOUND_LEN].copy_from_slice(wire.as_bytes());
        self.len = OUTBOUND_LEN;
        &self.bytes[..OUTBOUND_LEN]
    }

    /// Encode a cancel-order message into `self`. Returns the filled byte slice.
    #[inline]
    pub fn encode_cancel_order(&mut self, oid: u64) -> &[u8] {
        let wire = CancelOrderWire {
            msg_type: MSG_CANCEL_ORDER,
            oid: little_endian::U64::new(oid),
        };
        self.bytes[..CANCEL_LEN].copy_from_slice(wire.as_bytes());
        self.len = CANCEL_LEN;
        &self.bytes[..CANCEL_LEN]
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_order_roundtrip() {
        let mut buf = OutboundBuf::default();
        let bytes = buf.encode_new_order(99, Side::Buy, 5000, 10);
        assert_eq!(bytes[0], 0x01);
        assert_eq!(u64::from_le_bytes(bytes[1..9].try_into().unwrap()), 99);
        assert_eq!(bytes[9], 0); // Buy
        assert_eq!(u32::from_le_bytes(bytes[10..14].try_into().unwrap()), 5000);
        assert_eq!(u32::from_le_bytes(bytes[14..18].try_into().unwrap()), 10);
        assert_eq!(bytes.len(), OUTBOUND_LEN);
    }

    #[test]
    fn cancel_order_roundtrip() {
        let mut buf = OutboundBuf::default();
        let bytes = buf.encode_cancel_order(42);
        assert_eq!(bytes[0], 0x02);
        assert_eq!(u64::from_le_bytes(bytes[1..9].try_into().unwrap()), 42);
        assert_eq!(bytes.len(), 9);
    }
}
