//! Bounded reorder buffer for MoldUDP64 datagrams: watermark + sliding ring.
//!
//! Copies each datagram into fixed [`crate::udp_receiver::BUF_SIZE`] slabs in the ring so
//! [`crate::udp_receiver::UdpReceiver`] receive buffers can be reused immediately.
//! Drains strictly in ascending `seq` order for the hot path downstream.

use thiserror::Error;

use crate::udp_receiver::BUF_SIZE;
use crate::util::latency::RawTs;

/// Fixed storage for one datagram payload (same span as UDP receive buffers).
pub type DatagramBytes = [u8; BUF_SIZE];

/// One datagram ready to decode after in-order drain.
pub struct OrderedDatagram {
    len: usize,
    bytes: DatagramBytes,
    /// `LatencyRecorder::now()` taken when this payload was copied into the buffer (T0).
    pub t0: RawTs,
}

impl OrderedDatagram {
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

struct Slot {
    occupied: bool,
    seq: u64,
    len: usize,
    data: DatagramBytes,
    t0: RawTs,
}

/// Failure when `seq` is too far ahead of the watermark for this window size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error(
    "sequence {seq} is beyond reorder window (next_expected={next_expected}, window={window})"
)]
pub struct ReorderWindowExceeded {
    pub seq: u64,
    pub next_expected: u64,
    pub window: usize,
}

/// Datagram payload cannot fit in a reorder slot (`len` > [`BUF_SIZE`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("payload length {len} exceeds reorder slot ({max})")]
pub struct PayloadTooLarge {
    pub len: usize,
    pub max: usize,
}

