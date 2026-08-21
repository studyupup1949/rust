//! A cell that allows mutable access with re-entrancy support.
//!
//! When you call [`access`](AccessCell::access) with a closure that itself calls `access`
//! on the same cell, those nested calls are queued and run after the current closure
//! finishes, avoiding deadlock.
//!
//! This type is thread-safe: it implements [`Send`] and [`Sync`] when `T: Send`.

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex, RwLock,
    },
};

/// A cell holding a value of type `T` with re-entrant mutable access.
///
/// The first caller of [`access`](AccessCell::access) runs immediately; any further
/// calls to `access` from within that closure (or its queue) are enqueued and
/// executed in order after the current work completes.
///
/// Thread-safe: safe to share across threads and use from multiple threads.
pub struct AccessCell<T> {
    value: RwLock<T>,
    running: AtomicBool,
    queue: Mutex<VecDeque<Box<dyn FnOnce(&mut T) + Send>>>,
}

// Safe: only one thread holds the write guard at a time (enforced by `running` + queue).
unsafe impl<T: Send> Send for AccessCell<T> {}
unsafe impl<T: Send> Sync for AccessCell<T> {}

impl<T> AccessCell<T> {
    /// Creates a new `AccessCell` wrapping `value`.
    pub fn new(value: T) -> Self {
        Self {
            value: RwLock::new(value),
            running: AtomicBool::new(false),
            queue: Mutex::new(VecDeque::new()),
        }
    }

    /// Runs `f` with exclusive mutable access to the inner value.
    ///
    /// If called while another `access` closure is running (re-entrant or from
    /// another thread), `f` is queued and run after the current closure and
    /// any other queued closures finish.
    ///
    /// The closure must be `Send` so it can be moved into the queue when called
    /// from another thread.
    pub fn access(&self, f: impl FnOnce(&mut T) + Send + 'static) {
        // If already running (this thread re-entrant or another thread), enqueue.
        if self.running.swap(true, Ordering::Acquire) {
            self.queue.lock().unwrap().push_back(Box::new(f));
            return;
        }

        // We are the executor: hold the write lock for the whole run + drain.
        let mut guard = self.value.write().unwrap();
        f(&mut *guard);

        while let Some(job) = self.queue.lock().unwrap().pop_front() {
            job(&mut *guard);
        }

        drop(guard);
        self.running.store(false, Ordering::Release);
    }

    /// Returns an immutable reference to the inner value.
    ///
    /// Blocks if an `access` closure is currently running. Safe to call from
    /// any thread.
    pub fn access_ref(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.value.read().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_access() {
        let value = Arc::new(crate::AccessCell::new(0));

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

    #[test]
    fn test_access_multithreaded() {
        let cell = Arc::new(crate::AccessCell::new(0i32));
        let num_threads = 8;
        let increments_per_thread = 100;

        let handles: Vec<_> = (0..num_threads)
            .map(|_| {
                let cell = Arc::clone(&cell);
                thread::spawn(move || {
                    for _ in 0..increments_per_thread {
                        cell.access(|v| *v += 1);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let expected = num_threads * increments_per_thread;
        assert_eq!(*cell.access_ref(), expected);
    }
}
