use std::collections::HashMap;

use slab::Slab;

use crate::book_v1::book::OrderSlot;
use crate::order::Order;
use crate::types::{OrderId, OrderKey};

#[derive(Debug)]
pub struct BookOrders {
    slab: Slab<OrderSlot>,
    id_to_key: HashMap<OrderId, OrderKey>,
}

impl BookOrders {
    pub fn new(capacity: usize) -> BookOrders {
        Self {
            slab: Slab::with_capacity(capacity),
            id_to_key: HashMap::with_capacity(capacity),
        }
    }

    pub fn order(&self, id: OrderId) -> Option<&Order> {
        self.id_to_key.get(&id).map(|&key| &self.slab[key].order)
    }

    pub fn order_key(&self, id: OrderId) -> Option<OrderKey> {
        self.id_to_key.get(&id).copied()
    }

    pub fn order_count(&self) -> u64 {
        self.slab.len() as u64
    }

    pub fn slot(&self, key: OrderKey) -> &OrderSlot {
        &self.slab[key]
    }

    pub fn slot_mut(&mut self, key: OrderKey) -> &mut OrderSlot {
        &mut self.slab[key]
    }

    /// Inserts an order into the slab and records the external-ID → slab-key mapping.
    /// Returns the slab key for the newly inserted slot.
    pub fn add_order(&mut self, order: Order) -> OrderKey {
        let order_id = order.id;
        let key = self.slab.insert(OrderSlot {
            next: None,
            prev: None,
            order,
        });
        self.id_to_key.insert(order_id, key);
        key
    }

    /// Removes an order by slab key. Fixes up linked-list prev/next pointers.
    pub fn remove_by_key(&mut self, key: OrderKey) -> OrderSlot {
        let order_id = self.slab[key].order.id;
        self.id_to_key
            .remove(&order_id)
            .unwrap_or_else(|| panic!("order {order_id} exists"));
        self.remove_slot_by_key(key)
    }

    fn remove_slot_by_key(&mut self, key: OrderKey) -> OrderSlot {
        let order_slot = self.slab.remove(key);
        if let Some(prev) = order_slot.prev {
            self.slab[prev].next = order_slot.next;
        }
        if let Some(next) = order_slot.next {
            self.slab[next].prev = order_slot.prev;
        }
        order_slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_option_order_slot_size() {
        println!(
            "Size of Option<OrderSlot>: {}",
            size_of::<Option<OrderSlot>>()
        );
    }
}
