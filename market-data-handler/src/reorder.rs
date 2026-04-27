//! Bounded reorder buffer for MoldUDP64 datagrams: watermark + sliding ring.
//!
//! Copies each datagram into owned storage so [`crate::udp_receiver::UdpReceiver`] can reuse
//! receive slabs. Drains strictly in ascending `seq` order for the hot path downstream.

use crate::util::latency::RawTs;
use thiserror::Error;

/// One datagram ready to decode after in-order drain.
pub struct OrderedDatagram {
    pub bytes: Vec<u8>,
    /// `LatencyRecorder::now()` taken when this payload was copied into the buffer (T0).
    pub t0: RawTs,
}

struct Slot {
    seq: u64,
    seq_span: u64,
    data: Vec<u8>,
    t0: RawTs,
}

/// Failure when `seq` is too far ahead of the watermark for this window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("sequence {seq} is beyond reorder window (next_expected={next_expected}, window={window})")]
pub struct ReorderWindowExceeded {
    pub seq: u64,
    pub next_expected: u64,
    pub window: usize,
}

/// Result of [`ReorderBuffer::push`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    Buffered { arrived_ahead: bool },
    LateDuplicate,
    DuplicateSeq,
}

/// Counters for reorder-buffer ingest (e.g. aggregated per pipeline run).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReorderStats {
    pub packets_late_duplicate: u64,
    pub packets_duplicate_seq: u64,
    pub reorder_ahead_arrivals: u64,
}

impl ReorderStats {
    pub fn record_push_ok(&mut self, outcome: PushOutcome) {
        match outcome {
            PushOutcome::Buffered { arrived_ahead } => {
                if arrived_ahead {
                    self.reorder_ahead_arrivals += 1;
                }
            }
            PushOutcome::LateDuplicate => self.packets_late_duplicate += 1,
            PushOutcome::DuplicateSeq => self.packets_duplicate_seq += 1,
        }
    }
}

/// Fixed-size reorder ring: accepts datagrams with `seq` in
/// `[next_expected, next_expected + window)`.
pub struct ReorderBuffer {
    window: usize,
    next_expected: u64,
    /// Logical index of message at offset 0 in slots (`seq == next_expected`).
    start: usize,
    slots: Vec<Option<Slot>>,
}

impl ReorderBuffer {
    /// `window` must be >= 1.
    pub fn new(first_seq: u64, window: usize) -> Self {
        assert!(window >= 1, "reorder window must be positive");
        Self {
            window,
            next_expected: first_seq,
            start: 0,
            slots: (0..window).map(|_| None).collect(),
        }
    }

    pub fn next_expected(&self) -> u64 {
        self.next_expected
    }

    /// Insert a full raw datagram (MoldUDP64 wire bytes). `t0` is stamped at copy time.
    pub fn push(
        &mut self,
        seq: u64,
        data: Vec<u8>,
        t0: RawTs,
    ) -> Result<PushOutcome, ReorderWindowExceeded> {
        self.push_with_span(seq, 1, data, t0)
    }

    /// Insert a datagram whose sequence number covers `seq_span` MoldUDP64 message
    /// sequence numbers. Heartbeats use a span of zero; normal data packets use
    /// their `msg_count`.
    pub fn push_with_span(
        &mut self,
        seq: u64,
        seq_span: u64,
        data: Vec<u8>,
        t0: RawTs,
    ) -> Result<PushOutcome, ReorderWindowExceeded> {
        if seq < self.next_expected {
            return Ok(PushOutcome::LateDuplicate);
        }

        let dist = seq - self.next_expected;
        if dist >= self.window as u64 {
            return Err(ReorderWindowExceeded {
                seq,
                next_expected: self.next_expected,
                window: self.window,
            });
        }

        let arrived_ahead = dist > 0;
        let slots_index = (self.start + dist as usize) % self.window;

        if let Some(existing) = &self.slots[slots_index] {
            if existing.seq == seq {
                return Ok(PushOutcome::DuplicateSeq);
            }
            unreachable!(
                "reorder slot collision: seq {} but slot holds {}",
                seq, existing.seq
            );
        }

        self.slots[slots_index] = Some(Slot {
            seq,
            seq_span,
            data,
            t0,
        });
        Ok(PushOutcome::Buffered { arrived_ahead })
    }

