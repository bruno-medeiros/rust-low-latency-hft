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

use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use bench_tool::stats_alloc::Region;
use bench_tool::{
    AllocStats, BenchReport, BenchReportSection, BenchRunner, CliArgs, INSTRUMENTED_SYSTEM,
    LatencyScenario, LatencyStats, alloc_stats_from_basic_stats, core_pinning_disabled_by_env,
};
use limit_order_book::LimitOrderBookV1;
use market_data_handler::{LatencyRecorder, MarketHandlerPipeline, PipelineConfig, PipelineResult, itch::{Side, encode}, mold_udp64::{SESSION_LEN, encode_packet}, PipelineError};

const N_MESSAGES: usize = 50_000;
const SESSION: &[u8; SESSION_LEN] = b"BENCH     ";
const BUY_PRICE: u32 = 100;
const SELL_PRICE: u32 = 200;

/// Reorder chunk: 5 in order, then 4 ahead of the next in-sequence, then that slot.
const REORDER_CYCLE: usize = 10;

fn build_synthetic_packets() -> Vec<Vec<u8>> {
    (0..N_MESSAGES)
        .map(|i| {
            let seq = i as u64;
            let itch = if i % 2 == 0 {
                encode::encode_add_order(seq + 1, Side::Buy, 1, BUY_PRICE, "SYM")
            } else {
                encode::encode_add_order(seq + 1, Side::Sell, 1, SELL_PRICE, "SYM")
            };
            encode_packet(SESSION, seq, &[&itch])
        })
        .collect()
}

fn pipeline_config(pin_enabled: bool, pin_core: u32) -> PipelineConfig {
    PipelineConfig {
        price_range: (1, 1_000),
        order_capacity: N_MESSAGES as u64,
        core_pinning_enabled: pin_enabled,
        pin_core,
        first_seq: 0,
        reorder_window: 256,
        read_timeout_ms: Some(5),
    }
}

fn latency_stats(lat: &LatencyRecorder) -> LatencyStats {
    LatencyStats {
        min_ns: lat.min_ns(),
        p50_ns: lat.p50_ns(),
        p90_ns: lat.p90_ns(),
        p95_ns: lat.p95_ns(),
        p99_ns: lat.p99_ns(),
        p999_ns: lat.p999_ns(),
        max_ns: lat.max_ns(),
        mean_ns: lat.mean_ns(),
        stdev_ns: lat.stdev_ns(),
    }
}

fn add_shared_tick_to_trade_params(
    section: &mut BenchReportSection,
    result: &PipelineResult,
    alloc: &AllocStats,
) {
    section.add_param("packets_received", result.packets_received.to_string());
    section.add_param("messages_decoded", result.messages_decoded.to_string());
    section.add_param(
        "reorder_ahead_arrivals",
        result.reorder_stats.reorder_ahead_arrivals.to_string(),
    );
    section.add_param("orders_emitted", result.orders_emitted.to_string());
    section.add_param("total_allocs", alloc.total_allocs.to_string());
    section.add_param("total_deallocs", alloc.total_deallocs.to_string());
    section.add_param("total_bytes", alloc.total_bytes.to_string());
}

