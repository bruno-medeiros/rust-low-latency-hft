//! Multi-producer single-consumer (MPSC) lock-free queue.
//!
//! Uses an atomic sequence claim for multiple producers. Single consumer.

#![allow(dead_code)] // skeleton: fields used by real implementation

use std::marker::PhantomData;
use std::sync::Arc;

/// Shared state for the MPSC queue (buffer, sequence indices).
struct MpscInner<T> {
    capacity: usize,
    _marker: PhantomData<T>,
}

/// MPSC queue. Create with [`MpscQueue::new`]; the returned producer is [`Clone`] for multiple producers.
pub struct MpscQueue<T> {
    _marker: PhantomData<T>,
}

/// Producer handle for an MPSC queue. Clone this to get multiple producers.
pub struct MpscProducer<T> {
    inner: Arc<MpscInner<T>>,
}

/// Consumer handle for an MPSC queue. Only one consumer may exist per queue.
pub struct MpscConsumer<T> {
    inner: Arc<MpscInner<T>>,
}

impl<T> MpscQueue<T> {
    /// Creates a new MPSC queue with the given capacity.
    /// Returns a cloneable producer and the single consumer.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_capacity: usize) -> (MpscProducer<T>, MpscConsumer<T>) {
        todo!()
    }
}

impl<T> Clone for MpscProducer<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<T> MpscProducer<T> {
    /// Pushes a value into the queue. Returns `Err(value)` if the queue is full.
    pub fn push(&self, _value: T) -> Result<(), T> {
        todo!()
    }

    /// Tries to push without blocking. Returns `Ok(())` on success, `Err(value)` if full.
    pub fn try_push(&self, _value: T) -> Result<(), T> {
        todo!()
    }

    /// Returns the queue capacity.
    pub fn capacity(&self) -> usize {
        todo!()
    }

    /// Returns true if the queue is full (best-effort; may be outdated with multiple producers).
    pub fn is_full(&self) -> bool {
        todo!()
    }
}

impl<T> MpscConsumer<T> {
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
    fn mpsc_new_returns_producer_and_consumer() {
        let (prod, cons) = MpscQueue::<i32>::new(16);
        assert_eq!(prod.capacity(), 16);
        assert_eq!(cons.capacity(), 16);
        assert!(cons.is_empty());
    }

    #[test]
    fn mpsc_try_push_try_pop_roundtrip() {
        let (prod, mut cons) = MpscQueue::<i32>::new(4);
        assert!(prod.try_push(1).is_ok());
        assert!(prod.try_push(2).is_ok());
        assert_eq!(cons.try_pop(), Some(1));
        assert_eq!(cons.try_pop(), Some(2));
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn mpsc_producer_is_cloneable() {
        let (prod1, mut cons) = MpscQueue::<i32>::new(8);
        let prod2 = prod1.clone();
        let _ = prod1.try_push(1);
        let _ = prod2.try_push(2);
        let a = cons.try_pop();
        let b = cons.try_pop();
        assert!(matches!((a, b), (Some(1), Some(2)) | (Some(2), Some(1))));
    }

    #[test]
    fn mpsc_multiple_producers_contribute_to_same_queue() {
        let (p1, mut cons) = MpscQueue::<i32>::new(32);
        let p2 = p1.clone();
        let p3 = p1.clone();
        let _ = p1.try_push(1);
        let _ = p2.try_push(2);
        let _ = p3.try_push(3);
        let mut seen = [cons.try_pop(), cons.try_pop(), cons.try_pop()];
        seen.sort();
        assert_eq!(seen, [Some(1), Some(2), Some(3)]);
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn mpsc_try_push_full_returns_err() {
        let (prod, _cons) = MpscQueue::<i32>::new(2);
        let _ = prod.try_push(10);
        let _ = prod.try_push(20);
        let res = prod.try_push(30);
        assert!(res.is_err());
        if let Err(v) = res {
            assert_eq!(v, 30);
        }
    }

    #[test]
    fn mpsc_try_pop_empty_returns_none() {
        let (_prod, mut cons) = MpscQueue::<i32>::new(4);
        assert_eq!(cons.try_pop(), None);
    }

    #[test]
    fn mpsc_push_pop_roundtrip() {
        let (prod, mut cons) = MpscQueue::<i32>::new(4);
        let _ = prod.push(42);
        assert_eq!(cons.pop(), Some(42));
    }

    #[test]
    fn mpsc_is_empty_and_is_full() {
        let (prod, mut cons) = MpscQueue::<i32>::new(2);
        assert!(cons.is_empty());
        let _ = prod.try_push(1);
        assert!(!cons.is_empty());
        let _ = prod.try_push(2);
        assert!(prod.is_full());
        let _ = cons.try_pop();
        assert!(!prod.is_full());
    }
}
