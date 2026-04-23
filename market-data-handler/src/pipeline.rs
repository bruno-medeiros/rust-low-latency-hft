//! Market data handler hot loop: UDP RX → MoldUDP64-lite decode → strict seq check → ITCH decode
//! → book apply → strategy → outbound encode → tick-to-trade timestamp.
//!
//! Everything runs on a single pinned thread. No cross-thread queue on the hot path;
//! the book update and strategy decision are inline. A side-channel (e.g. SPSC journal)
//! can be added by wrapping the pipeline runner and forwarding `BookEvent`s.

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use core_affinity::CoreId;
use limit_order_book::LimitOrderBook;
use limit_order_book::event::CountingEventSink;

use crate::itch::ItchDecoder;
use crate::error::SeqOrderError;
use crate::itch_to_book::ItchToBookAdapter;
use crate::mold_udp64;
use crate::latency::LatencyRecorder;
use crate::outbound::OutboundBuf;
use crate::rx::UdpReceiver;
use crate::strategy::QuoterState;

/// Configuration for the market data handler pipeline.
#[derive(Clone, Copy)]
pub struct PipelineConfig {
    /// LOB price tick range `(min_price, max_price)`.
    pub price_range: (u64, u64),
    /// LOB pre-allocation hint for order capacity.
    pub order_capacity: u64,
    /// If true, pin the pipeline thread to `pin_core`.
    pub core_pinning_enabled: bool,
    /// CPU core to pin the pipeline thread to.
    pub pin_core: u32,
    /// Sequence number of the first expected packet.
    pub first_seq: u64,
    /// Socket read timeout in milliseconds. Applied before entering the hot loop so that
    /// `done` is checked periodically even when the feed is idle. `None` → blocking.
    pub read_timeout_ms: Option<u64>,
}

/// Summary statistics returned after the pipeline terminates.
pub struct PipelineResult {
    pub packets_received: u64,
    pub messages_decoded: u64,
    pub orders_emitted: u64,
    pub book_events: CountingEventSink,
    pub latency: LatencyRecorder,
}

/// Market data UDP → book → strategy pipeline, configured via [`PipelineConfig`].
pub struct MarketHandlerPipeline {
    pub config: PipelineConfig,
}

impl MarketHandlerPipeline {
    pub const fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Run until `done` is set.
    ///
    /// `book` is owned for the duration of the run. The caller supplies a bound socket.
    ///
    /// Returns [`Err`] if a decoded packet's sequence number is not exactly the next expected
    /// value (strict in-order delivery, including heartbeat / end-of-session packets).
    pub fn run<B: LimitOrderBook>(
        self,
        socket: UdpSocket,
        done: Arc<AtomicBool>,
        mut book: B,
    ) -> Result<PipelineResult, SeqOrderError> {
        let config = self.config;
        if config.core_pinning_enabled {
            core_affinity::set_for_current(CoreId { id: config.pin_core as usize });
        }

        if let Some(ms) = config.read_timeout_ms {
            socket
                .set_read_timeout(Some(std::time::Duration::from_millis(ms)))
                .expect("set_read_timeout");
        }

        let mut rx = UdpReceiver::new(socket);
        let mut expected_seq = config.first_seq;
        let mut decoder = ItchDecoder::new();
        let mut itch_to_book_adapter = ItchToBookAdapter::new();
        let mut events = CountingEventSink::default();
        let mut quoter = QuoterState::new();
        let mut latency = LatencyRecorder::new();

        let mut packets_received: u64 = 0;
        let mut messages_decoded: u64 = 0;
        let mut orders_emitted: u64 = 0;

        loop {
            if done.load(Ordering::Relaxed) {
                break;
            }

            let batch = match rx.recv_batch() {
                Ok(b) => b,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    continue
                }
                Err(_) => break,
            };

            // T0: timestamp at first-byte-available (after recvmmsg returns).
            let t0 = latency.now();

            for buf in batch {
                packets_received += 1;
                let Some(packet) = mold_udp64::decode_packet(buf.as_slice()) else {
                    continue;
                };

                let seq = packet.header.seq;
                if seq != expected_seq {
                    return Err(SeqOrderError::OutOfOrder {
                        expected: expected_seq,
                        got: seq,
                    });
                }
                expected_seq += 1;

                let mold_udp64::PacketKind::Messages(msg_iter) = packet.kind else {
                    continue;
                };

                for msg_slice in msg_iter {
                    if let Ok(Some((msg, _consumed))) = decoder.pop_message(msg_slice) {
                        messages_decoded += 1;
                        let _ = itch_to_book_adapter.apply(&mut book, &msg, &mut events);

                        let mut out = OutboundBuf::default();
                        if quoter.on_book_update(&book, &mut out) {
                            // T1: timestamp immediately before outbound write.
                            let t1 = latency.now();
                            latency.record(t0, t1);
                            orders_emitted += 1;
                            // In production: outbound_socket.send(out.as_slice())
                            std::hint::black_box(out.as_slice());
                        }
                    }
                }
            }
        }

        Ok(PipelineResult {
            packets_received,
            messages_decoded,
            orders_emitted,
            book_events: events,
            latency,
        })
    }
}
