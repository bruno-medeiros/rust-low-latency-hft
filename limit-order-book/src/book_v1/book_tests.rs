use crate::book_tests_common;
use crate::book_v1::book::LimitOrderBookV1;

fn order_book_v1() -> LimitOrderBookV1 {
    LimitOrderBookV1::new((0, 10_000), 1_000)
}

#[test]
fn reject_zero_quantity() {
    book_tests_common::reject_zero_quantity(order_book_v1());
}

#[test]
fn reject_zero_price() {
    book_tests_common::reject_zero_price(order_book_v1());
}

#[test]
fn add_limit_order_rests_in_book() {
    book_tests_common::add_limit_order_rests_in_book(order_book_v1());
}

#[test]
fn add_and_cancel() {
    book_tests_common::add_and_cancel(order_book_v1());
}

#[test]
fn cancel_unknown_order() {
    book_tests_common::cancel_unknown_order(order_book_v1());
}

#[test]
fn reduce_unknown_order() {
    book_tests_common::reduce_unknown_order(order_book_v1());
}

#[test]
fn reduce_order_partial_reduces_resting_qty() {
    book_tests_common::reduce_order_partial_reduces_resting_qty(order_book_v1());
}

#[test]
fn reduce_order_full_reduction_removes_order() {
    book_tests_common::reduce_order_full_reduction_removes_order(order_book_v1());
}

#[test]
fn reduce_order_rejects_zero_quantity() {
    book_tests_common::reduce_order_rejects_zero_quantity(order_book_v1());
}

#[test]
fn cancel_one_of_many_at_same_price() {
    book_tests_common::cancel_one_of_many_at_same_price(order_book_v1());
}

#[test]
fn reject_duplicate_id() {
    book_tests_common::reject_duplicate_id(order_book_v1());
}

#[test]
fn event_sequences_are_monotonic() {
    book_tests_common::event_sequences_are_monotonic(order_book_v1());
}

#[test]
fn best_bid_best_ask() {
    book_tests_common::best_bid_best_ask(order_book_v1());
}

#[test]
fn limit_order_full_match() {
    book_tests_common::limit_order_full_match(order_book_v1());
}

#[test]
fn limit_order_partial_match_passive_remains() {
    book_tests_common::limit_order_partial_match_passive_remains(order_book_v1());
}

#[test]
fn market_order_full_fill() {
    book_tests_common::market_order_full_fill(order_book_v1());
}

#[test]
fn market_order_partial_fill_exhausts_book() {
    book_tests_common::market_order_partial_fill_exhausts_book_and_emits_cancel(order_book_v1());
}

#[test]
fn fifo_priority() {
    book_tests_common::fifo_priority(order_book_v1());
}

#[test]
fn multi_level_sweep() {
    book_tests_common::multi_level_sweep(order_book_v1());
}

#[test]
fn no_match_when_prices_dont_cross() {
    book_tests_common::no_match_when_prices_dont_cross(order_book_v1());
}

#[test]
fn sell_side_matching_hits_best_bid_first() {
    book_tests_common::sell_side_matching_hits_best_bid_first(order_book_v1());
}

#[test]
fn order_preserves_original_qty_after_partial_fill() {
    book_tests_common::order_preserves_original_qty_after_partial_fill(order_book_v1());
}

#[test]
fn market_order_rejects_duplicate_id() {
    book_tests_common::market_order_rejects_duplicate_id(order_book_v1());
}

#[test]
fn market_order_emits_accepted_event() {
    book_tests_common::market_order_emits_accepted_event(order_book_v1());
}

#[test]
fn cancel_front_preserves_fifo_for_remaining() {
    book_tests_common::cancel_front_preserves_fifo_for_remaining(order_book_v1());
}

#[test]
fn sweep_multiple_orders_at_same_level() {
    book_tests_common::sweep_multiple_orders_at_same_level(order_book_v1());
}
