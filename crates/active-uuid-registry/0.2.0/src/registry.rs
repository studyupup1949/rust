use super::UuidPoolError;

use rand::Rng;
use std::sync::{Arc, OnceLock};
#[cfg(not(feature = "concurrent"))]
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[cfg(feature = "concurrent")]
use dashmap::{DashMap, DashSet};

type ContextKey = Arc<str>;

// Single threaded global pooling
#[cfg(not(feature = "concurrent"))]
type SingleThreadedPool = parking_lot::Mutex<HashMap<ContextKey, HashSet<Uuid>>>;

#[cfg(feature = "concurrent")]
type ConcurrentPool = DashMap<ContextKey, DashSet<Uuid>>;

enum GlobalUuidPool {
    #[cfg(not(feature = "concurrent"))]
    SingleThreaded(SingleThreadedPool),
    #[cfg(feature = "concurrent")]
    Concurrent(ConcurrentPool),
}

// Thread-safe UUID pool using Mutex
static GLOBAL_UUID_POOL: OnceLock<GlobalUuidPool> = OnceLock::new();

fn global_pool() -> &'static GlobalUuidPool {
    GLOBAL_UUID_POOL.get_or_init(|| {
        #[cfg(not(feature = "concurrent"))]
        {
            GlobalUuidPool::SingleThreaded(parking_lot::Mutex::new(HashMap::new()))
        }
        #[cfg(feature = "concurrent")]
        {
            GlobalUuidPool::Concurrent(DashMap::new())
        }
    })
}

fn make_uuid_with_base(base: u32) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&base.to_be_bytes());
    for i in bytes.iter_mut().skip(4) {
        *i = rand::rng().random_range(0..=255);
    }
    Uuid::new_v8(bytes)
}

fn try_insert(context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let key: ContextKey = Arc::from(context);
            map.entry(key).or_insert_with(HashSet::new).insert(uuid)
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => {
            let key: ContextKey = Arc::from(context);

            let set_ref = pool.entry(key).or_insert_with(DashSet::new);
            set_ref.insert(uuid)
        }
    }
}

fn contains(context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.get(context)
                .map(|set| set.contains(&uuid))
                .unwrap_or(false)
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => pool
            .get(context)
            .map(|set_ref| set_ref.value().contains(&uuid))
            .unwrap_or(false),
    }
}

fn remove(context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let Some(set) = map.get_mut(context) else {
                return false;
            };

            let removed = set.remove(&uuid);
            if set.is_empty() {
                map.remove(context);
            }
            removed
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => pool
            .get(context)
            .map(|set_ref| set_ref.value().remove(&uuid).is_some())
            .unwrap_or(false),
    }
}

fn clear_context(context: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.remove(context);
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.remove(context);
        }
    }
}

fn clear_all() {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.clear();
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.clear();
        }
    }
}

pub(crate) fn random_uuid(
    context: &str,
    base: u32,
    max_retries: usize,
    retry_count: usize,
) -> Result<Uuid, UuidPoolError> {
    if retry_count >= max_retries {
        return Err(UuidPoolError::FailedToGenerateUniqueUuidError(format!(
            "Failed to generate unique UUID after {} attempts",
            max_retries
        )));
    }

    let new_uuid = make_uuid_with_base(base);

    if try_insert(context, new_uuid) {
        Ok(new_uuid)
    } else {
        random_uuid(context, base, max_retries, retry_count + 1)
    }
}

pub(crate) fn add_uuid_to_pool(context: &str, uuid: &Uuid) -> Result<(), UuidPoolError> {
    match contains(context, *uuid) {
        true => {
            return Err(UuidPoolError::FailedToGenerateUniqueUuidError(format!(
                "UUID already exists in pool for context '{}': {}",
                context, uuid
            )));
        }
        false => {
            if !try_insert(context, *uuid) {
                return Err(UuidPoolError::FailedToGenerateUniqueUuidError(format!(
                    "UUID already exists in pool for context '{}': {}",
                    context, uuid
                )));
            }
        }
    }

    Ok(())
}

pub(crate) fn remove_uuid_from_pool(context: &str, uuid: &Uuid) -> Result<(), UuidPoolError> {
    match remove(context, *uuid) {
        true => Ok(()),
        false => Err(UuidPoolError::FailedToFindUuidInPoolError(
            "Failed to locate/remove UUID in pool".to_string(),
        )),
    }
}

pub(crate) fn replace_uuid_in_pool(
    context: &str,
    old_uuid: &Uuid,
    new_uuid: &Uuid,
) -> Result<(), UuidPoolError> {
    if !contains(context, *old_uuid) {
        add_uuid_to_pool(context, new_uuid)?
    }

    match remove(context, *old_uuid) {
        true => {
            if !try_insert(context, *new_uuid) {
                return Err(UuidPoolError::FailedToSetUuidInPoolError(format!(
                    "Failed to find UUID in pool for context '{}': {}",
                    context, old_uuid
                )));
            }
        }
        false => {
            return Err(UuidPoolError::FailedToFindUuidInPoolError(format!(
                "Failed to find UUID in pool for context '{}': {}",
                context, old_uuid
            )));
        }
    }

    Ok(())
}

pub(crate) fn get_context_uuids_from_pool(context: &str) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.get(context).map(|set| set.clone().iter().map(|uuid| (context.to_string(), *uuid)).collect()).ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!("Failed to find UUIDs in pool for context '{}'", context)))
        }
        #[cfg(feature = "concurrent")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.get(context).map(|set| set.value().clone().iter().map(|uuid| (context.to_string(), *uuid)).collect()).ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!("Failed to find UUIDs in pool for context '{}'", context)))
        }
    }
}

pub(crate) fn drain_context(context: &str) -> Result<(), UuidPoolError> {
    clear_context(context);
    Ok(())
}

pub(crate) fn drain_all_contexts() -> Result<(), UuidPoolError> {
    clear_all();
    Ok(())
}
