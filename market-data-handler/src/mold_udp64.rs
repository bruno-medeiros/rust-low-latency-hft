//! MoldUDP64 framing — closely following the Nasdaq MoldUDP64 specification v0.00.09.
//!
//! ## Wire format
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │ session    : [u8; 10]  ASCII, right-padded with spaces   │
//! │ seq        : u64 BE    first message's sequence number   │
//! │ msg_count  : u16 BE    0 = heartbeat, 0xFFFF = end-of-session │
//! ├──────────────────────────────────────────────────────────┤
//! │ msg_len    : u16 BE  ┐                                   │
//! │ msg_bytes  : [u8]    │ × msg_count  (absent for heartbeat / end-of-session)
//! └──────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Spec conformance
//!
//! | Feature              | Nasdaq spec               | This implementation             |
//! |----------------------|---------------------------|---------------------------------|
//! | Session identifier   | 10-byte ASCII             | ✓ `PacketHeader::session`       |
//! | Byte order           | Big-endian                | ✓ big-endian throughout         |
//! | Heartbeat            | `msg_count = 0`           | ✓ `PacketKind::Heartbeat`       |
//! | End-of-session       | `msg_count = 0xFFFF`      | ✓ `PacketKind::EndOfSession`    |
//! | Gap recovery channel | TCP retransmit request    | ✗ gap detection only (out of scope) |
//!
//! The decoder yields zero-copy slices into the caller's receive buffer.
//! The encoder allocates; it is intended for the replay sender, not the hot path.

use thiserror::Error;

/// Failure to decode MoldUDP64 wire bytes (e.g. truncated header).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoldDecodeError {
    #[error("MoldUDP64 header truncated: need {needed} bytes, have {have}")]
    HeaderTruncated { needed: usize, have: usize },
}

/// Size of the session identifier in bytes.
pub const SESSION_LEN: usize = 10;

/// Total size of the packet header in bytes: session(10) + seq(8) + msg_count(2).
pub const HEADER_LEN: usize = SESSION_LEN + 8 + 2;

/// Size of the per-message length prefix in bytes.
pub const MSG_LEN_PREFIX: usize = 2;

/// `msg_count` value that designates a heartbeat packet (carries no messages).
pub const HEARTBEAT_MSG_COUNT: u16 = 0;

/// `msg_count` value that designates an end-of-session packet (carries no messages).
pub const END_OF_SESSION_MSG_COUNT: u16 = 0xFFFF;

/// Decoded packet header, present in every MoldUDP64 packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketHeader {
    /// 10-byte ASCII session identifier, right-padded with spaces.
    pub session: [u8; SESSION_LEN],
    /// Sequence number of the first message in this packet (or the next expected
    /// sequence number for heartbeats / end-of-session).
    pub seq: u64,
    /// Raw `msg_count` field. Use [`PacketKind`] for a typed interpretation.
    pub msg_count: u16,
}

impl PacketHeader {
    /// Session as a `&str`, trimming trailing spaces. Returns `None` if non-UTF-8.
    pub fn session_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.session).ok().map(|s| s.trim_end())
    }
}

/// Typed classification of a decoded packet based on `msg_count`.
pub enum PacketKind<'a> {
    Messages(MsgIter<'a>),
    Heartbeat,
    EndOfSession,
}

/// A successfully decoded MoldUDP64 packet, borrowing from the original receive buffer.
pub struct DecodedPacket<'a> {
    pub header: PacketHeader,
    pub kind: PacketKind<'a>,
}

/// Parse only the 20-byte packet header.
pub fn parse_header(buf: &[u8]) -> Result<PacketHeader, MoldDecodeError> {
    if buf.len() < HEADER_LEN {
        return Err(MoldDecodeError::HeaderTruncated {
            needed: HEADER_LEN,
            have: buf.len(),
        });
    }
    let mut session = [0u8; SESSION_LEN];
    session.copy_from_slice(&buf[0..SESSION_LEN]);
    Ok(PacketHeader {
        session,
        seq: u64::from_be_bytes(buf[SESSION_LEN..SESSION_LEN + 8].try_into().unwrap()),
        msg_count: u16::from_be_bytes(buf[SESSION_LEN + 8..HEADER_LEN].try_into().unwrap()),
    })
}

/// Decode a full packet buffer into a [`DecodedPacket`].
///
/// Returns [`Err`] if the header is truncated. Heartbeat and end-of-session
/// packets are returned as [`PacketKind::Heartbeat`] / [`PacketKind::EndOfSession`]
/// with no message iterator. Malformed message bodies are handled gracefully by
/// [`MsgIter`] stopping early.
pub fn decode_packet(buf: &[u8]) -> Result<DecodedPacket<'_>, MoldDecodeError> {
    let header = parse_header(buf)?;
    let kind = match header.msg_count {
        HEARTBEAT_MSG_COUNT => PacketKind::Heartbeat,
        END_OF_SESSION_MSG_COUNT => PacketKind::EndOfSession,
        count => PacketKind::Messages(MsgIter::new(&buf[HEADER_LEN..], count)),
    };
    Ok(DecodedPacket { header, kind })
}

