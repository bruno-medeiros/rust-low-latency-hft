//! Ingest raw datagrams into a buffer for decoding.

use crate::decode::ItchDecoder;
use crate::error::IngestError;
use crate::message::ItchMessage;

const COMPACT_THRESHOLD: usize = 64 * 1024;

/// Ingestor that accepts raw datagrams and invokes a callback for each decoded message.
pub struct DatagramIngestor {
    buffer: Vec<u8>,
    read_offset: usize,
    decoder: ItchDecoder,
}

impl DatagramIngestor {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            read_offset: 0,
            decoder: ItchDecoder::new(),
        }
    }

    /// Feed one datagram (e.g. MoldUDP payload or raw UDP). Invokes `on_message` for each decoded record.
    /// Messages borrow from the internal buffer; the callback must not retain references beyond the call.
    pub fn push_datagram(
        &mut self,
        datagram: &[u8],
        mut on_message: impl FnMut(ItchMessage<'_>) -> Result<(), IngestError>,
    ) -> Result<(), IngestError> {
        self.buffer.extend_from_slice(datagram);
        loop {
            let data = &self.buffer[self.read_offset..];
            match self.decoder.pop_message(data)? {
                Some((msg, consumed)) => {
                    on_message(msg)?;
                    self.read_offset += consumed;
                }
                None => break,
            }
        }
        if self.read_offset >= COMPACT_THRESHOLD {
            self.buffer.drain(0..self.read_offset);
            self.read_offset = 0;
        }
        Ok(())
    }
}

impl Default for DatagramIngestor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::encode;
    use crate::message::{ItchMessage, Side};

    #[test]
    fn ingestor_decodes_single_message() {
        let datagram = encode::encode_system_event("START");
        let mut ingestor = DatagramIngestor::new();
        let mut count = 0;
        ingestor
            .push_datagram(&datagram, |msg| {
                assert!(matches!(
                    msg,
                    ItchMessage::SystemEvent { text } if text == "START"
                ));
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn ingestor_decodes_multiple_messages_in_one_datagram() {
        let mut datagram = encode::encode_system_event("A");
        datagram.extend_from_slice(&encode::encode_system_event("B"));
        datagram.extend_from_slice(&encode::encode_add_order(1, Side::Buy, 100, 5000, "AAPL"));
        let mut ingestor = DatagramIngestor::new();
        let mut count = 0;
        ingestor
            .push_datagram(&datagram, |msg| {
                match count {
                    0 => assert!(matches!(msg, ItchMessage::SystemEvent { text } if text == "A")),
                    1 => assert!(matches!(msg, ItchMessage::SystemEvent { text } if text == "B")),
                    2 => assert!(matches!(
                        msg,
                        ItchMessage::AddOrder {
                            oid: 1,
                            side: Side::Buy,
                            qty: 100,
                            price: 5000,
                            symbol
                        } if symbol == "AAPL"
                    )),
                    _ => panic!("unexpected message"),
                }
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn ingestor_reassembles_message_split_across_datagrams() {
        let full = encode::encode_system_event("SPLIT");
        let mid = full.len() / 2;
        let (first, second) = full.split_at(mid);
        let mut ingestor = DatagramIngestor::new();
        let mut count = 0;
        ingestor
            .push_datagram(first, |_msg| {
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 0);
        ingestor
            .push_datagram(second, |msg| {
                assert!(matches!(
                    msg,
                    ItchMessage::SystemEvent { text } if text == "SPLIT"
                ));
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}
