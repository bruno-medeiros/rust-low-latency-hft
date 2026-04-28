//! Manual-reset style gate: waiters block until [`NotifyGate::open`]; once open, the gate stays
//! open so late waiters do not block.
//!
//! Only benchmark crates depend on this; `bench-tool` itself does not call into it, so suppress
//! `dead_code` under `-D warnings`.

#![allow(dead_code)]

use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct NotifyGate {
    inner: Arc<NotifyGateInner>,
}

struct NotifyGateInner {
    open: Mutex<bool>,
    cv: Condvar,
}

impl NotifyGate {
    
    pub fn new() -> Self {
        Self {
            inner: Arc::new(NotifyGateInner {
                open: Mutex::new(false),
                cv: Condvar::new(),
            }),
        }
    }

    pub fn wait_until_open(&self) {
        let inner = &*self.inner;
        let mut open = inner.open.lock().expect("notify gate mutex poisoned");
        while !*open {
            open = inner
                .cv
                .wait(open)
                .expect("notify gate condvar poisoned");
        }
    }

    pub fn open(&self) {
        let inner = &*self.inner;
        let mut open = inner.open.lock().expect("notify gate mutex poisoned");
        *open = true;
        inner.cv.notify_all();
    }
}
