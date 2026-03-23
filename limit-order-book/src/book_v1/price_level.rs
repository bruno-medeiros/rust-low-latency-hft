use crate::book_v1::book::OrderSlot;
use crate::types::{OrderSlabKey, Price, Qty};

#[derive(Debug)]
pub struct PriceLevel {
    pub total_qty: Qty,
    pub price: Price,
    pub order_head: Option<OrderSlabKey>,
    pub order_tail: Option<OrderSlabKey>,
}

impl PriceLevel {
    pub fn new(price: Price) -> Self {
        Self {
            total_qty: 0,
            price,
            order_head: None,
            order_tail: None,
        }
    }

    pub fn append_order(&mut self, key: OrderSlabKey, qty: Qty) -> Option<OrderSlabKey> {
        self.total_qty += qty;
        match self.order_tail {
            None => {
                self.order_head = Some(key);
                self.order_tail = Some(key);
                None
            }
            Some(old_tail) => {
                self.order_tail = Some(key);
                Some(old_tail)
            }
        }
    }

    pub fn front(&self) -> Option<OrderSlabKey> {
        self.order_head
    }

    pub fn remove(&mut self, key: OrderSlabKey, order_slot: &OrderSlot) {
        if self.order_head == Some(key) {
            self.order_head = order_slot.next;
        }
        if self.order_tail == Some(key) {
            self.order_tail = order_slot.prev;
        }
        self.total_qty -= order_slot.order.remaining_qty;
    }

    pub fn reduce_qty(&mut self, amount: Qty) {
        self.total_qty -= amount;
    }

    pub fn is_empty(&self) -> bool {
        self.order_head.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::PriceLevel;
    use crate::book_v1::book::OrderSlot;
    use crate::order::Order;
    use crate::types::{OrderSlabKey, Side};

    fn order_slot(
        id: u64,
        price: u64,
        remaining_qty: u64,
        prev: Option<OrderSlabKey>,
        next: Option<OrderSlabKey>,
    ) -> OrderSlot {
        OrderSlot {
            next,
            prev,
            order: Order {
                id,
                side: Side::Buy,
                price,
                qty: remaining_qty,
                remaining_qty,
                sequence: 0,
            },
        }
    }

    #[test]
    fn push_sets_head_tail_and_total_qty() {
        let mut level = PriceLevel::new(101);

        level.append_order(0, 7);
        assert_eq!(level.front(), Some(0));
        assert_eq!(level.order_head, Some(0));
        assert_eq!(level.order_tail, Some(0));
        assert_eq!(level.total_qty, 7);
        assert!(!level.is_empty());

        level.append_order(1, 5);
        assert_eq!(level.front(), Some(0));
        assert_eq!(level.order_head, Some(0));
        assert_eq!(level.order_tail, Some(1));
        assert_eq!(level.total_qty, 12);
    }

    #[test]
    fn remove_head_updates_front_and_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(0, 7);
        level.append_order(1, 5);

        let head = order_slot(11, 101, 7, None, Some(1));
        level.remove(0, &head);

        assert_eq!(level.front(), Some(1));
        assert_eq!(level.order_head, Some(1));
        assert_eq!(level.order_tail, Some(1));
        assert_eq!(level.total_qty, 5);
        assert!(!level.is_empty());
    }

    #[test]
    fn remove_tail_updates_tail_and_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(0, 7);
        level.append_order(1, 5);

        let tail = order_slot(12, 101, 5, Some(0), None);
        level.remove(1, &tail);

        assert_eq!(level.front(), Some(0));
        assert_eq!(level.order_head, Some(0));
        assert_eq!(level.order_tail, Some(0));
        assert_eq!(level.total_qty, 7);
        assert!(!level.is_empty());
    }

    #[test]
    fn remove_last_order_clears_level() {
        let mut level = PriceLevel::new(101);
        level.append_order(0, 7);

        let only = order_slot(11, 101, 7, None, None);
        level.remove(0, &only);

        assert_eq!(level.front(), None);
        assert_eq!(level.order_head, None);
        assert_eq!(level.order_tail, None);
        assert_eq!(level.total_qty, 0);
        assert!(level.is_empty());
    }

    #[test]
    fn remove_middle_order_only_reduces_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(0, 7);
        level.append_order(1, 5);
        level.append_order(2, 3);

        let middle = order_slot(12, 101, 5, Some(0), Some(2));
        level.remove(1, &middle);

        assert_eq!(level.front(), Some(0));
        assert_eq!(level.order_head, Some(0));
        assert_eq!(level.order_tail, Some(2));
        assert_eq!(level.total_qty, 10);
        assert!(!level.is_empty());
    }
}
