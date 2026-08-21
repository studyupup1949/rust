use super::{UuidPoolError, NamespaceString, ContextString};

use std::sync::{Arc, OnceLock};
use uuid::Uuid;

#[cfg(not(feature = "concurrent-map"))]
use std::collections::{HashMap, HashSet};

#[cfg(feature = "concurrent-map")]
use dashmap::{DashMap, DashSet};

type NamespaceKey = Arc<str>;
type ContextKey = Arc<str>;

#[cfg(not(feature = "concurrent-map"))]
type SingleThreadedMap = HashMap<ContextKey, HashSet<Uuid>>;

// Single threaded global pooling
#[cfg(not(feature = "concurrent-map"))]
type SingleThreadedPool = parking_lot::Mutex<HashMap<NamespaceKey, SingleThreadedMap>>;

#[cfg(feature = "concurrent-map")]
type ConcurrentMap = DashMap<ContextKey, DashSet<Uuid>>;

#[cfg(feature = "concurrent-map")]
type ConcurrentPool = DashMap<NamespaceKey, ConcurrentMap>;

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

fn try_insert(namespace: &str, context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let nm_key: NamespaceKey = Arc::from(namespace);
            let ct_key: ContextKey = Arc::from(context);
            map.entry(nm_key)
                .or_insert_with(HashMap::new)
                .entry(ct_key)
                .or_insert_with(HashSet::new)
                .insert(uuid)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let nm_key: NamespaceKey = Arc::from(namespace);
            let ct_key: ContextKey = Arc::from(context);
            let nm_ref = pool.entry(nm_key).or_insert_with(DashMap::new);
            nm_ref.entry(ct_key).or_insert_with(DashSet::new).insert(uuid)
        }
    }
}

fn remove(namespace: &str, context: &str, uuid: Uuid) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut nm_map = pool.lock();

            let removed = {
                let Some(ct_map) = nm_map.get_mut(namespace) else {
                    return false;
                };
                let Some(set) = ct_map.get_mut(context) else {
                    return false;
                };
                let removed = set.remove(&uuid);
                if set.is_empty() {
                    ct_map.remove(context);
                }
                removed
            };

            if removed && nm_map.get(namespace).map(|m| m.is_empty()).unwrap_or(false) {
                nm_map.remove(namespace);
            }

            removed
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let removed: bool = if let Some(nm_ref) = pool.get(namespace) {
                let ct_map = nm_ref.value();
                let removed = ct_map
                    .get(context)
                    .map(|set_ref| set_ref.value().remove(&uuid))
                    .unwrap_or(None);
                if removed.is_some() && ct_map.get(context).map(|s| s.is_empty()).unwrap_or(false) {
                    ct_map.remove(context);
                }
                removed.is_some()
            } else {
                false
            };

            if removed {
                let ns_empty = pool
                    .get(namespace)
                    .map(|nm| nm.value().is_empty())
                    .unwrap_or(false);
                if ns_empty {
                    pool.remove(namespace);
                }
            }

            removed
        }
    }
}

pub(crate) fn add_namespace(namespace: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let nm_key: NamespaceKey = Arc::from(namespace);
            map.entry(nm_key).or_insert_with(HashMap::new);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let nm_key: NamespaceKey = Arc::from(namespace);
            pool.entry(nm_key).or_insert_with(DashMap::new);
        }
    }
}

pub(crate) fn remove_namespace(namespace: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.remove(namespace);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.remove(namespace);
        }
    }
}

pub(crate) fn replace_namespace(old_namespace: &str, new_namespace: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            if let Some(ct_map) = map.remove(old_namespace) {
                let nm_key: NamespaceKey = Arc::from(new_namespace);
                map.insert(nm_key, ct_map);
            }
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            if let Some((_, ct_map)) = pool.remove(old_namespace) {
                let nm_key: NamespaceKey = Arc::from(new_namespace);
                pool.insert(nm_key, ct_map);
            }
        }
    }
}

pub(crate) fn random_uuid(
    namespace: &str,
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

    if try_insert(namespace, context, new_uuid) {
        Ok(new_uuid)
    } else {
        random_uuid(namespace, context, base, max_retries, retry_count + 1)
    }
}

pub(crate) fn add_uuid_to_pool(
    namespace: &str,
    context: &str,
    uuid: &Uuid,
) -> Result<(), UuidPoolError> {
    if !try_insert(namespace, context, *uuid) {
        return Err(UuidPoolError::FailedToAddUuidToPoolError(format!(
            "Failed to add UUID to pool for namespace-context '{}':'{}': {}",
            namespace, context, uuid
        )));
    }

    Ok(())
}

