use rand::Rng;
use std::marker::PhantomData;
use std::mem::take;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{DerefLt, Empty, Guard};

use super::{GuardLt, MutexProvider, Result};
use async_trait::async_trait;
use bb8_redis::redis::Script;
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use rand::thread_rng;
use redis::AsyncCommands;
pub use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs, Value};
use tokio::sync::oneshot::Sender;
use tokio::sync::{RwLock, RwLockReadGuard};
use tracing::{error, trace, warn};

/// The time in milliseconds after which a lock lease will expire if not renewed.
const LOCK_LEASE_TIMEOUT_MILLIS: u64 = 10_000;
/// The time interval at which lock lease renewal will be attempted
const LOCK_REFRESH_INTERVAL_MILLIS: u64 = 1_000;
/// The time interval between attempts to acquire the lock
const LOCK_POLL_INTERVAL_MILLIS: u64 = 100;
/// Buffer time before lock lease expiration which determines when the guard
/// refresher will panic when it fails to renew the lock lease.
const RENEWAL_PANIC_BUFFER_MILLIS: u64 = 1_000;

#[derive(Debug, Clone)]
pub struct RedisMutexProvider {
    pool: Pool<RedisConnectionManager>,
    provider_id: String,
}

impl RedisMutexProvider {
    pub fn new(provider_id: String, pool: Pool<RedisConnectionManager>) -> RedisMutexProvider {
        RedisMutexProvider { pool, provider_id }
    }
}

#[derive(Clone, Debug)]
pub struct RedisMutex {
    pool: Pool<RedisConnectionManager>,
    key: String,
    mutex_id: u64,
}

// KEYS[1] = lock key
// ARGV[1] = mutex id
// ARGV[2] = lock timeout in millis
const ACQUIRE_LOCK_SCRIPT: &str = "\
  local got_lock = redis.call('SET', KEYS[1], ARGV[1], 'NX', 'PXAT', ARGV[2])
  if got_lock then
      return 1
  end
  return 0
";

// KEYS[1] = lock key
// ARGV[1] = mutex id
// ARGV[2] = new lock expiration time in unix millis
const RENEW_LOCK_SCRIPT: &str = "\
  if redis.call('GET', KEYS[1]) == ARGV[1] then
      redis.call('PEXPIREAT', KEYS[1],  ARGV[2])
      return 1
  end
  return 0
";

// KEYS[1] = lock key
// ARGV[1] = mutex id
const DROP_LOCK_SCRIPT: &str = "\
  if redis.call('GET', KEYS[1]) == ARGV[1] then
      redis.call('DEL', KEYS[1])
      return 1
  end
  return 0
";

impl RedisMutex {
    /// Attempts to acquire the lock. If successful, returns the lock lease
    /// expiration time as a unix timestamp in milliseconds. If the lock
    /// is already locked, returns None so that clients know to retry.
    async fn try_acquire_lock(&self) -> Result<Option<Duration>> {
        let exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
            + Duration::from_millis(LOCK_LEASE_TIMEOUT_MILLIS);
        Ok(
            if Script::new(ACQUIRE_LOCK_SCRIPT)
                .key(self.key.as_str())
                .arg(self.mutex_id)
                .arg(exp.as_millis() as i64)
                .invoke_async::<_, i32>(&mut *self.pool.get().await?)
                .await?
                == 1
            {
                Some(exp)
            } else {
                None
            },
        )
    }

    /// Attempts to renew the the lock lease. If successful, returns the new
    /// lock lease expiration time as a unix timestamp in milliseconds. If the lock
    /// is no longer held by this instance, returns None (clients should generally
    /// panic in this case)
    async fn try_renew_lock(&self) -> Result<Option<Duration>> {
        let new_exp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
            + Duration::from_millis(LOCK_LEASE_TIMEOUT_MILLIS);
        Ok(
            if Script::new(RENEW_LOCK_SCRIPT)
                .key(self.key.as_str())
                .arg(self.mutex_id)
                .arg(new_exp.as_millis() as i64)
                .invoke_async::<_, i32>(&mut *self.pool.get().await?)
                .await?
                == 1
            {
                Some(new_exp)
            } else {
                None
            },
        )
    }