fn add_tick_to_trade_global_report_params(
    report: &mut BenchReport,
    runner: &BenchRunner,
    pipeline_core: u32,
    sender_core: u32,
) {
    report.metadata.params.insert(
        "pipeline_pin_core".into(),
        runner.pin_to_isolated_core(pipeline_core),
    );
    report.metadata.params.insert(
        "bench_sender_pin_core".into(),
        runner.pin_to_isolated_core(sender_core),
    );
    report.metadata.params.insert(
        "T0_definition".into(),
        "quanta::Clock::raw() after recvmmsg() returns".into(),
    );
    report.metadata.params.insert(
        "T1_definition".into(),
        "quanta::Clock::raw() before OutboundBuf write (excludes sendto syscall)".into(),
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(N_MESSAGES % REORDER_CYCLE, 0);
    let args = CliArgs::parse_args();

    let pipeline_core = args.pin_core;
    let sender_core = args.pin_core_b;

    let mut runner = bench_tool::BenchRunner::new("market-data-handler: tick-to-trade")
        .filter(args.filter.clone());

    let mut report = runner.initial_report();
    add_tick_to_trade_global_report_params(&mut report, &runner, pipeline_core, sender_core);

    let mut section = BenchReportSection::new("Tick-to-trade pipeline (in-order)");
    run_scenario(
        SendPattern::InOrder,
        pipeline_core,
        "In-order packets",
        &mut section,
    )?;
    runner.push_section(section, &mut report);

    let mut section = BenchReportSection::new("Tick-to-trade pipeline (out of order inbound)");

    run_scenario(
        SendPattern::ReorderedSegments,
        pipeline_core,
        "Out-of-order inbound",
        &mut section,
    )?;
    runner.push_section(section, &mut report);

    args.execute(&report)
}

#[derive(Clone, Copy)]
enum SendPattern {
    InOrder,
    ReorderedSegments,
}

fn run_scenario(
    pattern: SendPattern,
    pipeline_core: u32,
    scenario_name: &str,
    section: &mut BenchReportSection,
) -> Result<(), Box<dyn std::error::Error>> {
    let rx_sock = UdpSocket::bind("127.0.0.1:0")?;
    let rx_addr = rx_sock.local_addr()?;
    let tx_sock = UdpSocket::bind("127.0.0.1:0")?;
    let pin_enabled = !core_pinning_disabled_by_env();
    let config = pipeline_config(pin_enabled, pipeline_core);
    let done = Arc::new(AtomicBool::new(false));
    let done_flag = done.clone();

    let book = LimitOrderBookV1::new(config.price_range, config.order_capacity as usize);
    let pipeline = MarketHandlerPipeline::from_config(config);

    let packets = build_synthetic_packets();

    // TODO use notify condition variable
    let input_sender_thread = start_pipeline_input_sender(packets, pattern, rx_addr, tx_sock, done_flag);

    let pipeline_handle = thread::spawn(move || {
        // Measure allocations around pipeline run
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let (pipeline_result, pipeline) = pipeline.run(rx_sock, done, book)?;
        let alloc_stats = region.change();

        Ok::<_, PipelineError>((pipeline_result, pipeline.latency, alloc_stats))
    });

    input_sender_thread
        .join()
        .expect("sender join");

    let (pipeline_result, latency, alloc_stats) = pipeline_handle.join().expect("pipeline join")?;
    let samples = latency.sample_count();
    let alloc_stats = alloc_stats_from_basic_stats(alloc_stats, samples);

    section.latency_scenarios.push(LatencyScenario {
        name: scenario_name.into(),
        samples: latency.sample_count(),
        latency: latency_stats(&latency),
        allocations: alloc_stats.clone(),
    });
    add_shared_tick_to_trade_params(section, &pipeline_result, &alloc_stats);

    Ok(())
}

fn start_pipeline_input_sender(
    packets: Vec<Vec<u8>>,
    pattern: SendPattern,
    rx_addr: SocketAddr,
    tx_sock: UdpSocket,
    done_flag: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));

        match pattern {
            SendPattern::InOrder => {
                for pkt in packets.iter() {
                    tx_sock.send_to(pkt.as_slice(), rx_addr).expect("send_to");
                }
            }
            SendPattern::ReorderedSegments => {
                let n = packets.len();
                for block in (0..n).step_by(REORDER_CYCLE) {
                    for i in 0..5 {
                        tx_sock
                            .send_to(packets[block + i].as_slice(), rx_addr)
                            .expect("send_to");
                    }
                    for i in 6..REORDER_CYCLE {
                        tx_sock
                            .send_to(packets[block + i].as_slice(), rx_addr)
                            .expect("send_to");
                    }
                    tx_sock
                        .send_to(packets[block + 5].as_slice(), rx_addr)
                        .expect("send_to");
                }
            }
        }
        // Give the pipeline a chance to drain packets already queued in the socket before
        // signalling shutdown; the sender thread still owns `packets` until after the
        // pipeline snapshots allocation stats below.
        thread::sleep(Duration::from_millis(50));
        done_flag.store(true, Ordering::Release);
        thread::sleep(Duration::from_millis(200));
    })
}
