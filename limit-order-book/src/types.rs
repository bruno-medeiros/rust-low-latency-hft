pub type OrderId = u64;

/// Price in ticks (smallest price increment). Fixed-point integer to avoid floating-point issues.
pub type Price = u64;

/// Quantity in lots (smallest tradeable unit).
pub type Qty = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}
