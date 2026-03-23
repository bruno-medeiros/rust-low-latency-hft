pub type OrderId = u64;

/// Internal slab key for order slot storage. Not exposed externally.
pub type OrderSlabKey = usize;

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

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}
