//! ITCH-style message types (zero-copy where possible).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// ITCH-style decoded message (borrows from input buffer).
#[derive(Debug, PartialEq)]
pub enum ItchMessage<'a> {
    SystemEvent { text: &'a str },
    AddOrder {
        oid: u64,
        side: Side,
        qty: u32,
        price: u32,
        symbol: &'a str,
    },
    OrderExecuted { oid: u64, qty: u32 },
    OrderCanceled { oid: u64, qty: u32 },
    // TODO: review what this itch message will be needed for.
    Trade { oid: u64, side: Side, qty: u32, price: u32 },
}
