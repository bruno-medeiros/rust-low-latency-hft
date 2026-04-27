//! End-to-end UDP reorder: sender permutes MoldUDP64 datagrams; pipeline drains in seq order.

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use limit_order_book::LimitOrderBookV1;
use market_data_handler::itch::Side;
use market_data_handler::itch::encode;
use market_data_handler::mold_udp64::{SESSION_LEN, encode_packet};
use market_data_handler::{MarketHandlerPipeline, PipelineConfig, PipelineError, PipelineResult};

const N: usize = 50;
const SESSION: &[u8; SESSION_LEN] = b"OOOTEST   ";

fn make_packets() -> Vec<Vec<u8>> {
    (0..N)
        .map(|i| {
            let seq = i as u64;
            let itch = encode::encode_add_order(seq + 1, Side::Buy, 1, 100, "SYM");
            encode_packet(SESSION, seq, &[&itch])
        })
        .collect()
}

fn make_multi_message_packets() -> Vec<Vec<u8>> {
    let first = encode::encode_add_order(1, Side::Buy, 1, 100, "SYM");
    let second = encode::encode_add_order(2, Side::Buy, 1, 100, "SYM");
    let third = encode::encode_add_order(3, Side::Buy, 1, 100, "SYM");

    vec![
        encode_packet(SESSION, 0, &[&first, &second]),
        encode_packet(SESSION, 2, &[&third]),
    ]
}

fn run_with_order(
    order: Vec<usize>,
    reorder_window: usize,
) -> Result<PipelineResult, PipelineError> {
    let packets = make_packets();
    run_packets_with_order(packets, order, reorder_window)
}

fn run_packets_with_order(
    packets: Vec<Vec<u8>>,
    order: Vec<usize>,
    reorder_window: usize,
) -> Result<PipelineResult, PipelineError> {
    let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = rx.local_addr().unwrap();
    let tx = UdpSocket::bind("127.0.0.1:0").unwrap();

    let done = Arc::new(AtomicBool::new(false));
    let done_tx = done.clone();

    thread::spawn(move || {
        for &i in &order {
            tx.send_to(&packets[i], addr).unwrap();
        }
        thread::sleep(Duration::from_millis(30));
        done_tx.store(true, Ordering::Release);
    });

    let config = PipelineConfig {
        price_range: (1, 1000),
        order_capacity: 64,
        core_pinning_enabled: false,
        pin_core: 0,
        first_seq: 0,
        reorder_window,
        read_timeout_ms: Some(100),
    };

    let book = LimitOrderBookV1::new(config.price_range, config.order_capacity as usize);
    MarketHandlerPipeline::from_config(config).run(rx, done, book)
}

#[test]
fn pipeline_recovers_shuffled_datagrams() {
    let order: Vec<usize> = (0..N).rev().collect();
    let result = run_with_order(order, 64).expect("reorder pipeline completes");

    assert_eq!(result.messages_decoded, N as u64);
    assert!(
        result.reorder_stats.reorder_ahead_arrivals > 0,
        "reverse send should buffer at least one ahead-of-watermark datagram"
    );
}

#[test]
fn reorder_window_zero_clamps_in_order_feed_completes() {
    let order: Vec<usize> = (0..N).collect();
    let result = run_with_order(order, 0).expect("in-order feed with clamped window");

    assert_eq!(result.messages_decoded, N as u64);
    assert_eq!(result.reorder_stats.reorder_ahead_arrivals, 0);
}

#[test]
fn reorder_window_zero_reverse_fails_window_exceeded() {
    let order: Vec<usize> = (0..N).rev().collect();
    let err = match run_with_order(order, 0) {
        Err(e) => e,
        Ok(_) => panic!("window 1 cannot hold reverse-ordered feed"),
    };

    assert!(matches!(err, PipelineError::ReorderWindowExceeded(_)));
}

#[test]
fn pipeline_accepts_mold_packets_with_multiple_messages() {
    let packets = make_multi_message_packets();
    let result =
        run_packets_with_order(packets, vec![0, 1], 8).expect("multi-message feed completes");

    assert_eq!(result.packets_received, 2);
    assert_eq!(result.messages_decoded, 3);
}
