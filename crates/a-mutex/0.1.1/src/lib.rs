pub mod local;
pub mod redis;

use std::ops::Deref;

use async_trait::async_trait;
use auto_impl::auto_impl;

pub type Result<T> = anyhow::Result<T>;

#[async_trait]
#[auto_impl(&, &mut, Box, Arc)]
pub trait MutexProvider<T, K>: Send + Sync {
    type Mutex: Send + Mutex<T>;
    async fn get(&self, key: K) -> Result<Self::Mutex>
    where
        K: 'async_trait;
}

pub trait GuardLt<'a, T> {
    type Guard: Guard<T>;
}

#[async_trait]
pub trait Mutex<T>: Send + Sync {
    type Guard: for<'a> GuardLt<'a, T>;
    async fn lock<'s>(&'s self) -> Result<<Self::Guard as GuardLt<'s, T>>::Guard>;
}

pub trait DerefLt<'a, T> {
    type Deref: Deref<Target = Option<T>>;
}

#[async_trait]
pub trait Guard<T>: Send + Sync {
    type D: for<'a> DerefLt<'a, T>;
    async fn store(&mut self, data: T) -> Result<()>;
    async fn load<'s>(&'s self) -> Result<<Self::D as DerefLt<'s, T>>::Deref>;
    async fn clear(&mut self) -> Result<()>;
}

pub struct Empty;

#[cfg(test)]
pub mod spec {
    use std::{
        sync::{atomic::AtomicI32, Arc},
        time::Duration,
    };

    use tokio::spawn;

    use crate::Empty;

    use super::{Guard, Mutex, MutexProvider};

    pub async fn check_empty(m: impl MutexProvider<Empty, &str> + 'static) {
        let count = Arc::new(AtomicI32::new(0));
        let m = Arc::new(m);
        let lock_key = "my_lock";
        let mut handles = vec![];
        for _ in 0..100 {
            let count = count.clone();
            let m = m.clone();
            handles.push(spawn(async move {
                let mutex = m.get(lock_key).await.unwrap();
                let _guard = mutex.lock().await.unwrap();
                let copy = count.load(std::sync::atomic::Ordering::Relaxed);
                tokio::time::sleep(Duration::from_millis(1)).await;
                count.store(copy + 1, std::sync::atomic::Ordering::Relaxed);
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        assert_eq!(100, count.load(std::sync::atomic::Ordering::Relaxed));
    }

    pub async fn check_val(m: impl MutexProvider<u64, &str> + 'static) {
        let m = Arc::new(m);
        let lock_key = "my_counter_lock";
        let mut handles = vec![];
        for _ in 0..100 {
            let m = m.clone();
            handles.push(spawn(async move {
                let mutex = m.get(lock_key).await.unwrap();
                let mut guard = mutex.lock().await.unwrap();
                let val = {
                    let val = guard.load().await.unwrap();
                    val.clone()
                };
                if let Some(val) = val {
                    guard.store(val + 1).await.unwrap();
                } else {
                    guard.store(1).await.unwrap();
                }
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
        let mutex = m.get(lock_key).await.unwrap();
        let guard = mutex.lock().await.unwrap();
        assert_eq!(
            100,
            *guard
                .load()
                .await
                .unwrap()
                .as_ref()
                .expect("no value found")
        );
    }
}
