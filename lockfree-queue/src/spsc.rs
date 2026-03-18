use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Error returned when [`SpscQueue::new`] is called with a capacity that is not a power of two.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("SPSC queue capacity must be a power of two (got {capacity})")]
pub struct NonPowerOfTwoCapacity {
    pub capacity: usize,
}

fn is_power_of_two(capacity: usize) -> bool {
    capacity > 0 && (capacity & (capacity - 1)) == 0
}

/// Shared state for the SPSC queue (buffer, head/tail indices).
struct SpscInner<T: Default> {
    buffer: Vec<UnsafeCell<T>>,
    /// `capacity - 1` for power-of-two ring indexing: `(idx + 1) & head_tail_mask`.
    head_tail_mask: u32,
    size: Arc<AtomicU32>,
}

/// SPSC queue.
pub struct SpscQueue<T: Default> {
    inner: Arc<SpscInner<T>>,
}

/// Producer handle for an SPSC queue. Only one producer may exist per queue.
pub struct SpscProducer<T: Default> {
    inner: Arc<SpscInner<T>>,
    tail: AtomicU32,
}

/// Consumer handle for an SPSC queue. Only one consumer may exist per queue.
pub struct SpscConsumer<T: Default> {
    inner: Arc<SpscInner<T>>,
    head: AtomicU32,
}

impl<T: Default> SpscQueue<T> {
    /// Creates a new SPSC queue with the given capacity (fixed-size ring buffer).
    ///
    /// `capacity` must be a power of two (e.g. 1, 2, 4, 8, …) and fit in `u32`.
    pub fn new(capacity: usize) -> Result<Self, NonPowerOfTwoCapacity> {
        if !is_power_of_two(capacity) {
            return Err(NonPowerOfTwoCapacity { capacity });
        }
        if capacity > u32::MAX as usize {
            return Err(NonPowerOfTwoCapacity { capacity });
        }
        let head_tail_mask = (capacity - 1) as u32;
        let data: Vec<UnsafeCell<T>> = (0..capacity)
            .map(|_| UnsafeCell::new(T::default()))
            .collect();
        Ok(Self {
            inner: Arc::new(SpscInner {
                buffer: data,
                head_tail_mask,
                size: Arc::new(AtomicU32::new(0)),
            }),
        })
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    /// Splits the queue into a single producer and single consumer handle.
    pub fn split(self) -> (SpscProducer<T>, SpscConsumer<T>) {
        (
            SpscProducer {
                inner: self.inner.clone(),
                tail: AtomicU32::new(0),
            },
            SpscConsumer {
                inner: self.inner.clone(),
                head: AtomicU32::new(0),
            },
        )
    }
}

impl<T: Default> SpscProducer<T> {
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.inner.size.load(Ordering::SeqCst) as usize == self.capacity()
    }

    /// Tries to push without blocking. Returns `Ok(())` on success, `Err(value)` if full.
    pub fn try_push(&mut self, mut value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }
        let tail = self.tail.load(Ordering::SeqCst);

        let cell = &self.inner.buffer[tail as usize];
        let ptr = cell.get();
        unsafe {
            ptr.swap(&mut value);
        }

        self.inner.size.fetch_add(1, Ordering::SeqCst);
        let new_tail = tail.wrapping_add(1) & self.inner.head_tail_mask;

        self.tail
            .compare_exchange(tail, new_tail, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to tail in SpscProducer"));

        Ok(())
    }

    /// Pushes a value into the queue. Blocks if full, until space is available.
    pub fn push_blocking(&mut self, mut value: T) {
        while let Err(err_value) = self.try_push(value) {
            value = err_value;
        }
    }
}

