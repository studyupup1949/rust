//! A cell that allows mutable access with re-entrancy support.
//!
//! When you call [`access`](AccessCell::access) with a closure that itself calls `access`
//! on the same cell, those nested calls are queued and run after the current closure
//! finishes, avoiding deadlock.

use std::{
    cell::{Cell, UnsafeCell},
    collections::VecDeque,
    sync::Mutex,
};

/// A cell holding a value of type `T` with re-entrant mutable access.
///
/// The first caller of [`access`](AccessCell::access) runs immediately; any further
/// calls to `access` from within that closure (or its queue) are enqueued and
/// executed in order after the current work completes.
pub struct AccessCell<T> {
    value: UnsafeCell<T>,
    running: Cell<bool>,
    queue: Mutex<VecDeque<Box<dyn FnOnce(&mut T)>>>,
}

impl<T> AccessCell<T> {
    /// Creates a new `AccessCell` wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            running: Cell::new(false),
            queue: Mutex::new(VecDeque::new()),
        }
    }

    /// Runs `f` with exclusive mutable access to the inner value.
    ///
    /// If called while another `access` closure is running (re-entrant call),
    /// `f` is queued and run after the current closure and any other queued
    /// closures finish.
    pub fn access(&self, f: impl FnOnce(&mut T) + 'static) {
        // already inside → enqueue
        if self.running.get() {
            self.queue.lock().unwrap().push_back(Box::new(f));

            return;
        }

        // first entrant = executor
        self.running.set(true);

        // call current
        f(self.access_mut());

        // drain queued re-entrant calls
        while let Some(job) = self.queue.lock().unwrap().pop_front() {
            let value = self.access_mut();
            job(value);
        }

        self.running.set(false);
    }

    /// Returns a mutable reference to the inner value.
    ///
    /// # Safety
    /// Only call this while you are inside an `access` closure (or while draining
    /// the queue). Otherwise you may create aliasing mutable references.
    pub fn access_mut(&self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }

    /// Returns an immutable reference to the inner value.
    ///
    /// Safe to call at any time; no closure is running when you use this for
    /// read-only access.
    pub fn access_ref(&self) -> &T {
        unsafe { &*self.value.get() }
    }
}

mod tests {
    #[test]
    fn test_access() {
        let value = std::sync::Arc::new(crate::AccessCell::new(0));

        value.access({
            let value = value.clone();

            move |_| {
                // normally this would cause a deadlock
                value.access(|v| {
                    *v = 10;
                })
            }
        });

        assert_eq!(*value.access_ref(), 10);
    }
}
