use std::{collections::HashMap, hash::Hash, marker::PhantomData};

use crate::{DerefLt, Guard};

use super::{GuardLt, MutexProvider, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug)]
pub struct LocalMutexProvider<T, K> {
    map: tokio::sync::RwLock<HashMap<K, Arc<Mutex<Option<T>>>>>,
}

impl<T, K> LocalMutexProvider<T, K> {
    pub fn new() -> LocalMutexProvider<T, K> {
        LocalMutexProvider {
            map: RwLock::new(HashMap::new()),
        }
    }
}

pub struct LocalMutex<T> {
    mutex: Arc<Mutex<Option<T>>>,
}

#[async_trait]
impl<T> super::Mutex<T> for LocalMutex<T>
where
    T: Send + Sync + 'static,
{
    type Guard = LocalGuardCtor<T>;
    async fn lock(&self) -> Result<LocalGuard<'_, T>> {
        let guard = self.mutex.lock().await;
        Ok(LocalGuard { guard })
    }
}

pub struct LocalGuardCtor<T>(PhantomData<T>);

impl<'a, T> GuardLt<'a, T> for LocalGuardCtor<T>
where
    T: Send + Sync + 'static,
{
    type Guard = LocalGuard<'a, T>;
}

pub struct LocalGuard<'a, T> {
    guard: tokio::sync::MutexGuard<'a, Option<T>>,
}

pub struct LocalDerefCtor<T>(PhantomData<T>);

impl<'a, T> DerefLt<'a, T> for LocalDerefCtor<T>
where
    T: Send + Sync + 'static,
{
    type Deref = &'a Option<T>;
}

#[async_trait]
impl<'a, T> Guard<T> for LocalGuard<'a, T>
where
    T: Send + Sync + 'static,
{
    type D = LocalDerefCtor<T>;
    async fn store(&mut self, data: T) -> Result<()> {
        *self.guard = Some(data);
        Ok(())
    }

    async fn load<'s>(&'s self) -> Result<&'s Option<T>> {
        Ok(&*self.guard)
    }

    async fn clear(&mut self) -> Result<()> {
        *self.guard = None;
        Ok(())
    }
}

#[async_trait]
impl<T, K> MutexProvider<T, K> for LocalMutexProvider<T, K>
where
    T: Send + Sync + 'static,
    K: Hash + Eq + Send + Sync,
{
    type Mutex = LocalMutex<T>;
    async fn get(&self, key: K) -> Result<Self::Mutex> {
        let mutex = {
            let map_readguard = self.map.read().await;
            if let Some(lock) = map_readguard.get(&key) {
                lock.clone()
            } else {
                drop(map_readguard);
                let mutex = Arc::new(Mutex::new(None));
                let mut writeguard = self.map.write().await;
                writeguard.insert(key, mutex.clone());
                mutex
            }
        };
        Ok(LocalMutex { mutex })
    }
}

#[cfg(test)]
mod tests {
    use crate::spec::{check_empty, check_val};

    use super::LocalMutexProvider;

    #[tokio::test]
    async fn test() {
        check_empty(LocalMutexProvider::new()).await;
        check_val(LocalMutexProvider::new()).await;
    }
}
