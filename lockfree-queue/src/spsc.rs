use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Shared state for the SPSC queue (buffer, head/tail indices).
struct SpscInner<T: Default> {
    buffer: Vec<UnsafeCell<T>>,
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
    pub fn new(capacity: usize) -> Self {
        let data: Vec<UnsafeCell<T>> = (0..capacity)
            .map(|_| UnsafeCell::new(T::default()))
            .collect();
        Self {
            inner: Arc::new(SpscInner {
                buffer: data,
                size: Arc::new(AtomicU32::new(0)),
            }),
        }
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        self.inner.buffer.capacity()
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

        let new_tail = (tail + 1) % self.capacity() as u32;

        self.tail
            .compare_exchange(tail, new_tail, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to tail in SpscProducer"));

        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.inner.buffer.capacity()
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

        let new_head = (head + 1) % self.capacity() as u32;

        self.head
            .compare_exchange(head, new_head, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|_| panic!("Concurrent modification to head in SpscConsumer"));

        Some(value)
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        self.inner.buffer.capacity()
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
    fn spsc_new_has_requested_capacity() {
        let q = SpscQueue::<i32>::new(16);
        assert_eq!(q.capacity(), 16);
    }

    #[test]
    fn spsc_split_returns_producer_and_consumer() {
        let q = SpscQueue::<i32>::new(8);
        let (prod, cons) = q.split();
        assert_eq!(prod.capacity(), 8);
        assert_eq!(cons.capacity(), 8);
        assert!(cons.is_empty());
        assert!(!prod.is_full());
    }

    #[test]
    fn spsc_try_push_try_pop_roundtrip() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(4).split();
        assert!(prod.try_push(1).is_ok());
        assert!(prod.try_push(2).is_ok());
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn spsc_try_push_full_returns_err() {
        let (mut prod, _cons) = SpscQueue::<i32>::new(2).split();
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
        let (mut _prod, mut cons) = SpscQueue::<i32>::new(4).split();
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn spsc_push_pop_roundtrip() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(4).split();
        prod.push(42);
        assert_eq!(cons.pop(), Some(42));
    }

    #[test]
    fn spsc_ring_wraps_around() {
        let (mut prod, mut cons) = SpscQueue::<i32>::new(2).split();
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
        let (mut prod, mut cons) = SpscQueue::<i32>::new(2).split();
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
