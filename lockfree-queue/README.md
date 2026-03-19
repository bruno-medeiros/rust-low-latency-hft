# lockfree-queue

Lock-free **SPSC** (single-producer single-consumer) queue crate.

## SPSC implementation

Fixed-size **ring buffer**; `SpscQueue::new(slot_count)` takes the number of **slots** (`slot_count`). It must be a **power of two** and **at least 2**. **One slot is reserved** so head and tail can tell **empty** from **full**, so the **maximum number of elements** you can hold at once is **one less than the slot count** (see **`slot_count()`** on the queue and handles).

**Head** and **tail** are shared [`AtomicU32`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU32.html)s (cache-line padded). The producer **writes** `tail` with `Release` after filling a slot; the consumer **loads** it with `Acquire` before reading. Slots are **`UnsafeCell<Option<T>>`** (no `T: Default` required).

## Possible improvement: `MaybeUninit<T>`

Slots could use **`MaybeUninit<T>`** instead of **`Option<T>`** to drop the discriminant (often smaller and less branching for small `T`), at the cost of more **`unsafe`** and careful **drop** handling.
