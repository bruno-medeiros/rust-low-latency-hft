//! Market data handler hot loop: UDP RX → MoldUDP64-lite decode → reorder ring → ITCH decode
//! → book apply → strategy → outbound encode → tick-to-trade timestamp.
//!
//! Datagrams are copied into a bounded reorder ring and drained in strict `seq` order.
//! `PipelineConfig::reorder_window` is clamped to at least 1 when constructing the ring.
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
use thiserror::Error;

use crate::itch::{ItchDecoder, ItchMessage};
use crate::itch_to_book::ItchToBookAdapter;
use crate::mold_udp64;
use crate::outbound::OutboundBuf;
use crate::reorder::{OrderedDatagram, PushError, ReorderBuffer, ReorderStats};
use crate::strategy::QuoterState;
use crate::udp_receiver::UdpReceiver;
use crate::util::latency::{LatencyRecorder, RawTs};

/// Errors surfaced by [`MarketHandlerPipeline::run`].
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error(transparent)]
    ReorderPush(#[from] PushError),

    #[error(transparent)]
    MoldDecode(#[from] mold_udp64::MoldDecodeError),
}

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
    /// Datagrams reorder ring capacity (values below 1 are clamped to 1 at runtime).
    pub reorder_window: usize,
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
    pub reorder_stats: ReorderStats,
}

/// Market data UDP → book → strategy pipeline, configured via [`PipelineConfig`].
pub struct MarketHandlerPipeline {
    pub config: PipelineConfig,
    reorder: ReorderBuffer,
    decoder: ItchDecoder,
    itch_to_book_adapter: ItchToBookAdapter,
    events: CountingEventSink,
    quoter: QuoterState,
    latency: LatencyRecorder,
}

impl MarketHandlerPipeline {
    /// Build pipeline state from config (reorder ring, decoders, strategy, latency recorder).
    pub fn from_config(config: PipelineConfig) -> Self {
        let reorder_window = config.reorder_window.max(1);
        Self {
            config,
            reorder: ReorderBuffer::new(config.first_seq, reorder_window),
            decoder: ItchDecoder::new(),
            itch_to_book_adapter: ItchToBookAdapter::new(),
            events: CountingEventSink::default(),
            quoter: QuoterState::new(),
            latency: LatencyRecorder::new(),
        }
    }

    /// Run until `done` is set.
    ///
    /// `book` is owned for the duration of the run. The caller supplies a bound socket.
    pub fn run<B: LimitOrderBook>(
        mut self,
        socket: UdpSocket,
        done: Arc<AtomicBool>,
        mut book: B,
    ) -> Result<PipelineResult, PipelineError> {
        if self.config.core_pinning_enabled {
            core_affinity::set_for_current(CoreId {
                id: self.config.pin_core as usize,
            });
        }

        if let Some(ms) = self.config.read_timeout_ms {
            socket
                .set_read_timeout(Some(std::time::Duration::from_millis(ms)))
                .expect("set_read_timeout");
        }

        let mut rx = UdpReceiver::new(socket);

        let mut packets_received: u64 = 0;
        let mut messages_decoded: u64 = 0;
        let mut orders_emitted: u64 = 0;
        let mut reorder_stats = ReorderStats::default();

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

            for buf in batch {
                packets_received += 1;
                let packet = match mold_udp64::decode_packet(buf.as_slice()) {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                let seq = packet.header.seq;
                let t0 = self.latency.now();

                reorder_stats.record_push_ok(self.reorder.push(seq, buf.as_slice(), t0)?);

                while let Some(d) = self.reorder.pop_ready() {
                    self.process_next_message(d, &mut book, &mut messages_decoded, &mut orders_emitted)?;
                }
            }
        }

        Ok(PipelineResult {
            packets_received,
            messages_decoded,
            orders_emitted,
            book_events: self.events,
            latency: self.latency,
            reorder_stats,
        })
    }

    fn process_next_message<B: LimitOrderBook>(
        &mut self,
        datagram: OrderedDatagram,
        book: &mut B,
        messages_decoded: &mut u64,
        orders_emitted: &mut u64,
    ) -> Result<(), PipelineError> {
        let t0 = datagram.t0;
        let packet = mold_udp64::decode_packet(datagram.as_slice())?;
        let mold_udp64::PacketKind::Messages(msg_iter) = packet.kind else {
            return Ok(());
        };
        for msg_slice in msg_iter {
            if let Ok(Some((msg, _consumed))) = self.decoder.pop_message(msg_slice) {
                self.process_itch_message(&msg, t0, book, messages_decoded, orders_emitted);
            }
        }
        Ok(())
    }

    fn process_itch_message<B: LimitOrderBook>(
        &mut self,
        msg: &ItchMessage<'_>,
        t0: RawTs,
        book: &mut B,
        messages_decoded: &mut u64,
        orders_emitted: &mut u64,
    ) {
        *messages_decoded += 1;
        let _ = self
            .itch_to_book_adapter
            .apply(book, msg, &mut self.events);

        let mut out = OutboundBuf::default();
        if self.quoter.on_book_update(book, &mut out) {
            let t1 = self.latency.now();
            self.latency.record(t0, t1);
            *orders_emitted += 1;
            std::hint::black_box(out.as_slice());
        }
    }
}
