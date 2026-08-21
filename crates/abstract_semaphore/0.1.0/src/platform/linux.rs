use std::cell::UnsafeCell;

use crate::{Result, SemaphoreError};

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

/// Linux backend for [`Semaphore`](crate::Semaphore).
///
/// Uses POSIX semaphores provided by libc.
pub struct Semaphore {
    inner: UnsafeCell<libc::sem_t>,
}

impl Semaphore {
    /// Creates a new semaphore.
    ///
    /// The semaphore is initialized with `initial` available resources.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `sem_init()` call fails.
    pub fn new(initial: u32) -> Result<Self> {
        let inner = UnsafeCell::new(unsafe { std::mem::zeroed() });

        unsafe {
            if libc::sem_init(inner.get(), 0, initial) != 0 {
                return Err(SemaphoreError::last_os_error());
            }
        }

        Ok(Self { inner })
    }

    /// Acquires one resource from the semaphore.
    ///
    /// If no resources are available, the current thread blocks until another
    /// thread releases one by calling [`post`](Self::post).
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `sem_wait()` call fails.
    pub fn wait(&self) -> Result<()> {
        unsafe {
            if libc::sem_wait(self.as_mut_ptr()) != 0 {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(())
        }
    }

    /// Releases one resource back to the semaphore.
    ///
    /// If one or more threads are blocked in [`wait`](Self::wait), the
    /// operating system may wake one of them.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `sem_post()` call fails.
    pub fn post(&self) -> Result<()> {
        unsafe {
            if libc::sem_post(self.as_mut_ptr()) != 0 {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(())
        }
    }

    /// Destroys the semaphore.
    ///
    /// This consumes the semaphore, preventing further use after destruction.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `sem_destroy()` call
    /// fails.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no other thread is currently blocked on or
    /// accessing this semaphore. Violating this requirement results in the
    /// platform-specific behaviour of the operating system.
    pub fn destroy(self) -> Result<()> {
        unsafe {
            // SAFETY:
            // `self` owns a valid initialized semaphore which is consumed by
            // this method.
            if libc::sem_destroy(self.as_mut_ptr()) != 0 {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(())
        }
    }

    /// Returns a mutable pointer to the underlying POSIX semaphore.
    ///
    /// This helper is used exclusively for FFI calls to the operating system.
    fn as_mut_ptr(&self) -> *mut libc::sem_t {
        self.inner.get()
    }
}

//Tests unitarios

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_destroy() {
        let sem = Semaphore::new(1).unwrap();
        sem.destroy().unwrap();
    }
}