impl<T: Default> SpscConsumer<T> {
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.size.load(Ordering::SeqCst) == 0
    }

    /// Tries to pop without blocking. Returns `Some(value)` or `None` if empty.
    pub fn try_pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let head = self.head.load(Ordering::SeqCst);

        // remove item from the buffer
        let mut value = T::default();

        let cell = &self.inner.buffer[head as usize];
        let ptr = cell.get();
        unsafe {
            ptr.swap(&mut value);
        }

        self.inner.size.fetch_sub(1, Ordering::SeqCst);
        let new_head = head.wrapping_add(1) & self.inner.head_tail_mask;

        self.head
            .compare_exchange(head, new_head, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to head in SpscConsumer"));

        Some(value)
    }

    /// Pops a value from the queue. Blocks until an item is available.
    pub fn pop_blocking(&mut self) -> Option<T> {
        loop {
            if let Some(value) = self.try_pop() {
                return Some(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_i32(capacity: usize) -> (SpscProducer<i32>, SpscConsumer<i32>) {
        SpscQueue::new(capacity).unwrap().split()
    }

    // --- constructor & capacity ---

    #[test]
    fn new_rejects_zero() {
        assert!(matches!(
            SpscQueue::<i32>::new(0),
            Err(NonPowerOfTwoCapacity { capacity: 0 })
        ));
    }

    #[test]
    fn new_rejects_non_power_of_two() {
        for &bad in &[3usize, 5, 6, 7, 9, 15, 17, 1023] {
            assert!(
                matches!(SpscQueue::<i32>::new(bad), Err(NonPowerOfTwoCapacity { capacity: c }) if c == bad),
                "capacity {} should be rejected",
                bad
            );
        }
    }

    #[test]
    fn new_accepts_powers_of_two_including_one() {
        for &cap in &[1usize, 2, 4, 8, 16, 32, 256, 4096] {
            let q = SpscQueue::<i32>::new(cap).unwrap();
            assert_eq!(q.capacity(), cap, "capacity {}", cap);
        }
    }

    #[test]
    fn split_exposes_same_capacity_and_initial_state() {
        let q = SpscQueue::<i32>::new(64).unwrap();
        assert_eq!(q.capacity(), 64);
        let (prod, cons) = q.split();
        assert_eq!(prod.capacity(), 64);
        assert_eq!(cons.capacity(), 64);
        assert!(cons.is_empty());
        assert!(!prod.is_full());
    }

    // --- basic try_push / try_pop ---

    #[test]
    fn try_push_one_try_pop_one() {
        let (mut prod, mut cons) = split_i32(8);
        assert_eq!(prod.try_push(100), Ok(()));
        assert_eq!(cons.try_pop(), Some(100));
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn try_pop_empty_queue_returns_none() {
        let (mut _prod, mut cons) = split_i32(4);
        assert_eq!(cons.try_pop(), None);
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn fifo_order_multiple_items() {
        let (mut prod, mut cons) = split_i32(16);
        for i in 0..10 {
            assert_eq!(prod.try_push(i), Ok(()));
        }
        for i in 0..10 {
            assert_eq!(cons.try_pop(), Some(i));
        }
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn try_push_full_returns_err_and_preserves_value() {
        let (mut prod, mut cons) = split_i32(4);
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(prod.try_push(2), Ok(()));
        assert_eq!(prod.try_push(3), Ok(()));
        assert_eq!(prod.try_push(4), Ok(()));
        assert_eq!(prod.try_push(999), Err(999));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(999), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
    }

    #[test]
    fn try_push_succeeds_after_partial_drain() {
        let (mut prod, mut cons) = split_i32(2);
        assert_eq!(prod.try_push(10), Ok(()));
        assert_eq!(prod.try_push(20), Ok(()));
        assert_eq!(prod.try_push(30), Err(30));
        assert_eq!(cons.try_pop(), Some(10));
        assert_eq!(prod.try_push(30), Ok(()));
        assert_eq!(cons.try_pop(), Some(20));
        assert_eq!(cons.try_pop(), Some(30));
    }

    // --- wraparound (index & slot reuse) ---

    #[test]
    fn wrap_single_slot_capacity_one() {
        let (mut prod, mut cons) = split_i32(1);
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(prod.try_push(2), Err(2));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(2), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
    }

    #[test]
    fn wrap_fill_drain_then_reuse_slots() {
        let (mut prod, mut cons) = split_i32(2);
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(prod.try_push(2), Ok(()));
        assert!(queue_is_full(&mut prod, &mut cons));
        assert_eq!(cons.try_pop(), Some(1));
        assert!(!prod.is_full() && !cons.is_empty());
        assert_eq!(prod.try_push(3), Ok(()));
        assert!(queue_is_full(&mut prod, &mut cons));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), Some(3));
        assert!(queue_is_empty(&mut prod, &mut cons));
        assert_eq!(cons.try_pop(), None);
    }

    fn queue_is_empty(prod: &mut SpscProducer<i32>, cons: &mut SpscConsumer<i32>) -> bool {
        !prod.is_full() && cons.is_empty()
    }

    fn queue_is_full(prod: &mut SpscProducer<i32>, cons: &mut SpscConsumer<i32>) -> bool {
        prod.is_full() && !cons.is_empty()
    }

    #[test]
    fn wrap_many_cycles_small_buffer() {
        let (mut prod, mut cons) = split_i32(4);
        for round in 0..200 {
            for k in 0..4 {
                assert_eq!(prod.try_push(round * 4 + k), Ok(()));
            }
            assert!(prod.is_full());
            for k in 0..4 {
                assert_eq!(cons.try_pop(), Some(round * 4 + k));
            }
            // Advance tail position
            assert!(queue_is_empty(&mut prod, &mut cons));
            assert_eq!(prod.try_push(123), Ok(()));
            assert!(!prod.is_full() && !cons.is_empty());
            assert_eq!(cons.try_pop(), Some(123));
            assert!(queue_is_empty(&mut prod, &mut cons));
        }
    }

    #[test]
    fn wrap_interleaved_push_pop_advances_indices() {
        let (mut prod, mut cons) = split_i32(8);
        for i in 0..50 {
            assert_eq!(prod.try_push(i), Ok(()));
            assert_eq!(cons.try_pop(), Some(i));
        }
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn wrap_staggered_producer_consumer() {
        let (mut prod, mut cons) = split_i32(4);
        assert_eq!(prod.try_push(0), Ok(()));
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(cons.try_pop(), Some(0));
        assert_eq!(prod.try_push(2), Ok(()));
        assert_eq!(prod.try_push(3), Ok(()));
        assert_eq!(prod.try_push(4), Ok(()));
        assert_eq!(prod.try_push(5), Err(5));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(5), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), Some(3));
        assert_eq!(cons.try_pop(), Some(4));
        assert_eq!(cons.try_pop(), Some(5));
    }

    // --- blocking push/pop, is_empty / is_full ---

    #[test]
    fn push_pop_blocking() {
        let (mut prod, mut cons) = split_i32(4);
        prod.push_blocking(1);
        prod.push_blocking(2);
        assert_eq!(cons.pop_blocking(), Some(1));
        assert_eq!(cons.pop_blocking(), Some(2));
    }

    // --- non-Copy / heap types ---

    #[test]
    fn try_push_try_pop_string_roundtrip() {
        let (mut prod, mut cons) = SpscQueue::<String>::new(4).unwrap().split();
        assert_eq!(prod.try_push("hello".into()), Ok(()));
        assert_eq!(cons.try_pop().as_deref(), Some("hello"));
    }

    #[test]
    fn string_wrap_and_order() {
        let (mut prod, mut cons) = SpscQueue::<String>::new(2).unwrap().split();
        prod.try_push("a".into()).unwrap();
        prod.try_push("b".into()).unwrap();
        assert_eq!(cons.try_pop().as_deref(), Some("a"));
        prod.try_push("c".into()).unwrap();
        assert_eq!(cons.try_pop().as_deref(), Some("b"));
        assert_eq!(cons.try_pop().as_deref(), Some("c"));
    }
}
