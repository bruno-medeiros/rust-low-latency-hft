# lockfree-queue

Lock-free queue: SPSC (single-producer single-consumer).

## SPSC implementation

The queue is a **fixed-size ring buffer**. Capacity must be a **power of two**; head/tail indices advance with `(i + 1) & mask` where `mask = capacity - 1`.

Occupancy is tracked with a shared **`AtomicU32` size**. The producer keeps **`tail`** and the consumer **`head`** (each on its own handle after `split`). Slots are **`UnsafeCell<Option<T>>`**: empty slots are `None`, live elements `Some(value)`.

## Possible improvement: `MaybeUninit<T>`

Slots could use **`MaybeUninit<T>`** instead of **`Option<T>`** to avoid the discriminant (often smaller memory footprint and less branching for small `T`). That would mean **`write` / `read`** (or `assume_init_*`) per slot and careful **drop** logic when the queue or remaining items are destroyed—more `unsafe`, typical trade-off for tighter rings.