pub(crate) fn remove_uuid_from_pool(
    namespace: &str,
    context: &str,
    uuid: &Uuid,
) -> Result<(), UuidPoolError> {
    match remove(namespace, context, *uuid) {
        true => Ok(()),
        false => Err(UuidPoolError::FailedToRemoveUuidFromPoolError(
            "Failed to locate/remove UUID in pool".to_string(),
        )),
    }
}

pub(crate) fn replace_uuid_in_pool(
    namespace: &str,
    context: &str,
    old_uuid: &Uuid,
    new_uuid: &Uuid,
) -> Result<(), UuidPoolError> {
    match remove(namespace, context, *old_uuid) {
        true => {
            if !try_insert(namespace, context, *new_uuid) {
                return Err(UuidPoolError::FailedToReplaceUuidInPoolError(format!(
                    "Failed to insert new UUID in pool for namespace-context '{}':'{}': {}",
                    namespace, context, new_uuid
                )));
            }
        }
        false => {
            return Err(UuidPoolError::FailedToFindUuidInPoolError(format!(
                "Failed to find UUID in pool for namespace-context '{}':'{}': {}",
                namespace, context, old_uuid
            )));
        }
    }

    Ok(())
}

pub(crate) fn get_context_entries(
    namespace: &str,
    context: &str,
) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let nm_map = pool.lock();
            nm_map
                .get(namespace)
                .and_then(|ct_map| ct_map.get(context))
                .map(|set| set.iter().map(|uuid| (namespace.to_string(), context.to_string(), *uuid)).collect())
                .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find UUIDs in pool for namespace-context '{}':'{}'",
                    namespace, context
                )))
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => pool
            .get(namespace)
            .and_then(|nm_ref| {
                nm_ref.value().get(context).map(|set_ref| {
                    set_ref
                        .value()
                        .iter()
                        .map(|uuid| (namespace.to_string(), context.to_string(), *uuid))
                        .collect()
                })
            })
            .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                "Failed to find UUIDs in pool for namespace-context '{}':'{}'",
                namespace, context
            ))),
    }
}

pub(crate) fn get_namespace_entries(
    namespace: &str,
) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let pool_guard = pool.lock();
            pool_guard
                .get(namespace)
                .map(|ct_map| {
                    ct_map
                        .iter()
                        .flat_map(|(ctx, ids)| {
                            ids.iter().map(move |id| (namespace.to_string(), ctx.to_string(), *id))
                        })
                        .collect()
                })
                .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find '{}' namespace in pool",
                    namespace
                )))
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let Some(nm_ref) = pool.get(namespace) else {
                return Err(UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find '{}' namespace in pool",
                    namespace
                )));
            };
            let mut pairs = Vec::new();
            for ct_entry in nm_ref.value().iter() {
                let ctx = ct_entry.key().to_string();
                for uuid in ct_entry.value().iter() {
                    pairs.push((namespace.to_string(), ctx.clone(), *uuid));
                }
            }
            Ok(pairs)
        }
    }
}

pub(crate) fn get_all_namespace_entries() -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let pool_guard = pool.lock();
            Ok(pool_guard
                .iter()
                .flat_map(move |(nm, ct_map)| {
                    ct_map
                        .iter()
                        .flat_map(|(ctx, ids)| ids.iter().map( {
                            let namespace = nm.to_string();
                            move |id| (namespace.clone(), ctx.to_string(), *id)}))
                })
                .collect())
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let mut pairs = Vec::new();
            for nm_entry in pool.iter() {
                let namespace = nm_entry.key().to_string();
                for ct_entry in nm_entry.value().iter() {
                    let ctx = ct_entry.key().to_string();
                    for uuid in ct_entry.value().iter() {
                        pairs.push((namespace.clone(), ctx.clone(), *uuid));
                    }
                }
            }
            Ok(pairs)
        }
    }
}

pub(crate) fn list_namespaces() -> Vec<String> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.keys().map(|k| k.to_string()).collect()
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.iter().map(|entry| entry.key().to_string()).collect()
        }
    }
}

pub(crate) fn list_contexts(namespace: &str) -> Vec<String> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.get(namespace)
                .map(|ct_map| ct_map.keys().map(|k| k.to_string()).collect())
                .unwrap_or_default()
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => pool
            .get(namespace)
            .map(|nm_ref| {
                nm_ref
                    .value()
                    .iter()
                    .map(|e| e.key().to_string())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

pub(crate) fn list_ids(namespaces: &str, contexts: &str) -> Vec<Uuid> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let map = pool.lock();
            map.get(namespaces)
                .and_then(|ct_map| ct_map.get(contexts))
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default()
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            if let Some(nm_ref) = pool.get(namespaces) {
                if let Some(ctx_ref) = nm_ref.value().get(contexts) {
                    ctx_ref.value().iter().map(|r| *r).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
    }   
}

pub(crate) fn clear_namespace(namespace: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            map.remove(namespace);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.remove(namespace);
        }
    }
}