/// Error returned by [`ReorderBuffer::push`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PushError {
    #[error(transparent)]
    WindowExceeded(#[from] ReorderWindowExceeded),
    #[error(transparent)]
    PayloadTooLarge(#[from] PayloadTooLarge),
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
    slots: Vec<Slot>,
    stats: ReorderStats,
}

impl ReorderBuffer {
    /// `window` must be >= 1.
    pub fn new(first_seq: u64, window: usize) -> Self {
        assert!(window >= 1, "reorder window must be positive");
        Self {
            window,
            next_expected: first_seq,
            start: 0,
            slots: (0..window)
                .map(|_| Slot {
                    occupied: false,
                    seq: 0,
                    len: 0,
                    data: [0u8; BUF_SIZE],
                    t0: 0,
                })
                .collect(),
            stats: ReorderStats::default(),
        }
    }

    pub fn next_expected(&self) -> u64 {
        self.next_expected
    }

    /// Insert a full raw datagram (MoldUDP64 wire bytes). `t0` is stamped at copy time.
    pub fn push(
        &mut self,
        seq: u64,
        src: &[u8],
        t0: RawTs,
    ) -> Result<PushOutcome, PushError> {
        if src.len() > BUF_SIZE {
            return Err(PayloadTooLarge {
                len: src.len(),
                max: BUF_SIZE,
            }
            .into());
        }

        if seq < self.next_expected {
            self.stats.record_push_ok(PushOutcome::LateDuplicate);
            return Ok(PushOutcome::LateDuplicate);
        }

        let dist = seq - self.next_expected;
        if dist >= self.window as u64 {
            return Err(ReorderWindowExceeded {
                seq,
                next_expected: self.next_expected,
                window: self.window,
            }
            .into());
        }

        let arrived_ahead = dist > 0;
        let slots_index = (self.start + dist as usize) % self.window;

        if self.slots[slots_index].occupied {
            if self.slots[slots_index].seq == seq {
                self.stats.record_push_ok(PushOutcome::DuplicateSeq);
                return Ok(PushOutcome::DuplicateSeq);
            }
            unreachable!(
                "reorder slot collision: seq {} but slot holds {}",
                seq,
                self.slots[slots_index].seq
            );
        }

        let slot = &mut self.slots[slots_index];
        slot.occupied = true;
        slot.seq = seq;
        slot.len = src.len();
        slot.t0 = t0;
        slot.data[..src.len()].copy_from_slice(src);
        let outcome = PushOutcome::Buffered { arrived_ahead };
        self.stats.record_push_ok(outcome);
        Ok(outcome)
    }

    /// Advance `next_expected` by one without inserting anything into the ring.
    pub fn advance_in_order(&mut self) {
        self.next_expected += 1;
        self.start = (self.start + 1) % self.window;
    }

    pub fn stats(&self) -> ReorderStats {
        self.stats
    }

    /// Remove the next contiguous ready datagram if `next_expected` is present; otherwise `None`.
    pub fn pop_ready(&mut self) -> Option<OrderedDatagram> {
        let slot = &mut self.slots[self.start];
        if !slot.occupied {
            return None;
        }
        debug_assert_eq!(slot.seq, self.next_expected, "slot seq mismatch at drain");
        slot.occupied = false;
        let datagram = OrderedDatagram {
            len: slot.len,
            // FIXME: optimize this
            bytes: slot.data,
            t0: slot.t0,
        };
        self.next_expected += 1;
        self.start = (self.start + 1) % self.window;
        Some(datagram)
    }
}

#[cfg(test)]
impl ReorderBuffer {
    /// Drains all contiguous ready datagrams into a `Vec` (unit tests only; hot path uses [`pop_ready`](ReorderBuffer::pop_ready)).
    fn drain_ready(&mut self) -> Vec<OrderedDatagram> {
        let mut out = Vec::new();
        while let Some(d) = self.pop_ready() {
            out.push(d);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_seq(rb: &mut ReorderBuffer, seq: u64) -> Result<PushOutcome, PushError> {
        let b = [seq as u8; 4];
        rb.push(seq, &b, seq)
    }

    #[test]
    fn permuted_321_drains_123() {
        let mut rb = ReorderBuffer::new(1, 8);
        assert!(matches!(
            push_seq(&mut rb, 3).unwrap(),
            PushOutcome::Buffered { arrived_ahead: true }
        ));
        assert!(push_seq(&mut rb, 1).unwrap() == PushOutcome::Buffered { arrived_ahead: false });
        assert_eq!(rb.drain_ready().len(), 1);
        assert!(matches!(
            push_seq(&mut rb, 2).unwrap(),
            PushOutcome::Buffered { arrived_ahead: false }
        ));
        let drained = rb.drain_ready();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].as_slice(), &[2u8; 4]);
        assert_eq!(drained[1].as_slice(), &[3u8; 4]);
        assert_eq!(rb.next_expected(), 4);
    }

    #[test]
    fn late_packet_dropped() {
        let mut rb = ReorderBuffer::new(10, 4);
        assert_eq!(
            rb.push(9, &[1], 0).unwrap(),
            PushOutcome::LateDuplicate
        );
        assert!(rb.drain_ready().is_empty());
    }

    #[test]
    fn duplicate_seq_rejected() {
        let mut rb = ReorderBuffer::new(1, 8);
        push_seq(&mut rb, 2).unwrap();
        assert_eq!(
            push_seq(&mut rb, 2).unwrap(),
            PushOutcome::DuplicateSeq
        );
    }

    #[test]
    fn window_exceeded_err() {
        let mut rb = ReorderBuffer::new(0, 4);
        let err = rb.push(4, &[], 0).unwrap_err();
        assert!(matches!(
            err,
            PushError::WindowExceeded(ReorderWindowExceeded {
                seq: 4,
                next_expected: 0,
                window: 4
            })
        ));
    }

    #[test]
    fn ring_wraps_start() {
        let mut rb = ReorderBuffer::new(0, 4);
        for s in 0u64..4 {
            push_seq(&mut rb, s).unwrap();
        }
        assert_eq!(rb.drain_ready().len(), 4);
        assert_eq!(rb.next_expected(), 4);

        push_seq(&mut rb, 5).unwrap();
        push_seq(&mut rb, 4).unwrap();
        assert_eq!(rb.drain_ready().len(), 2);
        assert_eq!(rb.next_expected(), 6);
    }
}
