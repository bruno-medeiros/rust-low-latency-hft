use std::cell::UnsafeCell;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[repr(align(64))]
struct CacheLinePadded<T> {
    value: T,
}

impl<T> CacheLinePadded<T> {
    const fn new(value: T) -> Self {
        Self { value }
    }
}

impl Deref for CacheLinePadded<AtomicU32> {
    type Target = AtomicU32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

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
struct SpscInner<T> {
    buffer: Vec<UnsafeCell<Option<T>>>,
    /// `capacity - 1` for power-of-two ring indexing: `(idx + 1) & head_tail_mask`.
    head_tail_mask: u32,
    tail: CacheLinePadded<AtomicU32>,
    head: CacheLinePadded<AtomicU32>,
}

// SAFETY: Slots are written only from the producer side and read only from the
// consumer side ([`SpscProducer`] / [`SpscConsumer`]), at most one thread each.
unsafe impl<T: Send> Sync for SpscInner<T> {}

/// SPSC queue.
pub struct SpscQueue<T> {
    inner: Arc<SpscInner<T>>,
}

/// Producer handle for an SPSC queue. Only one producer may exist per queue.
pub struct SpscProducer<T> {
    inner: Arc<SpscInner<T>>,
}

/// Consumer handle for an SPSC queue. Only one consumer may exist per queue.
pub struct SpscConsumer<T> {
    inner: Arc<SpscInner<T>>,
}

impl<T> SpscQueue<T> {
    /// Creates a new SPSC queue with the given capacity (fixed-size ring buffer).
    ///
    /// `capacity` must be a power of two and fit in `u32`.
    pub fn new(capacity: usize) -> Result<Self, NonPowerOfTwoCapacity> {
        if !is_power_of_two(capacity) {
            return Err(NonPowerOfTwoCapacity { capacity });
        }
        if capacity > u32::MAX as usize {
            return Err(NonPowerOfTwoCapacity { capacity });
        }
        let head_tail_mask = (capacity - 1) as u32;
        let data: Vec<UnsafeCell<Option<T>>> =
            (0..capacity).map(|_| UnsafeCell::new(None)).collect();
        Ok(Self {
            inner: Arc::new(SpscInner {
                buffer: data,
                head_tail_mask,
                tail: CacheLinePadded::new(AtomicU32::new(0)),
                head: CacheLinePadded::new(AtomicU32::new(0)),
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
            },
            SpscConsumer {
                inner: self.inner.clone(),
            },
        )
    }
}

impl<T> SpscProducer<T> {
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        let tail = self.inner.tail.load(Ordering::Relaxed);
        let head = self.inner.head.load(Ordering::Acquire);
        (tail + 1) & self.inner.head_tail_mask == head
    }

    /// Tries to push without blocking. Returns `Ok(())` on success, `Err(value)` if full.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }
        let tail = self.inner.tail.load(Ordering::Relaxed);

        let cell = &self.inner.buffer[tail as usize];
        unsafe {
            *cell.get() = Some(value);
        }

        let new_tail = tail.wrapping_add(1) & self.inner.head_tail_mask;

        #[cfg(debug_assertions)]
        self.inner
            .tail
            .compare_exchange(tail, new_tail, Ordering::Release, Ordering::Relaxed)
            .expect("Concurrent modification to tail in SpscProducer");

        #[cfg(not(debug_assertions))]
        self.inner.tail.store(new_tail, Ordering::Release);

        Ok(())
    }

    /// Pushes a value into the queue. Blocks if full, until space is available.
    pub fn push_blocking(&mut self, mut value: T) {
        while let Err(err_value) = self.try_push(value) {
            value = err_value;
        }
    }
}

impl<T> SpscConsumer<T> {
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.head.load(Ordering::Relaxed) == self.inner.tail.load(Ordering::Acquire)
    }

    /// Tries to pop without blocking. Returns `Some(value)` or `None` if empty.
    pub fn try_pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let head = self.inner.head.load(Ordering::Relaxed);

        let cell = &self.inner.buffer[head as usize];
        let value = unsafe {
            (*cell.get())
                .take()
                .expect("pop: slot empty despite size > 0")
        };

        let new_head = head.wrapping_add(1) & self.inner.head_tail_mask;

        #[cfg(debug_assertions)]
        self.inner
            .head
            .compare_exchange(head, new_head, Ordering::Release, Ordering::Relaxed)
            .expect("Concurrent modification to head in SpscConsumer");

        #[cfg(not(debug_assertions))]
        self.inner.head.store(new_head, Ordering::Release);

        Some(value)
    }

    /// Pops a value from the queue. Blocks until an item is available.
    pub fn pop_blocking(&mut self) -> T {
        loop {
            if let Some(value) = self.try_pop() {
                return value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

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
        assert!(!prod.is_full());
        assert!(!cons.is_empty());
        assert_eq!(cons.try_pop(), Some(100));
        assert!(!prod.is_full() && cons.is_empty());
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn try_pop_empty_queue_returns_none() {
        let (_prod, mut cons) = split_i32(4);
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
        assert_eq!(prod.try_push(999), Err(999));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(999), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
    }

    // --- wraparound (index & slot reuse) ---

    #[test]
    fn wrap_single_slot_capacity_one() {
        let (mut prod, mut cons) = split_i32(2);
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(prod.try_push(2), Err(2));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(2), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
    }

    #[test]
    fn wrap_fill_drain_then_reuse_slots() {
        let (mut prod, mut cons) = split_i32(4);
        assert_eq!(prod.try_push(1), Ok(()));
        assert_eq!(prod.try_push(2), Ok(()));
        assert_eq!(prod.try_push(3), Ok(()));
        assert!(queue_is_full(&mut prod, &mut cons));
        assert_eq!(cons.try_pop(), Some(1));
        assert!(!prod.is_full() && !cons.is_empty());
        assert_eq!(prod.try_push(4), Ok(()));
        assert!(queue_is_full(&mut prod, &mut cons));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), Some(3));
        assert_eq!(cons.try_pop(), Some(4));
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
            for k in 0..4 - 1 {
                assert_eq!(prod.try_push(round * 4 + k), Ok(()));
            }
            assert!(prod.is_full());
            for k in 0..4 - 1 {
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
        assert_eq!(prod.try_push(4), Err(4));
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(prod.try_push(4), Ok(()));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), Some(3));
        assert_eq!(cons.try_pop(), Some(4));
    }

    // --- blocking push/pop, is_empty / is_full ---

    #[test]
    fn push_pop_blocking_over_capacity_with_consumer_thread() {
        let (mut prod, mut cons) = split_i32(4);

        let consumer = thread::spawn(move || {
            for i in 1..=40 {
                assert_eq!(cons.pop_blocking(), i);
            }
        });

        for i in 1..=40 {
            prod.push_blocking(i);
        }

        consumer.join().expect("consumer thread panicked");
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
        let (mut prod, mut cons) = SpscQueue::<String>::new(4).unwrap().split();
        prod.try_push("a".into()).unwrap();
        prod.try_push("b".into()).unwrap();
        assert_eq!(cons.try_pop().as_deref(), Some("a"));
        prod.try_push("c".into()).unwrap();
        assert_eq!(cons.try_pop().as_deref(), Some("b"));
        assert_eq!(cons.try_pop().as_deref(), Some("c"));
    }
}