    /// Attempts to drop the lock. Returns true if the lock was held by this
    /// owner and was dropped. Returns false if the lock was not held by
    /// this owner and nothing happened.
    async fn drop_lock(&self) -> Result<bool> {
        Ok(Script::new(DROP_LOCK_SCRIPT)
            .key(self.key.as_str())
            .arg(self.mutex_id)
            .invoke_async::<_, i32>(&mut *self.pool.get().await?)
            .await?
            == 1)
    }
}

#[async_trait]
impl<T> super::Mutex<T> for RedisMutex
where
    T: Send + FromRedisValue + ToRedisArgs + Sync + 'static,
{
    type Guard = RedisGuardCtor<T>;
    async fn lock(&self) -> Result<RedisGuard<'_, T>> {
        let mut interval =
            tokio::time::interval(core::time::Duration::from_millis(LOCK_POLL_INTERVAL_MILLIS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let expires_at;
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Some(exp) = self.try_acquire_lock().await? {
                        expires_at = exp;
                        break;
                    }
                }
            }
        }
        Ok(RedisGuard::new(&self, expires_at))
    }
}

pub struct RedisGuardCtor<T>(PhantomData<T>);

impl<'a, T> GuardLt<'a, T> for RedisGuardCtor<T>
where
    T: FromRedisValue + ToRedisArgs + Send + Sync + 'static,
{
    type Guard = RedisGuard<'a, T>;
}

pub struct RedisGuard<'a, T> {
    mutex: &'a RedisMutex,
    drop_tx: Option<Sender<()>>,
    loaded: AtomicBool,
    data: RwLock<Option<T>>,
    _pd: PhantomData<T>,
}

impl<'a, T> RedisGuard<'a, T> {
    fn new(mutex: &'a RedisMutex, exp_at: Duration) -> RedisGuard<'a, T> {
        trace!(key = %mutex.key, mutex_id = %mutex.mutex_id, expires_at = ?exp_at, "acquired lock");
        let (drop_tx, mut drop_rx) = tokio::sync::oneshot::channel();
        let mutex_clone = mutex.clone();
        let _ = tokio::spawn(async move {
            let mutex = mutex_clone;
            let mut renewal_interval = tokio::time::interval(core::time::Duration::from_millis(
                LOCK_REFRESH_INTERVAL_MILLIS,
            ));
            renewal_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            let panic_timeout = tokio::time::sleep(
                exp_at
                    - SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                    - Duration::from_millis(RENEWAL_PANIC_BUFFER_MILLIS),
            );
            tokio::pin!(panic_timeout);
            loop {
                tokio::select! {
                    _ = &mut drop_rx => {
                        break;
                    }
                    _ = renewal_interval.tick() => {
                        match mutex.try_renew_lock().await {
                            Ok(Some(new_exp)) => {
                                trace!(key = %mutex.key, mutex_id = %mutex.mutex_id, expires_at = ?new_exp, "renewed lock lease");
                                panic_timeout.as_mut().reset(tokio::time::Instant::from_std(Instant::now() + new_exp
                                    - SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                                    - Duration::from_millis(RENEWAL_PANIC_BUFFER_MILLIS)));
                            },
                            Ok(None) => {
                              panic!("failed to renew mutex because it had a different owner: {}", mutex.key);
                            },
                            Err(e) => {
                                error!(key = %mutex.key, mutex_id = %mutex.mutex_id, "failed to renew lease on lock, scheduling retry: {}", e);
                                continue;
                            },
                        }
                    }
                    _ = &mut panic_timeout => {
                        panic!("failed to renew mutex before lease expiration: {}", mutex.key);
                    }
                }
            }
            match mutex.drop_lock().await {
                Ok(false) => {
                    warn!(key = %mutex.key, mutex_id = %mutex.mutex_id, "lock already had different owner while attempting to drop");
                }
                Err(e) => {
                    error!(key = %mutex.key, mutex_id = %mutex.mutex_id, "failed to drop lock: {}", e);
                }
                _ => {
                    trace!(key = %mutex.key, mutex_id = %mutex.mutex_id, "successfully dropped lock");
                }
            }
        });
        RedisGuard {
            mutex,
            loaded: AtomicBool::new(false),
            drop_tx: Some(drop_tx),
            data: RwLock::new(None),
            _pd: Default::default(),
        }
    }
}

impl<'a, T> Drop for RedisGuard<'a, T> {
    fn drop(&mut self) {
        if let Some(tx) = take(&mut self.drop_tx) {
            let _ = tx.send(());
            trace!(key = %self.mutex.key, mutex_id = %self.mutex.mutex_id, "guard dropped");
        }
    }
}

fn format_data_key(key: &str) -> String {
    format!("{}_data", key)
}

pub struct RedisDerefCtor<T>(PhantomData<T>);

impl<'a, T> DerefLt<'a, T> for RedisDerefCtor<T>
where
    T: Send + Sync + 'static,
{
    type Deref = RwLockReadGuard<'a, Option<T>>;
}

#[async_trait]
impl<'a, T> Guard<T> for RedisGuard<'a, T>
where
    T: FromRedisValue + ToRedisArgs + Send + Sync + 'static,
{
    type D = RedisDerefCtor<T>;
    async fn store(&mut self, data: T) -> Result<()> {
        let mut con = self.mutex.pool.get().await?;
        con.set(format_data_key(&self.mutex.key), &data).await?;
        let mut guard = self.data.write().await;
        *guard = Some(data);
        self.loaded.store(true, Ordering::Relaxed);
        Ok(())
    }
    async fn load<'s>(&'s self) -> Result<RwLockReadGuard<'s, Option<T>>> {
        if !self.loaded.load(std::sync::atomic::Ordering::Relaxed) {
            let mut con = self.mutex.pool.get().await?;
            let val: Option<T> = con.get(format_data_key(&self.mutex.key)).await?;
            let mut guard = self.data.write().await;
            *guard = val;
            self.loaded.store(true, Ordering::Relaxed);
        }
        return Ok(self.data.read().await);
    }
    async fn clear(&mut self) -> Result<()> {
        let mut con = self.mutex.pool.get().await?;
        con.del(format_data_key(&self.mutex.key)).await?;
        let mut guard = self.data.write().await;
        *guard = None;
        self.loaded.store(true, Ordering::Relaxed);
        Ok(())
    }
}

