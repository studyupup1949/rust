use super::UuidPoolError;

use std::sync::{Arc, OnceLock};
use uuid::Uuid;

#[cfg(not(feature = "concurrent-map"))]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "concurrent-map")]
use dashmap::{DashMap, DashSet};

type ContextKey = Arc<str>;

// Single threaded global pooling
#[cfg(not(feature = "concurrent-map"))]
type SingleThreadedPool = parking_lot::Mutex<HashMap<ContextKey, HashSet<Uuid>>>;

#[cfg(feature = "concurrent-map")]
type ConcurrentPool = DashMap<ContextKey, DashSet<Uuid>>;

enum GlobalUuidPool {
    #[cfg(not(feature = "concurrent-map"))]
    SingleThreaded(SingleThreadedPool),
    #[cfg(feature = "concurrent-map")]
    Concurrent(ConcurrentPool),
}

// Thread-safe UUID pool using Mutex
static GLOBAL_UUID_POOL: OnceLock<GlobalUuidPool> = OnceLock::new();

fn global_pool() -> &'static GlobalUuidPool {
    GLOBAL_UUID_POOL.get_or_init(|| {
        #[cfg(not(feature = "concurrent-map"))]
        {
            GlobalUuidPool::SingleThreaded(parking_lot::Mutex::new(HashMap::new()))
        }
        #[cfg(feature = "concurrent-map")]
        {
            GlobalUuidPool::Concurrent(DashMap::new())
        }
    })
}

fn make_uuid_with_base(base: u32) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&base.to_be_bytes());
    for i in bytes.iter_mut().skip(4) {
        *i = rand::random_range(0..=255);
    }
    Uuid::new_v8(bytes)
}

fn try_insert(context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let key: ContextKey = Arc::from(context);
            map.entry(key).or_insert_with(HashSet::new).insert(uuid)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let key: ContextKey = Arc::from(context);

            let set_ref = pool.entry(key).or_insert_with(DashSet::new);
            set_ref.insert(uuid)
        }
    }
}

fn remove(context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
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
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let removed = pool.get(context).map(|set_ref| set_ref.value().remove(&uuid).is_some()).unwrap_or(false);
        
            if removed {
                if let Some(set_ref) = pool.get(context) {
                    if set_ref.value().is_empty() {
                        drop(set_ref);
                        pool.remove(context);
                    }
                }
            }
            removed
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
    if !try_insert(context, *uuid) {
        return Err(UuidPoolError::FailedToAddUuidToPoolError(format!(
            "Failed to add UUID to pool for context '{}': {}",
            context, uuid
        )));
    }

    Ok(())
}

pub(crate) fn remove_uuid_from_pool(context: &str, uuid: &Uuid) -> Result<(), UuidPoolError> {
    match remove(context, *uuid) {
        true => Ok(()),
        false => Err(UuidPoolError::FailedToRemoveUuidFromPoolError(
            "Failed to locate/remove UUID in pool".to_string(),
        )),
    }
}

pub(crate) fn replace_uuid_in_pool(
    context: &str,
    old_uuid: &Uuid,
    new_uuid: &Uuid,
) -> Result<(), UuidPoolError> {
    match remove(context, *old_uuid) {
        true => {
            if !try_insert(context, *new_uuid) {
                return Err(UuidPoolError::FailedToReplaceUuidInPoolError(format!(
                    "Failed to insert new UUID in pool for context '{}': {}",
                    context, new_uuid
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
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.get(context).map(|set| set.iter().map(|uuid| (context.to_string(), *uuid)).collect()).ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!("Failed to find UUIDs in pool for context '{}'", context)))
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.get(context).map(|set| set.value().iter().map(|uuid| (context.to_string(), *uuid)).collect()).ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!("Failed to find UUIDs in pool for context '{}'", context)))
        }
    }
}

pub(crate) fn get_all_contexts_uuids_from_pool() -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            Ok(map.iter().flat_map(|(context, ids)| {
                ids.iter().map(move |id| (context.to_string(), *id))
            }).collect())
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            Ok(pool.iter().flat_map(|entry| {
                let context = entry.key().to_string();
                let uuids: Vec<Uuid> = entry.value().iter().map(|id| *id).collect();
                uuids.into_iter().map(move |id| (context.clone(), id))
            }).collect())
        }
    }
}

pub(crate) fn list_contexts() -> Vec<String> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.keys().map(|context| context.to_string()).collect()
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.iter().map(|entry| entry.key().to_string()).collect()
        }
    }
}

pub(crate) fn clear_context(context: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.remove(context);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.remove(context);
        }
    }
}

pub(crate) fn clear_all_contexts() {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.clear();
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.clear();
        }
    }
}

pub(crate) fn drain_context(context: &str) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let pairs = map
                .get(context)
                .map(|set| set.iter().map(|uuid| (context.to_string(), *uuid)).collect())
                .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find UUIDs in pool for context '{}'",
                    context
                )))?;
            map.remove(context);
            Ok(pairs)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let set = pool
                .remove(context)
                .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find UUIDs in pool for context '{}'",
                    context
                )))?;

            let pairs = set.1.iter().map(|uuid| (context.to_string(), *uuid)).collect();
            
            Ok(pairs)
        }
    }
}

pub(crate) fn drain_all_contexts() -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let pairs: Vec<(String, Uuid)> = map
                .iter()
                .flat_map(|(context, ids)| {
                    ids.iter().map(move |id| (context.to_string(), *id))
                })
                .collect();
            map.clear();
            Ok(pairs)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            // drains an acquired snapshot of the pool
            let mut pairs = Vec::new();
            
            let keys: Vec<ContextKey> = pool.iter().map(|entry| entry.key().clone()).collect();
            
            for key in keys {
                if let Some((context, set)) = pool.remove(&*key) {
                    for uuid in set.iter() {
                        pairs.push((context.to_string(), *uuid));
                    }
                }
            }
            
            Ok(pairs)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    struct SingleThreadedTests {

    }
}