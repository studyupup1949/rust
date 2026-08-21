use std::sync::{Arc, Mutex};

use accepts::{Accepts, AsyncAccepts};

/// Test helper that records accepted values into a shared Vec.
#[derive(Debug)]
pub struct Recorder<T> {
    storage: Arc<Mutex<Vec<T>>>,
}

impl<T> Recorder<T> {
    pub fn new() -> Self {
        Self::with_storage(Arc::new(Mutex::new(Vec::new())))
    }

    pub fn with_storage(storage: Arc<Mutex<Vec<T>>>) -> Self {
        Self { storage }
    }

    #[allow(dead_code)]
    pub fn storage(&self) -> Arc<Mutex<Vec<T>>> {
        Arc::clone(&self.storage)
    }

    pub fn take(&self) -> Vec<T> {
        let mut guard = self.storage.lock().expect("lock poisoned");
        std::mem::take(&mut *guard)
    }
}

impl<T> Default for Recorder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Recorder<T> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl<T> Accepts<T> for Recorder<T> {
    fn accept(&self, value: T) {
        self.storage
            .lock()
            .expect("storage mutex poisoned")
            .push(value);
    }
}

impl<T> AsyncAccepts<T> for Recorder<T> {
    fn accept_async<'a>(&'a self, value: T) -> impl core::future::Future<Output = ()> + 'a
    where
        T: 'a,
    {
        async move {
            self.storage
                .lock()
                .expect("storage mutex poisoned")
                .push(value);
        }
    }
}