/// Encode a normal data packet containing one or more ITCH messages.
/// Allocates — for the replay sender only.
///
/// `session` must be exactly 10 bytes; right-pad with spaces if shorter.
pub fn encode_packet(session: &[u8; SESSION_LEN], seq: u64, itch_messages: &[&[u8]]) -> Vec<u8> {
    let msg_count = itch_messages.len() as u16;
    debug_assert_ne!(
        msg_count, END_OF_SESSION_MSG_COUNT,
        "0xFFFF messages in one packet is not valid; use encode_end_of_session"
    );
    let body_len: usize = itch_messages.iter().map(|m| MSG_LEN_PREFIX + m.len()).sum();
    let mut buf = Vec::with_capacity(HEADER_LEN + body_len);
    write_header(&mut buf, session, seq, msg_count);
    for msg in itch_messages {
        buf.extend_from_slice(&(msg.len() as u16).to_be_bytes());
        buf.extend_from_slice(msg);
    }
    buf
}

#[inline]
fn write_header(buf: &mut Vec<u8>, session: &[u8; SESSION_LEN], seq: u64, msg_count: u16) {
    buf.extend_from_slice(session);
    buf.extend_from_slice(&seq.to_be_bytes());
    buf.extend_from_slice(&msg_count.to_be_bytes());
}

/// Zero-copy iterator over ITCH message slices within a MoldUDP64 packet body.
///
/// Created by [`decode_packet`]; `buf` starts immediately after the 20-byte header.
pub struct MsgIter<'a> {
    buf: &'a [u8],
    remaining: u16,
}

impl<'a> MsgIter<'a> {
    #[inline]
    pub fn new(buf: &'a [u8], msg_count: u16) -> Self {
        Self { buf, remaining: msg_count }
    }
}

impl<'a> Iterator for MsgIter<'a> {
    type Item = &'a [u8];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 || self.buf.len() < MSG_LEN_PREFIX {
            return None;
        }
        let msg_len = u16::from_be_bytes([self.buf[0], self.buf[1]]) as usize;
        let total = MSG_LEN_PREFIX + msg_len;
        if self.buf.len() < total {
            return None;
        }
        let msg = &self.buf[MSG_LEN_PREFIX..total];
        self.buf = &self.buf[total..];
        self.remaining -= 1;
        Some(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SESSION: &[u8; SESSION_LEN] = b"DEMO      ";

    #[test]
    fn header_len_is_20() {
        assert_eq!(HEADER_LEN, 20);
    }

    #[test]
    fn roundtrip_single_message() {
        let payload = b"hello world";
        let packet = encode_packet(SESSION, 42, &[payload]);
        let decoded = decode_packet(&packet).unwrap();
        assert_eq!(decoded.header.seq, 42);
        assert_eq!(decoded.header.msg_count, 1);
        assert_eq!(decoded.header.session, *SESSION);
        let PacketKind::Messages(iter) = decoded.kind else { panic!("expected Messages") };
        let msgs: Vec<_> = iter.collect();
        assert_eq!(msgs, vec![payload.as_slice()]);
    }

    #[test]
    fn roundtrip_multiple_messages() {
        let a = b"msg-a";
        let b = b"msg-b-longer";
        let packet = encode_packet(SESSION, 7, &[a, b]);
        let decoded = decode_packet(&packet).unwrap();
        assert_eq!(decoded.header.seq, 7);
        assert_eq!(decoded.header.msg_count, 2);
        let PacketKind::Messages(iter) = decoded.kind else { panic!("expected Messages") };
        let msgs: Vec<_> = iter.collect();
        assert_eq!(msgs, vec![a.as_slice(), b.as_slice()]);
    }

    #[test]
    fn parse_header_only() {
        let packet = encode_packet(SESSION, 99, &[b"x"]);
        let hdr = parse_header(&packet).unwrap();
        assert_eq!(hdr.seq, 99);
        assert_eq!(hdr.msg_count, 1);
        assert_eq!(&hdr.session, SESSION);
    }

    #[test]
    fn truncated_header_returns_error() {
        assert!(matches!(
            decode_packet(&[0u8; 19]),
            Err(MoldDecodeError::HeaderTruncated {
                needed: HEADER_LEN,
                have: 19
            })
        ));
    }

    #[test]
    fn truncated_message_body_stops_early() {
        let packet = encode_packet(SESSION, 1, &[b"full"]);
        let truncated = &packet[..packet.len() - 1];
        let decoded = decode_packet(truncated).unwrap();
        let PacketKind::Messages(iter) = decoded.kind else { panic!("expected Messages") };
        assert_eq!(iter.collect::<Vec<_>>(), Vec::<&[u8]>::new());
    }

    #[test]
    fn fields_are_big_endian() {
        // Verify seq and msg_count are stored in BE by inspecting raw bytes.
        let packet = encode_packet(SESSION, 0x0102030405060708, &[b"x"]);
        // Bytes 10..18 = seq in BE
        assert_eq!(&packet[10..18], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        // Bytes 18..20 = msg_count = 1 in BE
        assert_eq!(&packet[18..20], &[0x00, 0x01]);
    }
}