pub(crate) fn clear_all_namespaces() {
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

pub(crate) fn clear_context(namespace: &str, context: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            if let Some(ct_map) = map.get_mut(namespace) {
                ct_map.remove(context);
            }
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            if let Some(nm_ref) = pool.get(namespace) {
                nm_ref.value().remove(context);
            }
        }
    }
}

pub(crate) fn clear_all_contexts(namespace: &str) {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            if let Some(ct_map) = map.get_mut(namespace) {
                ct_map.clear();
            }
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            if let Some(nm_ref) = pool.get(namespace) {
                nm_ref.value().clear();
            }
        }
    }
}

pub(crate) fn drain_namespace(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let ct_map =
                map.remove(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find '{}' namespace in pool",
                        namespace
                    )))?;
            Ok(ct_map
                .into_iter()
                .flat_map(|(ctx, ids)| {
                    let nm = namespace.to_string();
                    let ctx = ctx.to_string();
                    ids.into_iter().map(move |id| (nm.clone(), ctx.clone(), id))
                })
                .collect())
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let (_, ct_map) =
                pool.remove(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find '{}' namespace in pool",
                        namespace
                    )))?;
            let mut pairs = Vec::new();
            for ct_entry in ct_map.iter() {
                let ctx = ct_entry.key().to_string();
                for uuid in ct_entry.value().iter() {
                    pairs.push((namespace.to_string(), ctx.clone(), *uuid));
                }
            }
            Ok(pairs)
        }
    }
}

pub(crate) fn drain_all_namespaces() -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let mut pairs = Vec::new();
            for (nm, ct_map) in map.drain() {
                let nm = nm.to_string();
                for (ctx, ids) in ct_map {
                    let ctx = ctx.to_string();
                    for id in ids {
                        pairs.push((nm.clone(), ctx.clone(), id));
                    }
                }
            }
            Ok(pairs)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let keys: Vec<NamespaceKey> = pool.iter().map(|e| e.key().clone()).collect();
            let mut pairs = Vec::new();
            for key in keys {
                if let Some((nm, ct_map)) = pool.remove(&*key) {
                    let nm = nm.to_string();
                    for ct_entry in ct_map.iter() {
                        let ctx = ct_entry.key().to_string();
                        for uuid in ct_entry.value().iter() {
                            pairs.push((nm.clone(), ctx.clone(), *uuid));
                        }
                    }
                }
            }
            Ok(pairs)
        }
    }
}

pub(crate) fn drain_context(
    namespace: &str,
    context: &str,
) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let ct_map =
                map.get_mut(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find namespace '{}' in pool",
                        namespace
                    )))?;
            let ids =
                ct_map
                    .remove(context)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find context '{}' in namespace '{}' in pool",
                        context, namespace
                    )))?;
            Ok(ids.into_iter().map(|id| (context.to_string(), id)).collect())
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let nm_ref =
                pool.get(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find namespace '{}' in pool",
                        namespace
                    )))?;
            let (_, set) = nm_ref.value().remove(context).ok_or(
                UuidPoolError::FailedToFindUuidInPoolError(format!(
                    "Failed to find context '{}' in namespace '{}' in pool",
                    context, namespace
                )),
            )?;
            Ok(set.iter().map(|uuid| (context.to_string(), *uuid)).collect())
        }
    }
}

