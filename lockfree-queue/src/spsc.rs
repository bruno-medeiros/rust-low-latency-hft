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
    /// Pushes a value into the queue. Blocks if full, until space is available.
    pub fn push(&mut self, mut value: T) {
        while let Err(err_value) = self.try_push(value) {
            value = err_value;
        }
    }

    /// Tries to push without blocking. Returns `Ok(())` on success, `Err(value)` if full.
    pub fn try_push(&mut self, mut value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }
        self.inner.size.fetch_add(1, Ordering::SeqCst);

        let tail = self.tail.load(Ordering::SeqCst);

        let cell = &self.inner.buffer[tail as usize];
        let ptr = cell.get();
        unsafe {
            ptr.swap(&mut value);
        }

        let new_tail = tail.wrapping_add(1) & self.inner.head_tail_mask;

        self.tail
            .compare_exchange(tail, new_tail, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to tail in SpscProducer"));

        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    pub fn is_full(&self) -> bool {
        self.inner.size.load(Ordering::SeqCst) as usize == self.capacity()
    }
}

impl<T: Default> SpscConsumer<T> {
    /// Pops a value from the queue. Blocks until an item is available.
    pub fn pop(&mut self) -> Option<T> {
        loop {
            if let Some(value) = self.try_pop() {
                return Some(value);
            }
        }
    }

    /// Tries to pop without blocking. Returns `Some(value)` or `None` if empty.
    pub fn try_pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        self.inner.size.fetch_sub(1, Ordering::SeqCst);

        let head = self.head.load(Ordering::SeqCst);

        // remove item from the buffer
        let mut value = T::default();

        let cell = &self.inner.buffer[head as usize];
        let ptr = cell.get();
        unsafe {
            ptr.swap(&mut value);
        }

        let new_head = head.wrapping_add(1) & self.inner.head_tail_mask;

        self.head
            .compare_exchange(head, new_head, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to head in SpscConsumer"));

        Some(value)
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        self.inner.buffer.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.size.load(Ordering::SeqCst) == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spsc_new_rejects_non_power_of_two() {
        assert!(matches!(
            SpscQueue::<i32>::new(0),
            Err(NonPowerOfTwoCapacity { capacity: 0 })
        ));
        assert!(matches!(
            SpscQueue::<i32>::new(3),
            Err(NonPowerOfTwoCapacity { capacity: 3 })
        ));
        assert!(matches!(
            SpscQueue::<i32>::new(15),
            Err(NonPowerOfTwoCapacity { capacity: 15 })
        ));
    }

    #[test]
    fn spsc_new_has_requested_capacity() {
        let q = SpscQueue::<i32>::new(16).unwrap();
        assert_eq!(q.capacity(), 16);
    }

    #[test]
    fn spsc_split_returns_producer_and_consumer() {
        let q = SpscQueue::<i32>::new(8).unwrap();
        let (prod, cons) = q.split();
        assert_eq!(prod.capacity(), 8);
        assert_eq!(cons.capacity(), 8);
        assert!(cons.is_empty());
        assert!(!prod.is_full());
    }

    #[test]
    fn spsc_try_push_try_pop_roundtrip() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(4).unwrap().split();
        assert!(prod.try_push(1).is_ok());
        assert!(prod.try_push(2).is_ok());
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn spsc_try_push_full_returns_err() {
        let (mut prod, _cons) = SpscQueue::<i32>::new(2).unwrap().split();
        let _ = prod.try_push(10);
        let _ = prod.try_push(20);
        let res = prod.try_push(30);
        assert!(res.is_err());
        if let Err(v) = res {
            assert_eq!(v, 30);
        }
    }

    #[test]
    fn spsc_try_pop_empty_returns_none() {
        let (mut _prod, mut cons) = SpscQueue::<i32>::new(4).unwrap().split();
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn spsc_push_pop_roundtrip() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(4).unwrap().split();
        prod.push(42);
        assert_eq!(cons.pop(), Some(42));
    }

    #[test]
    fn spsc_ring_wraps_around() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(2).unwrap().split();
        let _ = prod.try_push(1);
        let _ = prod.try_push(2);
        assert_eq!(cons.try_pop(), Some(1));
        let _ = prod.try_push(3);
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), Some(3));
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn spsc_is_empty_and_is_full() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(2).unwrap().split();
        assert!(cons.is_empty());
        assert!(!prod.is_full());
        let _ = prod.try_push(1);
        assert!(!cons.is_empty());
        let _ = prod.try_push(2);
        assert!(prod.is_full());
        let _ = cons.try_pop();
        assert!(!prod.is_full());
    }
}
