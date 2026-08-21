use crate::{Result, SemaphoreError};

/// Windows semaphore implementation.
///
/// This type is a thin wrapper around a native Windows semaphore kernel
/// object created through `CreateSemaphoreW`.
///
/// It provides the Windows backend for the portable [`Semaphore`](crate::Semaphore)
/// abstraction.
unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0},
    System::Threading::{CreateSemaphoreW, INFINITE, ReleaseSemaphore, WaitForSingleObject},
};

/// Windows backend for [`Semaphore`](crate::Semaphore).
///
/// Uses the Windows kernel semaphore object API.
pub struct Semaphore {
    inner: HANDLE,
}

impl Semaphore {
    /// Creates a new semaphore.
    ///
    /// `initial` specifies the initial number of available resources.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `CreateSemaphoreW`
    /// call fails.
    pub fn new(initial: u32) -> Result<Self> {
        if initial > i32::MAX as u32 {
            return Err(SemaphoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "initial semaphore value exceeds Windows limit",
            )));
        }

        unsafe {
            // SAFETY:
            // CreateSemaphoreW is called with a null security descriptor and
            // name, creating an unnamed semaphore owned by this instance.
            let handle =
                CreateSemaphoreW(std::ptr::null(), initial as i32, i32::MAX, std::ptr::null());

            if handle.is_null() {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(Self { inner: handle })
        }
    }

    /// Acquires one resource from the semaphore.
    ///
    /// Blocks the current thread until a resource becomes available.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if waiting on the Windows semaphore fails.
    pub fn wait(&self) -> Result<()> {
        unsafe {
            // SAFETY:
            // self.inner is a valid semaphore handle owned by this object.
            let result = WaitForSingleObject(self.inner, INFINITE);

            if result == WAIT_OBJECT_0 {
                Ok(())
            } else {
                Err(SemaphoreError::last_os_error())
            }
        }
    }

    /// Releases one resource back to the semaphore.
    ///
    /// This may wake one waiting thread.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if the underlying `ReleaseSemaphore`
    /// call fails.
    pub fn post(&self) -> Result<()> {
        unsafe {
            // SAFETY:
            // self.inner refers to a valid Windows semaphore object.
            if ReleaseSemaphore(self.inner, 1, std::ptr::null_mut()) == 0 {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(())
        }
    }

    /// Destroys the semaphore.
    ///
    /// Consumes the object, preventing further use after the handle is closed.
    ///
    /// # Errors
    ///
    /// Returns a [`SemaphoreError`] if closing the underlying Windows handle
    /// fails.
    pub fn destroy(self) -> Result<()> {
        unsafe {
            // SAFETY:
            // self owns this handle and no other operation is performed after
            // CloseHandle.
            if CloseHandle(self.inner) == 0 {
                return Err(SemaphoreError::last_os_error());
            }

            Ok(())
        }
    }
}

#[test]
fn invalid_initial_value() {
    assert!(Semaphore::new(u32::MAX).is_err());
}