pub(crate) fn drain_all_contexts(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let ct_map =
                map.get_mut(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find namespace '{}' in pool",
                        namespace
                    )))?;
            let mut pairs = Vec::new();
            for (ctx, ids) in ct_map.drain() {
                let ctx = ctx.to_string();
                for id in ids {
                    pairs.push((namespace.to_string(), ctx.clone(), id));
                }
            }
            Ok(pairs)
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let nm_ref =
                pool.get(namespace)
                    .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                        "Failed to find namespace '{}' in pool",
                        namespace
                    )))?;
            let ct_map = nm_ref.value();
            let ctx_keys: Vec<ContextKey> = ct_map.iter().map(|e| e.key().clone()).collect();
            let mut pairs = Vec::new();
            for key in ctx_keys {
                if let Some((ctx, set)) = ct_map.remove(&*key) {
                    let ctx = ctx.to_string();
                    for uuid in set.iter() {
                        pairs.push((namespace.to_string(), ctx.clone(), *uuid));
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

    #[test]
    fn add_random_uuid() -> Result<(), UuidPoolError> {
        let ns = "test_add_random";
        remove_namespace(ns);

        let random_id = random_uuid(ns, "ctx", 67, 10, 0)?;
        let stored_id_vec = get_context_uuids_from_pool(ns, "ctx")?;

        assert!(stored_id_vec.contains(&("ctx".to_string(), random_id)));
        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn add_different_context_same_uuid() -> Result<(), UuidPoolError> {
        let ns = "test_add_diff_ctx";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx_a", 69, 10, 0)?;

        match add_uuid_to_pool(ns, "ctx_b", &id1) {
            Ok(()) => assert!(true, "Same UUID added to two different contexts."),
            Err(e) => assert!(false, "{}", e.to_string()),
        }

        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn add_same_context_same_uuid() -> Result<(), UuidPoolError> {
        let ns = "test_add_same_ctx";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx", 420, 10, 0)?;

        match add_uuid_to_pool(ns, "ctx", &id1) {
            Ok(()) => assert!(
                false,
                "The same context should not be able to hold the same UUID twice"
            ),
            Err(e) => assert!(true, "{}", e.to_string()),
        }

        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn remove_uuid() -> Result<(), UuidPoolError> {
        let ns = "test_remove_uuid";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx", 69, 10, 0)?;

        match remove_uuid_from_pool(ns, "ctx", &id1) {
            Ok(()) => assert!(true),
            Err(e) => assert!(false, "{}", e.to_string()),
        }

        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn replace_uuid() -> Result<(), UuidPoolError> {
        let ns = "test_replace_uuid";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx", 117, 10, 0)?;
        let id2 = make_uuid_with_base(67);

        match replace_uuid_in_pool(ns, "ctx", &id1, &id2) {
            Ok(()) => assert!(true),
            Err(e) => assert!(false, "{}", e.to_string()),
        }

        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn list_context() -> Result<(), UuidPoolError> {
        let ns = "test_list_context";
        remove_namespace(ns);

        let context_list = [
            "context_test1",
            "context_test2",
            "context_test3",
            "context_test4",
            "context_test5",
        ];
        for context in context_list {
            let _ = random_uuid(ns, context, 117, 10, 0)?;
        }

        let stored_contexts = list_contexts(ns);

        for context in context_list {
            if stored_contexts.contains(&context.to_string()) {
                continue;
            } else {
                assert!(false, "{} not found in list of contexts", context);
            }
        }

        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn get_context_uuids() -> Result<(), UuidPoolError> {
        let ns = "test_get_ctx_uuids";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx", 42, 10, 0)?;
        let id2 = random_uuid(ns, "ctx", 42, 10, 0)?;

        let pairs = get_context_uuids_from_pool(ns, "ctx")?;

        assert!(pairs.contains(&("ctx".to_string(), id1)));
        assert!(pairs.contains(&("ctx".to_string(), id2)));
        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn get_all_contexts() -> Result<(), UuidPoolError> {
        let ns = "test_get_all_ctx";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx_a", 42, 10, 0)?;
        let id2 = random_uuid(ns, "ctx_b", 42, 10, 0)?;

        let pairs = get_all_contexts_uuids_from_namespace(ns)?;

        assert!(pairs.contains(&("ctx_a".to_string(), id1)));
        assert!(pairs.contains(&("ctx_b".to_string(), id2)));
        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn drain_context_uuids() -> Result<(), UuidPoolError> {
        let ns = "test_drain_ctx";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx", 42, 10, 0)?;
        let id2 = random_uuid(ns, "ctx", 42, 10, 0)?;

        let drained = drain_context(ns, "ctx")?;

        assert!(drained.contains(&("ctx".to_string(), id1)));
        assert!(drained.contains(&("ctx".to_string(), id2)));
        assert!(get_context_uuids_from_pool(ns, "ctx").is_err());
        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn drain_all_contexts_test() -> Result<(), UuidPoolError> {
        let ns = "test_drain_all_ctx";
        remove_namespace(ns);

        let id1 = random_uuid(ns, "ctx_a", 42, 10, 0)?;
        let id2 = random_uuid(ns, "ctx_b", 42, 10, 0)?;

        let drained = drain_all_contexts(ns)?;

        assert!(drained.contains(&(ns.to_string(), "ctx_a".to_string(), id1)));
        assert!(drained.contains(&(ns.to_string(), "ctx_b".to_string(), id2)));
        assert!(list_contexts(ns).is_empty());
        Ok(())
    }

    #[test]
    fn clear_context_test() -> Result<(), UuidPoolError> {
        let ns = "test_clear_ctx";
        remove_namespace(ns);

        let _ = random_uuid(ns, "ctx", 42, 10, 0)?;
        clear_context(ns, "ctx");

        assert!(get_context_uuids_from_pool(ns, "ctx").is_err());
        remove_namespace(ns);
        Ok(())
    }

    #[test]
    fn clear_all_contexts_test() -> Result<(), UuidPoolError> {
        let ns = "test_clear_all";
        remove_namespace(ns);

        let _ = random_uuid(ns, "ctx", 42, 10, 0)?;
        remove_namespace(ns);

        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }
}
