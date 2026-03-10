use std::collections::VecDeque;

use crate::types::{OrderId, Qty};

#[derive(Debug)]
pub(crate) struct PriceLevel {
    pub total_qty: Qty,
    orders: VecDeque<OrderId>,
}

impl PriceLevel {
    pub fn new() -> Self {
        Self {
            total_qty: 0,
            orders: VecDeque::new(),
        }
    }

    pub fn push(&mut self, order_id: OrderId, qty: Qty) {
        self.orders.push_back(order_id);
        self.total_qty += qty;
    }

    pub fn front(&self) -> Option<OrderId> {
        self.orders.front().copied()
    }

    // pub fn pop_front(&mut self) {
    //     self.orders.pop_front();
    // }

    /// TODO: O(n) scan — acceptable for Phase 1, replace with intrusive list or arena later.
    pub fn remove(&mut self, order_id: OrderId, qty: Qty) {
        if let Some(pos) = self.orders.iter().position(|&id| id == order_id) {
            self.orders.remove(pos);
            self.total_qty -= qty;
        } else {
            panic!("Order not found in price level");
        }
    }

    pub fn reduce_qty(&mut self, amount: Qty) {
        self.total_qty -= amount;
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }
}

impl Default for PriceLevel {
    fn default() -> Self {
        Self::new()
    }
}
