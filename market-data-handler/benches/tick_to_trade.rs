//! Tick-to-trade latency benchmark for the market data handler pipeline.
//!
//! # What is measured
//!
//! T0 = `quanta::Clock::raw()` immediately after `recvmmsg(2)` returns (first byte of the
//!      batch is available in userspace; kernel RX path is already complete).
//! T1 = `quanta::Clock::raw()` immediately before writing to `OutboundBuf` (order bytes
//!      encoded, ready for `sendto`; the actual syscall is excluded).
//!
//! Measured on loopback. Does not include kernel TX path.
//! Strategy is a minimal top-of-book cross-spread quoter; intentionally trivial so
//! the number isolates pipeline latency rather than strategy complexity.
//!
//! # Data
//!
//! Synthetic ITCH messages: alternating AddOrder (Buy @ 100, Sell @ 200) to maintain a
//! persistent spread that triggers the quoter on every book update.

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use bench_tool::{
    AllocStats, BenchReportSection, CliArgs, LatencyScenario, LatencyStats,
    core_pinning_disabled_by_env,
};
use limit_order_book::LimitOrderBookV1;
use market_data_handler::{
    MarketHandlerPipeline, PipelineConfig,
    itch::{Side, encode},
    mold_udp64::{SESSION_LEN, encode_packet},
};

/// Number of ITCH messages to send. Drives sample count in the histogram.
const N_MESSAGES: usize = 50_000;

/// MoldUDP64 session identifier (10 bytes, right-padded with spaces).
const SESSION: &[u8; SESSION_LEN] = b"BENCH     ";

/// Prices chosen so best_bid + 1 < best_ask holds throughout — quoter fires every update.
const BUY_PRICE: u32 = 100;
const SELL_PRICE: u32 = 200;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse_args();

    // ── Pre-compute all UDP packets (no alloc on hot path) ───────────────────────────
    let packets: Vec<Vec<u8>> = (0..N_MESSAGES)
        .map(|i| {
            let seq = i as u64;
            let itch = if i % 2 == 0 {
                encode::encode_add_order(seq + 1, Side::Buy, 1, BUY_PRICE, "SYM")
            } else {
                encode::encode_add_order(seq + 1, Side::Sell, 1, SELL_PRICE, "SYM")
            };
            encode_packet(SESSION, seq, &[&itch])
        })
        .collect();

    // ── Bind sockets ─────────────────────────────────────────────────────────────────
    let rx_sock = UdpSocket::bind("127.0.0.1:0")?;
    let rx_addr = rx_sock.local_addr()?;
    let tx_sock = UdpSocket::bind("127.0.0.1:0")?;

    // ── Core pinning ─────────────────────────────────────────────────────────────────
    let pin_enabled = !core_pinning_disabled_by_env();
    let pipeline_core = args.pin_core;
    let sender_core = args.pin_core_b;

    // ── Configuration ────────────────────────────────────────────────────────────────
    let config = PipelineConfig {
        // Price range covers the synthetic BUY_PRICE and SELL_PRICE.
        price_range: (1, 1_000),
        order_capacity: N_MESSAGES as u64,
        core_pinning_enabled: pin_enabled,
        pin_core: pipeline_core,
        first_seq: 0,
        // 5 ms timeout so `done` is checked promptly after sender finishes.
        read_timeout_ms: Some(5),
    };

    let done = Arc::new(AtomicBool::new(false));
    let done_tx = done.clone();

    // ── Sender thread ────────────────────────────────────────────────────────────────
    let sender = thread::spawn(move || {
        if pin_enabled {
            core_affinity::set_for_current(core_affinity::CoreId { id: sender_core as usize });
        }
        // Small initial delay so the pipeline thread is ready in recvmmsg.
        thread::sleep(Duration::from_millis(20));

        for pkt in &packets {
            tx_sock.send_to(pkt, rx_addr).expect("send_to");
        }
        // Give pipeline time to drain the kernel RX buffer before signalling done.
        thread::sleep(Duration::from_millis(50));
        done_tx.store(true, Ordering::Release);
    });

    // ── Pipeline (runs on this thread) ───────────────────────────────────────────────
    let book = LimitOrderBookV1::new(config.price_range, config.order_capacity as usize);
    let result = MarketHandlerPipeline::new(config)
        .run(rx_sock, done, book)
        .expect("pipeline sequential seq");

    sender.join().expect("sender thread panicked");

    // ── Build bench-tool report ───────────────────────────────────────────────────────
    let lat = &result.latency;
    let latency_stats = LatencyStats {
        min_ns:  lat.min_ns(),
        p50_ns:  lat.p50_ns(),
        p90_ns:  lat.p90_ns(),
        p95_ns:  lat.p95_ns(),
        p99_ns:  lat.p99_ns(),
        p999_ns: lat.p999_ns(),
        max_ns:  lat.max_ns(),
        mean_ns: lat.mean_ns(),
        stdev_ns: lat.stdev_ns(),
    };

    let zero_allocs = AllocStats {
        total_allocs: 0,
        total_deallocs: 0,
        total_bytes: 0,
        avg_allocs_per_op: 0.0,
        avg_deallocs_per_op: 0.0,
        avg_bytes_per_op: 0.0,
    };

    let scenario = LatencyScenario {
        name: "Tick-to-trade".into(),
        samples: lat.sample_count(),
        latency: latency_stats,
        allocations: zero_allocs,
    };

    let mut runner = bench_tool::BenchRunner::new("market-data-handler: tick-to-trade")
        .filter(args.filter.clone());

    let mut report = runner.initial_report();

    let mut section = BenchReportSection::new("Tick-to-trade pipeline");
    section.add_param("messages_sent", N_MESSAGES.to_string());
    section.add_param("samples_recorded", lat.sample_count().to_string());
    section.add_param("packets_received", result.packets_received.to_string());
    section.add_param("messages_decoded", result.messages_decoded.to_string());
    section.add_param("orders_emitted", result.orders_emitted.to_string());
    section.add_param("book_events_accepted", result.book_events.accepted.to_string());
    let pin_note = runner.pin_to_isolated_core(pipeline_core);
    let sender_pin_note = runner.pin_to_isolated_core(sender_core);
    section.add_param("pipeline_pin_core", pin_note);
    section.add_param("sender_pin_core", sender_pin_note);
    section.add_param(
        "T0_definition",
        "quanta::Clock::raw() after recvmmsg() returns",
    );
    section.add_param(
        "T1_definition",
        "quanta::Clock::raw() before OutboundBuf write (excludes sendto syscall)",
    );

    section.latency_scenarios.push(scenario);
    runner.push_section(section, &mut report);

    args.execute(&report)
}
