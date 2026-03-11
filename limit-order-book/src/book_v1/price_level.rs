use crate::book_v1::book::OrderSlot;
use crate::types::{OrderId, Price, Qty};

#[derive(Debug)]
pub struct PriceLevel {
    pub total_qty: Qty,
    pub price: Price,
    pub order_head: Option<OrderId>,
    pub order_tail: Option<OrderId>,
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

    pub fn append_order(&mut self, order_id: OrderId, qty: Qty) -> Option<OrderId> {
        self.total_qty += qty;
        match self.order_tail {
            None => {
                self.order_head = Some(order_id);
                self.order_tail = Some(order_id);
                None
            }
            Some(old_tail) => {
                self.order_tail = Some(order_id);
                Some(old_tail)
            }
        }
    }

    pub fn front(&self) -> Option<OrderId> {
        self.order_head
    }

    pub fn remove(&mut self, order_slot: &mut OrderSlot) {
        if let Some(head) = self.order_head
            && head == order_slot.order.id
        {
            self.order_head = order_slot.next;
        }
        if let Some(tail) = self.order_tail
            && tail == order_slot.order.id
        {
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
    use crate::types::Side;

    fn order_slot(
        id: u64,
        price: u64,
        remaining_qty: u64,
        prev: Option<u64>,
        next: Option<u64>,
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

        level.append_order(11, 7);
        assert_eq!(level.front(), Some(11));
        assert_eq!(level.order_head, Some(11));
        assert_eq!(level.order_tail, Some(11));
        assert_eq!(level.total_qty, 7);
        assert!(!level.is_empty());

        level.append_order(12, 5);
        assert_eq!(level.front(), Some(11));
        assert_eq!(level.order_head, Some(11));
        assert_eq!(level.order_tail, Some(12));
        assert_eq!(level.total_qty, 12);
    }

    #[test]
    fn remove_head_updates_front_and_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(11, 7);
        level.append_order(12, 5);

        let mut head = order_slot(11, 101, 7, None, Some(12));
        level.remove(&mut head);

        assert_eq!(level.front(), Some(12));
        assert_eq!(level.order_head, Some(12));
        assert_eq!(level.order_tail, Some(12));
        assert_eq!(level.total_qty, 5);
        assert!(!level.is_empty());
    }

    #[test]
    fn remove_tail_updates_tail_and_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(11, 7);
        level.append_order(12, 5);

        let mut tail = order_slot(12, 101, 5, Some(11), None);
        level.remove(&mut tail);

        assert_eq!(level.front(), Some(11));
        assert_eq!(level.order_head, Some(11));
        assert_eq!(level.order_tail, Some(11));
        assert_eq!(level.total_qty, 7);
        assert!(!level.is_empty());
    }

    #[test]
    fn remove_last_order_clears_level() {
        let mut level = PriceLevel::new(101);
        level.append_order(11, 7);

        let mut only = order_slot(11, 101, 7, None, None);
        level.remove(&mut only);

        assert_eq!(level.front(), None);
        assert_eq!(level.order_head, None);
        assert_eq!(level.order_tail, None);
        assert_eq!(level.total_qty, 0);
        assert!(level.is_empty());
    }

    #[test]
    fn remove_middle_order_only_reduces_total_qty() {
        let mut level = PriceLevel::new(101);
        level.append_order(11, 7);
        level.append_order(12, 5);
        level.append_order(13, 3);

        let mut middle = order_slot(12, 101, 5, Some(11), Some(13));
        level.remove(&mut middle);

        assert_eq!(level.front(), Some(11));
        assert_eq!(level.order_head, Some(11));
        assert_eq!(level.order_tail, Some(13));
        assert_eq!(level.total_qty, 10);
        assert!(!level.is_empty());
    }
}
