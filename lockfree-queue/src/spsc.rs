//! Single-producer single-consumer (SPSC) lock-free queue.
//!
//! Ring buffer over a fixed-size heap-allocated array with cache-line padding
//! on head/tail. Uses sequence numbers with Acquire/Release ordering.

use std::marker::PhantomData;
use std::sync::Arc;

/// Shared state for the SPSC queue (buffer, head/tail indices).
struct SpscInner<T> {
    capacity: usize,
    _marker: PhantomData<T>,
}

/// SPSC queue. Create with [`SpscQueue::new`], then [`split`](SpscQueue::split) into producer and consumer.
pub struct SpscQueue<T> {
    data: Vec<T>,
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
    pub fn new(capacity: usize) -> Self {
        let data = Vec::with_capacity(capacity);
        Self { data }
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Splits the queue into a single producer and single consumer handle.
    pub fn split(self) -> (SpscProducer<T>, SpscConsumer<T>) {
        todo!()
    }
}

impl<T> SpscProducer<T> {
    /// Pushes a value into the queue. Blocks or returns error if full.
    /// Returns `Err(value)` if the queue is full (caller keeps the value).
    pub fn push(&mut self, _value: T) -> Result<(), T> {
        todo!()
    }

    /// Tries to push without blocking. Returns `Ok(())` on success, `Err(value)` if full.
    pub fn try_push(&mut self, _value: T) -> Result<(), T> {
        todo!()
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        todo!()
    }

    /// Returns true if the queue is full.
    pub fn is_full(&self) -> bool {
        todo!()
    }
}

impl<T> SpscConsumer<T> {
    /// Pops a value from the queue. Blocks until an item is available.
    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }

    /// Tries to pop without blocking. Returns `Some(value)` or `None` if empty.
    pub fn try_pop(&mut self) -> Option<T> {
        todo!()
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        todo!()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        todo!()
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
        let _ = prod.push(42);
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