#[async_trait]
impl<T, K> MutexProvider<T, K> for RedisMutexProvider
where
    T: FromRedisValue + ToRedisArgs + Send + Sync + 'static,
    K: AsRef<str> + Send,
{
    type Mutex = RedisMutex;
    async fn get(&self, key: K) -> Result<Self::Mutex>
    where
        K: 'async_trait,
    {
        let key = format!("amutex_{}_{}", self.provider_id, key.as_ref());
        let mutex_id = thread_rng().gen::<u64>();
        Ok(RedisMutex {
            pool: self.pool.clone(),
            key,
            mutex_id,
        })
    }
}

impl ToRedisArgs for Empty {
    fn write_redis_args<W>(&self, _out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
    }
}

impl FromRedisValue for Empty {
    fn from_redis_value(_v: &Value) -> RedisResult<Self> {
        return Ok(Empty);
    }
}

#[cfg(test)]
mod tests {
    use bb8_redis::{bb8::Pool, RedisConnectionManager};
    use testcontainers::{clients::Cli, images::generic::GenericImage};

    use crate::spec::{check_empty, check_val};

    use super::RedisMutexProvider;

    #[tokio::test]
    async fn test() {
        let cli = Cli::default();
        let port = 6379;
        let container = cli.run(GenericImage::new("redis", "7.0").with_exposed_port(port));
        let host_port = container.get_host_port_ipv4(port);
        let uri = format!("redis://localhost:{host_port}");
        let redis_connection_manager = RedisConnectionManager::new(uri.as_str()).unwrap();
        let pool = Pool::builder()
            .build(redis_connection_manager)
            .await
            .unwrap();
        check_empty(RedisMutexProvider::new("testing".to_string(), pool.clone())).await;
        check_val(RedisMutexProvider::new("testing_vals".to_string(), pool)).await;
    }
}
