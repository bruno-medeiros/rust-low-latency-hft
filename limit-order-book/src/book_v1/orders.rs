use crate::book_v1::book::OrderSlot;
use crate::order::Order;
use crate::types::OrderId;

#[derive(Debug)]
pub struct BookOrders {
    pub orders: Box<[Option<OrderSlot>]>,
    pub order_count: u64,
}

impl BookOrders {
    pub fn new(order_capacity: OrderId) -> BookOrders {
        let orders = (0..order_capacity)
            .map(|_| None)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            orders,
            order_count: 0,
        }
    }

    pub fn order(&self, id: OrderId) -> Option<&Order> {
        self.orders[id as usize].as_ref().map(|os| &os.order)
    }

    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    pub fn existing_order(&mut self, order_id: OrderId) -> &mut OrderSlot {
        self.orders[order_id as usize]
            .as_mut()
            .expect(format!("order {} exists", order_id).as_str())
    }

    pub fn add_order(&mut self, order_id: OrderId, order: Order) -> &mut OrderSlot {
        self.order_count += 1;
        assert!(self.orders[order_id as usize].is_none());
        self.orders[order_id as usize].insert(OrderSlot {
            next: None,
            prev: None,
            order,
        })
    }

    pub fn remove_order(&mut self, order_id: OrderId) -> OrderSlot {
        let order_slot = self.orders[order_id as usize].take().unwrap();

        self.order_count -= 1;
        if let Some(prev) = order_slot.prev {
            self.existing_order(prev).next = order_slot.next;
        }
        if let Some(next) = order_slot.next {
            self.existing_order(next).prev = order_slot.prev;
        }
        order_slot
    }
}
