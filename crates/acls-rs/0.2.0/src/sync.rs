//! Generic synchronization strategy for interior mutability.
//!
//! This module provides a trait abstraction over `Mutex<T>` and `RefCell<T>`
//! to enable compile-time selection between thread-safe and single-threaded variants.

use std::cell::RefCell;
use std::sync::Mutex;

/// A synchronization strategy for providing interior mutability.
///
/// This trait abstracts over `Mutex<T>` (thread-safe) and `RefCell<T>` (single-threaded)
/// to allow generic code to work with either synchronization primitive.
///
/// # Type Parameters
///
/// * `T` - The type being protected by the synchronization primitive
///
/// # Examples
///
/// ```
/// use acls_rs::sync::SyncStrategy;
/// use std::sync::Mutex;
/// use std::cell::RefCell;
///
/// fn use_cache<S: SyncStrategy<Vec<String>>>(cache: &S) -> usize {
///     cache.with(|data| data.len())
/// }
///
/// // Thread-safe variant
/// let cache = Mutex::new(vec!["a".to_string()]);
/// assert_eq!(use_cache(&cache), 1);
///
/// // Single-threaded variant
/// let cache = RefCell::new(vec!["a".to_string()]);
/// assert_eq!(use_cache(&cache), 1);
/// ```
pub trait SyncStrategy<T>: Sized {
    /// Wrap a value in this synchronization primitive.
    ///
    /// # Arguments
    ///
    /// * `inner` - The value to protect
    ///
    /// # Returns
    ///
    /// A new instance wrapping the value
    fn new(inner: T) -> Self;

    /// Execute a closure with mutable access to the protected value.
    ///
    /// For `Mutex`, this acquires the lock. For `RefCell`, this borrows mutably.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that receives mutable access to the value
    ///
    /// # Returns
    ///
    /// The result of the closure
    ///
    /// # Panics
    ///
    /// For `RefCell`, panics if already borrowed.
    /// For `Mutex`, handles poisoned mutexes by logging a warning and recovering.
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R;
}

/// Thread-safe synchronization using `Mutex`.
///
/// Automatically handles poisoned mutexes by logging a warning and recovering
/// the inner value. This prevents cascading failures when a thread panics while
/// holding the lock.
impl<T> SyncStrategy<T> for Mutex<T> {
    fn new(inner: T) -> Self {
        Mutex::new(inner)
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = match self.lock() {
            Ok(g) => g,
            Err(e) => {
                log::warn!(
                    "acls: mutex was poisoned (a thread panicked while holding the lock). \
                     Recovering, but state may be inconsistent."
                );
                e.into_inner()
            }
        };
        f(&mut guard)
    }
}

/// Single-threaded synchronization using `RefCell`.
///
/// Provides zero-cost interior mutability for single-threaded use cases.
/// Use this variant when you know the type will only be used from one thread.
impl<T> SyncStrategy<T> for RefCell<T> {
    fn new(inner: T) -> Self {
        RefCell::new(inner)
    }

    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(&mut self.borrow_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_strategy() {
        let cache = Mutex::new(Vec::<String>::new());
        cache.with(|data| data.push("test".to_string()));
        cache.with(|data| {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0], "test");
        });
    }

    #[test]
    fn test_refcell_strategy() {
        let cache = RefCell::new(Vec::<String>::new());
        cache.with(|data| data.push("test".to_string()));
        cache.with(|data| {
            assert_eq!(data.len(), 1);
            assert_eq!(data[0], "test");
        });
    }

    #[test]
    fn test_generic_function() {
        fn count_items<S: SyncStrategy<Vec<i32>>>(storage: &S) -> usize {
            storage.with(|data| data.len())
        }

        let mutex_storage = Mutex::new(vec![1, 2, 3]);
        let refcell_storage = RefCell::new(vec![1, 2, 3]);

        assert_eq!(count_items(&mutex_storage), 3);
        assert_eq!(count_items(&refcell_storage), 3);
    }
}