    /// Remove and return all contiguous datagrams starting at `next_expected`.
    pub fn drain_ready(&mut self) -> Vec<OrderedDatagram> {
        let mut out = Vec::new();
        loop {
            // TODO: refactor this to use a match
            if self.slots[self.start].is_none() {
                break;
            }
            let slot = self.slots[self.start].take().expect("checked is_some");
            debug_assert_eq!(slot.seq, self.next_expected, "slot seq mismatch at drain");
            let old_start = self.start;
            self.next_expected += slot.seq_span;
            if slot.seq_span > 0 {
                let slots_to_clear = slot.seq_span.min(self.window as u64) as usize;
                for offset in 1..slots_to_clear {
                    self.slots[(old_start + offset) % self.window] = None;
                }
                self.start = (old_start + slots_to_clear) % self.window;
            }
            out.push(OrderedDatagram {
                bytes: slot.data,
                t0: slot.t0,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_seq(rb: &mut ReorderBuffer, seq: u64) -> Result<PushOutcome, ReorderWindowExceeded> {
        rb.push(seq, vec![seq as u8; 4], seq)
    }

    #[test]
    fn permuted_321_drains_123() {
        let mut rb = ReorderBuffer::new(1, 8);
        assert!(matches!(
            push_seq(&mut rb, 3).unwrap(),
            PushOutcome::Buffered {
                arrived_ahead: true
            }
        ));
        assert!(
            push_seq(&mut rb, 1).unwrap()
                == PushOutcome::Buffered {
                    arrived_ahead: false
                }
        );
        assert_eq!(rb.drain_ready().len(), 1);
        assert!(matches!(
            push_seq(&mut rb, 2).unwrap(),
            PushOutcome::Buffered {
                arrived_ahead: false
            }
        ));
        let d = rb.drain_ready();
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].bytes, vec![2u8; 4]);
        assert_eq!(d[1].bytes, vec![3u8; 4]);
        assert_eq!(rb.next_expected(), 4);
    }

    #[test]
    fn late_packet_dropped() {
        let mut rb = ReorderBuffer::new(10, 4);
        assert_eq!(rb.push(9, vec![1], 0).unwrap(), PushOutcome::LateDuplicate);
        assert_eq!(rb.drain_ready().len(), 0);
    }

    #[test]
    fn duplicate_seq_rejected() {
        let mut rb = ReorderBuffer::new(1, 8);
        push_seq(&mut rb, 2).unwrap();
        assert_eq!(push_seq(&mut rb, 2).unwrap(), PushOutcome::DuplicateSeq);
    }

    #[test]
    fn window_exceeded_err() {
        let mut rb = ReorderBuffer::new(0, 4);
        let err = rb.push(4, vec![], 0).unwrap_err();
        assert_eq!(err.seq, 4);
        assert_eq!(err.next_expected, 0);
        assert_eq!(err.window, 4);
    }

    #[test]
    fn ring_wraps_start() {
        let mut rb = ReorderBuffer::new(0, 4);
        for s in 0u64..4 {
            push_seq(&mut rb, s).unwrap();
        }
        let d = rb.drain_ready();
        assert_eq!(d.len(), 4);
        assert_eq!(rb.next_expected(), 4);
        assert_eq!(rb.start, 0);

        push_seq(&mut rb, 5).unwrap();
        push_seq(&mut rb, 4).unwrap();
        let d2 = rb.drain_ready();
        assert_eq!(d2.len(), 2);
        assert_eq!(rb.next_expected(), 6);
    }

    #[test]
    fn multi_message_datagram_advances_by_message_span() {
        let mut rb = ReorderBuffer::new(0, 8);
        rb.push_with_span(0, 2, vec![0], 0).unwrap();
        rb.push_with_span(2, 1, vec![2], 0).unwrap();

        let d = rb.drain_ready();
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].bytes, vec![0]);
        assert_eq!(d[1].bytes, vec![2]);
        assert_eq!(rb.next_expected(), 3);
    }
}
