//! Minimal top-of-book cross-spread quoter strategy stub.
//!
//! Decision rule: if best_bid + 1 < best_ask, post a passive buy at best_bid + 1 for qty 1.
//! If already resting and market moved away, cancel the resting order.
//!
//! Intentionally trivial — the goal is a realistic hot-path shape (top-of-book read,
//! conditional branch, outbound encode) not a profitable alpha model.

use crate::outbound::OutboundBuf;
use limit_order_book::LimitOrderBook;
use limit_order_book::types::Side;

pub struct QuoterState {
    /// Order ID of any currently-resting passive buy, if present.
    resting_oid: Option<u64>,
    /// Monotonically increasing OID counter for emitted orders.
    next_oid: u64,
}

impl QuoterState {
    pub fn new() -> Self {
        Self { resting_oid: None, next_oid: 1 }
    }

    /// Called after every book update. Writes an outbound order into `out` and returns
    /// `true` if an order (new or cancel) should be sent, `false` if nothing to send.
    ///
    /// Uses a concrete type — no dyn dispatch — so the compiler can inline this fully
    /// into the pipeline hot loop.
    #[inline]
    pub fn on_book_update<B: LimitOrderBook>(
        &mut self,
        book: &B,
        out: &mut OutboundBuf,
    ) -> bool {
        let bid = book.best_bid();
        let ask = book.best_ask();

        match (bid, ask) {
            (Some((bid_price, _)), Some((ask_price, _))) if bid_price + 1 < ask_price => {
                // Spread wide enough — quote passively one tick above best bid.
                let target_price = (bid_price + 1) as u32;
                if let Some(oid) = self.resting_oid {
                    // Cancel stale resting order first (in production this would be
                    // a modify/replace, but cancel+new is correct for this demo).
                    out.encode_cancel_order(oid);
                    self.resting_oid = None;
                    return true;
                }
                let oid = self.next_oid;
                self.next_oid += 1;
                out.encode_new_order(oid, Side::Buy, target_price, 1);
                self.resting_oid = Some(oid);
                true
            }
            _ => {
                // No spread opportunity or no two-sided market — cancel any resting order.
                if let Some(oid) = self.resting_oid.take() {
                    out.encode_cancel_order(oid);
                    return true;
                }
                false
            }
        }
    }

    pub fn resting_oid(&self) -> Option<u64> {
        self.resting_oid
    }
}

impl Default for QuoterState {
    fn default() -> Self {
        Self::new()
    }
}