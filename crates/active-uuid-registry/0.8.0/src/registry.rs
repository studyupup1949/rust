use super::{UuidPoolError, NamespaceString, ContextString};

use std::collections::HashSet;
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

#[cfg(not(feature = "concurrent-map"))]
use std::collections::HashMap;

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

/// Acquires the global UUID pool instance using a lazy initialization pattern.
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

/// A key for an owned namespace.
pub(crate) type OwnedNamespaceSignature = Arc<[u8; 128]>;

static OWNED_NAMESPACE_POOL: OnceLock<OwnedNamespaces> = OnceLock::new();

enum OwnedNamespaces {
    #[cfg(not(feature = "concurrent-map"))]
    SingleThreaded(parking_lot::Mutex<HashMap<NamespaceKey, OwnedNamespaceSignature>>),
    #[cfg(feature = "concurrent-map")]
    Concurrent(DashMap<NamespaceKey, OwnedNamespaceSignature>),
}

/// Stores namespaces that are owned and require a key to be written to.
fn owned_namespaces() -> &'static OwnedNamespaces {
    OWNED_NAMESPACE_POOL.get_or_init(|| {
        #[cfg(not(feature = "concurrent-map"))]
        {
            OwnedNamespaces::SingleThreaded(parking_lot::Mutex::new(HashMap::new()))
        }
        #[cfg(feature = "concurrent-map")]
        {
            OwnedNamespaces::Concurrent(DashMap::new())
        }
    })
}

/// Global read/write gate that serializes namespace-ownership lifecycle changes
/// (claiming, releasing, renaming) against every authorized write, in both storage
/// backends. Regular per-context/per-UUID writes take a shared (read) guard so they
/// can proceed concurrently with each other; ownership lifecycle operations take an
/// exclusive (write) guard so no write can observe a half-completed claim/release/rename
/// and no claim can race a write's authorization check.
static OWNERSHIP_GATE: OnceLock<parking_lot::RwLock<()>> = OnceLock::new();

fn ownership_gate() -> &'static parking_lot::RwLock<()> {
    OWNERSHIP_GATE.get_or_init(|| parking_lot::RwLock::new(()))
}

/// The credential presented by a caller attempting to write to a namespace.
#[derive(Clone, Copy)]
pub(crate) enum WriteAccess<'a> {
    /// No ownership signature is presented. Grants access to namespaces that are not owned.
    Unowned,
    /// An ownership signature is presented. Grants access to namespaces owned under a matching signature.
    Owned(&'a OwnedNamespaceSignature),
}

impl<'a> WriteAccess<'a> {
    pub(crate) fn owned(signature: &'a OwnedNamespaceSignature) -> Self {
        WriteAccess::Owned(signature)
    }
}

/// Generates a new UUID with a custom base.
pub(crate) fn make_uuid_with_base(base: u32) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&base.to_be_bytes());
    for i in bytes.iter_mut().skip(4) {
        *i = rand::random_range(0..=255);
    }
    Uuid::new_v8(bytes)
}

fn make_owner_signature() -> OwnedNamespaceSignature {
    let mut byte_array = [0u8; 128];
    rand::fill(&mut byte_array);

    Arc::from(byte_array)
}

fn signature_matches(stored: &OwnedNamespaceSignature, provided: &OwnedNamespaceSignature) -> bool {
    Arc::ptr_eq(stored, provided) || **stored == **provided
}

fn unauthorized(namespace: &str) -> UuidPoolError {
    UuidPoolError::UnauthorizedNamespaceWriteAccessError(format!(
        "Unauthorized write access to namespace '{}'",
        namespace
    ))
}

/// Checks whether `access` authorizes a write to `namespace`.
///
/// A namespace with no ownership record grants access to anyone (`WriteAccess::Unowned` or
/// `WriteAccess::Owned`). A namespace with an ownership record only grants access to a
/// presented signature that matches the stored one.
fn check_write_access(namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => {
            let map = map.lock();
            match map.get(namespace) {
                None => Ok(()),
                Some(stored) => match access {
                    WriteAccess::Owned(provided) if signature_matches(stored, provided) => Ok(()),
                    _ => Err(unauthorized(namespace)),
                },
            }
        }
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => match map.get(namespace) {
            None => Ok(()),
            Some(stored) => match access {
                WriteAccess::Owned(provided) if signature_matches(stored.value(), provided) => Ok(()),
                _ => Err(unauthorized(namespace)),
            },
        },
    }
}

/// Runs `mutate` while holding a shared ownership-gate guard, after verifying `access`
/// authorizes writing to `namespace`. Safe for context/UUID-scoped mutations that do not
/// themselves alter the ownership map.
fn with_shared_access<T>(
    namespace: &str,
    access: WriteAccess,
    mutate: impl FnOnce() -> Result<T, UuidPoolError>,
) -> Result<T, UuidPoolError> {
    let _guard = ownership_gate().read();
    check_write_access(namespace, access)?;
    mutate()
}

/// Runs `mutate` while holding an exclusive ownership-gate guard, after verifying `access`
/// authorizes writing to `namespace`. Required for namespace-lifecycle mutations (create,
/// remove, rename) that may also alter the ownership map, and for anything that must be
/// consistent with in-flight claims/releases/renames.
fn with_exclusive_access<T>(
    namespace: &str,
    access: WriteAccess,
    mutate: impl FnOnce() -> Result<T, UuidPoolError>,
) -> Result<T, UuidPoolError> {
    let _guard = ownership_gate().write();
    check_write_access(namespace, access)?;
    mutate()
}

fn is_namespace_owned(namespace: &str) -> bool {
    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => map.lock().contains_key(namespace),
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => map.contains_key(namespace),
    }
}

fn is_namespace_empty_or_absent(namespace: &str) -> bool {
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => pool
            .lock()
            .get(namespace)
            .map(|ct_map| ct_map.is_empty())
            .unwrap_or(true),
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => pool
            .get(namespace)
            .map(|nm_ref| nm_ref.value().is_empty())
            .unwrap_or(true),
    }
}

/// Verifies that `namespace` may be claimed as an owned namespace: it must not already be
/// owned, and must be either absent from the pool or present with no data.
fn ensure_claimable(namespace: &str) -> Result<(), UuidPoolError> {
    if is_namespace_owned(namespace) {
        return Err(UuidPoolError::FailedToClaimNamespaceError(format!(
            "Namespace '{}' is already owned",
            namespace
        )));
    }
    if !is_namespace_empty_or_absent(namespace) {
        return Err(UuidPoolError::FailedToClaimNamespaceError(format!(
            "Namespace '{}' already contains data and cannot be claimed",
            namespace
        )));
    }
    Ok(())
}

fn owned_namespace_keys() -> HashSet<NamespaceKey> {
    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => map.lock().keys().cloned().collect(),
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => map.iter().map(|e| e.key().clone()).collect(),
    }
}

fn remove_ownership_record(namespace: &str) {
    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => {
            map.lock().remove(namespace);
        }
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => {
            map.remove(namespace);
        }
    }
}

/// Moves an ownership record from `old_namespace` to `new_namespace`, if one exists.
/// No-op if `old_namespace` is unowned.
fn transfer_ownership_record(old_namespace: &str, new_namespace: &str) {
    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => {
            let mut map = map.lock();
            if let Some(signature) = map.remove(old_namespace) {
                let nm_key: NamespaceKey = Arc::from(new_namespace);
                map.insert(nm_key, signature);
            }
        }
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => {
            if let Some((_, signature)) = map.remove(old_namespace) {
                let nm_key: NamespaceKey = Arc::from(new_namespace);
                map.insert(nm_key, signature);
            }
        }
    }
}

fn try_insert(namespace: &str, context: &str, uuid: Uuid, access: WriteAccess) -> Result<bool, UuidPoolError> {
    with_shared_access(namespace, access, || {
        Ok(match global_pool() {
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
        })
    })
}

fn remove(namespace: &str, context: &str, uuid: Uuid, access: WriteAccess) -> Result<bool, UuidPoolError> {
    with_shared_access(namespace, access, || {
        Ok(match global_pool() {
            #[cfg(not(feature = "concurrent-map"))]
            GlobalUuidPool::SingleThreaded(pool) => {
                let mut nm_map = pool.lock();

                let removed = {
                    let Some(ct_map) = nm_map.get_mut(namespace) else {
                        return Ok(false);
                    };
                    let Some(set) = ct_map.get_mut(context) else {
                        return Ok(false);
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
        })
    })
}

enum ReplaceOutcome {
    Replaced,
    OldNotFound,
    NewAlreadyExists,
}

/// Removes `old_uuid` and inserts `new_uuid` within a single authorized critical section,
/// so a failed insert cannot leave the context missing both UUIDs: the old UUID is
/// restored if the new one already exists.
fn replace(
    namespace: &str,
    context: &str,
    old_uuid: Uuid,
    new_uuid: Uuid,
    access: WriteAccess,
) -> Result<ReplaceOutcome, UuidPoolError> {
    with_shared_access(namespace, access, || {
        Ok(match global_pool() {
            #[cfg(not(feature = "concurrent-map"))]
            GlobalUuidPool::SingleThreaded(pool) => {
                let mut map = pool.lock();
                let Some(ct_map) = map.get_mut(namespace) else {
                    return Ok(ReplaceOutcome::OldNotFound);
                };
                let Some(set) = ct_map.get_mut(context) else {
                    return Ok(ReplaceOutcome::OldNotFound);
                };
                if !set.remove(&old_uuid) {
                    return Ok(ReplaceOutcome::OldNotFound);
                }
                if !set.insert(new_uuid) {
                    set.insert(old_uuid);
                    return Ok(ReplaceOutcome::NewAlreadyExists);
                }
                ReplaceOutcome::Replaced
            }
            #[cfg(feature = "concurrent-map")]
            GlobalUuidPool::Concurrent(pool) => {
                let Some(nm_ref) = pool.get(namespace) else {
                    return Ok(ReplaceOutcome::OldNotFound);
                };
                let ct_map = nm_ref.value();
                let Some(set_ref) = ct_map.get(context) else {
                    return Ok(ReplaceOutcome::OldNotFound);
                };
                let set = set_ref.value();
                if set.remove(&old_uuid).is_none() {
                    return Ok(ReplaceOutcome::OldNotFound);
                }
                if !set.insert(new_uuid) {
                    set.insert(old_uuid);
                    return Ok(ReplaceOutcome::NewAlreadyExists);
                }
                ReplaceOutcome::Replaced
            }
        })
    })
}

/// Claims `namespace` as an owned namespace, returning the newly generated ownership
/// signature. Fails if the namespace is already owned or already contains data.
pub(crate) fn add_owned_namespace(namespace: &str) -> Result<OwnedNamespaceSignature, UuidPoolError> {
    let _guard = ownership_gate().write();
    ensure_claimable(namespace)?;

    let nm_key: NamespaceKey = Arc::from(namespace);
    let new_owner_signature = make_owner_signature();

    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            pool.lock().entry(nm_key.clone()).or_insert_with(HashMap::new);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.entry(nm_key.clone()).or_insert_with(DashMap::new);
        }
    }

    match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(owned_map) => {
            owned_map.lock().insert(nm_key, new_owner_signature.clone());
        }
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(owned_map) => {
            owned_map.insert(nm_key, new_owner_signature.clone());
        }
    }

    Ok(new_owner_signature)
}

/// Releases ownership of `namespace` and removes its data, but only if the stored
/// ownership signature still matches `signature`. No-op (returns `Ok`) if the namespace
/// is unowned or owned under a different signature, so a stale handle can never delete
/// data belonging to a namespace that has since been released and re-claimed by another
/// owner. Used by [`crate::raii::OwnedNamespace`]'s automatic cleanup on drop.
pub(crate) fn release_owned_namespace(namespace: &str, signature: &OwnedNamespaceSignature) -> Result<(), UuidPoolError> {
    let _guard = ownership_gate().write();

    let owns = match owned_namespaces() {
        #[cfg(not(feature = "concurrent-map"))]
        OwnedNamespaces::SingleThreaded(map) => map
            .lock()
            .get(namespace)
            .map(|stored| signature_matches(stored, signature))
            .unwrap_or(false),
        #[cfg(feature = "concurrent-map")]
        OwnedNamespaces::Concurrent(map) => map
            .get(namespace)
            .map(|stored| signature_matches(stored.value(), signature))
            .unwrap_or(false),
    };

    if !owns {
        return Ok(());
    }

    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            pool.lock().remove(namespace);
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.remove(namespace);
        }
    }

    remove_ownership_record(namespace);
    Ok(())
}

pub(crate) fn add_namespace(namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_exclusive_access(namespace, access, || {
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
        Ok(())
    })
}

pub(crate) fn remove_namespace(namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_exclusive_access(namespace, access, || {
        match global_pool() {
            #[cfg(not(feature = "concurrent-map"))]
            GlobalUuidPool::SingleThreaded(pool) => {
                pool.lock().remove(namespace);
            }
            #[cfg(feature = "concurrent-map")]
            GlobalUuidPool::Concurrent(pool) => {
                pool.remove(namespace);
            }
        }
        remove_ownership_record(namespace);
        Ok(())
    })
}

pub(crate) fn replace_namespace(old_namespace: &str, new_namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_exclusive_access(old_namespace, access, || {
        // Renaming to the same name is always allowed; only a genuinely different
        // destination needs to be validated as claimable.
        if new_namespace != old_namespace {
            ensure_claimable(new_namespace)?;
        }

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

        transfer_ownership_record(old_namespace, new_namespace);
        Ok(())
    })
}

pub(crate) fn random_uuid(
    namespace: &str,
    context: &str,
    base: u32,
    max_retries: usize,
    retry_count: usize,
    access: WriteAccess,
) -> Result<Uuid, UuidPoolError> {
    if retry_count >= max_retries {
        return Err(UuidPoolError::FailedToGenerateUniqueUuidError(format!(
            "Failed to generate unique UUID after {} attempts",
            max_retries
        )));
    }

    let new_uuid = make_uuid_with_base(base);

    if try_insert(namespace, context, new_uuid, access)? {
        Ok(new_uuid)
    } else {
        random_uuid(namespace, context, base, max_retries, retry_count + 1, access)
    }
}

pub(crate) fn add_uuid_to_pool(
    namespace: &str,
    context: &str,
    uuid: &Uuid,
    access: WriteAccess,
) -> Result<(), UuidPoolError> {
    if !try_insert(namespace, context, *uuid, access)? {
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
    access: WriteAccess,
) -> Result<(), UuidPoolError> {
    match remove(namespace, context, *uuid, access)? {
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
    access: WriteAccess,
) -> Result<(), UuidPoolError> {
    match replace(namespace, context, *old_uuid, *new_uuid, access)? {
        ReplaceOutcome::Replaced => Ok(()),
        ReplaceOutcome::OldNotFound => Err(UuidPoolError::FailedToFindUuidInPoolError(format!(
            "Failed to find UUID in pool for namespace-context '{}':'{}': {}",
            namespace, context, old_uuid
        ))),
        ReplaceOutcome::NewAlreadyExists => Err(UuidPoolError::FailedToReplaceUuidInPoolError(format!(
            "Failed to insert new UUID in pool for namespace-context '{}':'{}': {}",
            namespace, context, new_uuid
        ))),
    }
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

pub(crate) fn clear_namespace(namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_exclusive_access(namespace, access, || {
        match global_pool() {
            #[cfg(not(feature = "concurrent-map"))]
            GlobalUuidPool::SingleThreaded(pool) => {
                pool.lock().remove(namespace);
            }
            #[cfg(feature = "concurrent-map")]
            GlobalUuidPool::Concurrent(pool) => {
                pool.remove(namespace);
            }
        }
        remove_ownership_record(namespace);
        Ok(())
    })
}

/// Clears every unowned namespace and all of its data. Owned namespaces are preserved.
pub(crate) fn clear_all_namespaces() {
    let _guard = ownership_gate().write();
    let owned_keys = owned_namespace_keys();
    match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            pool.lock().retain(|k, _| owned_keys.contains(k));
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            pool.retain(|k, _| owned_keys.contains(k));
        }
    }
}

pub(crate) fn clear_context(namespace: &str, context: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_shared_access(namespace, access, || {
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
        Ok(())
    })
}

pub(crate) fn clear_all_contexts(namespace: &str, access: WriteAccess) -> Result<(), UuidPoolError> {
    with_shared_access(namespace, access, || {
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
        Ok(())
    })
}

pub(crate) fn drain_namespace(
    namespace: &str,
    access: WriteAccess,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    with_exclusive_access(namespace, access, || {
        let pairs = match global_pool() {
            #[cfg(not(feature = "concurrent-map"))]
            GlobalUuidPool::SingleThreaded(pool) => {
                let mut map = pool.lock();
                let ct_map =
                    map.remove(namespace)
                        .ok_or(UuidPoolError::FailedToFindUuidInPoolError(format!(
                            "Failed to find '{}' namespace in pool",
                            namespace
                        )))?;
                ct_map
                    .into_iter()
                    .flat_map(|(ctx, ids)| {
                        let nm = namespace.to_string();
                        let ctx = ctx.to_string();
                        ids.into_iter().map(move |id| (nm.clone(), ctx.clone(), id))
                    })
                    .collect()
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
                pairs
            }
        };

        remove_ownership_record(namespace);
        Ok(pairs)
    })
}

/// Drains every unowned namespace and all of its data. Owned namespaces are preserved.
pub(crate) fn drain_all_namespaces() -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    let _guard = ownership_gate().write();
    let owned_keys = owned_namespace_keys();
    Ok(match global_pool() {
        #[cfg(not(feature = "concurrent-map"))]
        GlobalUuidPool::SingleThreaded(pool) => {
            let mut map = pool.lock();
            let drain_keys: Vec<NamespaceKey> = map
                .keys()
                .filter(|k| !owned_keys.contains(*k))
                .cloned()
                .collect();
            let mut pairs = Vec::new();
            for key in drain_keys {
                if let Some(ct_map) = map.remove(&key) {
                    let nm = key.to_string();
                    for (ctx, ids) in ct_map {
                        let ctx = ctx.to_string();
                        for id in ids {
                            pairs.push((nm.clone(), ctx.clone(), id));
                        }
                    }
                }
            }
            pairs
        }
        #[cfg(feature = "concurrent-map")]
        GlobalUuidPool::Concurrent(pool) => {
            let keys: Vec<NamespaceKey> = pool
                .iter()
                .map(|e| e.key().clone())
                .filter(|k| !owned_keys.contains(k))
                .collect();
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
            pairs
        }
    })
}

pub(crate) fn drain_context(
    namespace: &str,
    context: &str,
    access: WriteAccess,
) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    with_shared_access(namespace, access, || {
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
    })
}

pub(crate) fn drain_all_contexts(
    namespace: &str,
    access: WriteAccess,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    with_shared_access(namespace, access, || {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNOWNED: WriteAccess<'static> = WriteAccess::Unowned;

    /// All tests share the same process-wide registry singleton. Some tests exercise truly
    /// global operations (`clear_all_namespaces`), which would otherwise be free to wipe out
    /// data belonging to unrelated tests running concurrently on other threads. Serializing
    /// every test through this lock keeps the suite deterministic regardless of test-harness
    /// parallelism.
    static TEST_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

    #[test]
    fn add_random_uuid() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_add_random";
        let _ = remove_namespace(ns, UNOWNED);

        let random_id = random_uuid(ns, "ctx", 67, 10, 0, UNOWNED)?;
        let stored_id_vec = get_context_entries(ns, "ctx")?;

        assert!(stored_id_vec.contains(&(ns.to_string(), "ctx".to_string(), random_id)));
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn add_different_context_same_uuid() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_add_diff_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx_a", 69, 10, 0, UNOWNED)?;

        add_uuid_to_pool(ns, "ctx_b", &id1, UNOWNED).expect("same UUID should be addable to a different context");

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn add_same_context_same_uuid() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_add_same_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 420, 10, 0, UNOWNED)?;

        assert!(
            add_uuid_to_pool(ns, "ctx", &id1, UNOWNED).is_err(),
            "the same context should not be able to hold the same UUID twice"
        );

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn remove_uuid() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_remove_uuid";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 69, 10, 0, UNOWNED)?;

        remove_uuid_from_pool(ns, "ctx", &id1, UNOWNED).expect("UUID should be removable");

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn replace_uuid() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_replace_uuid";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 117, 10, 0, UNOWNED)?;
        let id2 = make_uuid_with_base(67);

        replace_uuid_in_pool(ns, "ctx", &id1, &id2, UNOWNED).expect("UUID should be replaceable");

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn replace_uuid_missing_old_restores_nothing() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_replace_missing_old";
        let _ = remove_namespace(ns, UNOWNED);

        let missing = make_uuid_with_base(1);
        let new_id = make_uuid_with_base(2);
        add_namespace(ns, UNOWNED)?;

        assert!(replace_uuid_in_pool(ns, "ctx", &missing, &new_id, UNOWNED).is_err());
        assert!(get_context_entries(ns, "ctx").is_err());

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn replace_uuid_conflicting_new_restores_old() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_replace_conflict";
        let _ = remove_namespace(ns, UNOWNED);

        let old_id = random_uuid(ns, "ctx", 3, 10, 0, UNOWNED)?;
        let new_id = random_uuid(ns, "ctx", 4, 10, 0, UNOWNED)?;

        assert!(replace_uuid_in_pool(ns, "ctx", &old_id, &new_id, UNOWNED).is_err());
        let entries = get_context_entries(ns, "ctx")?;
        assert!(entries.contains(&(ns.to_string(), "ctx".to_string(), old_id)));
        assert!(entries.contains(&(ns.to_string(), "ctx".to_string(), new_id)));

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn list_context() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_list_context";
        let _ = remove_namespace(ns, UNOWNED);

        let context_list = [
            "context_test1",
            "context_test2",
            "context_test3",
            "context_test4",
            "context_test5",
        ];
        for context in context_list {
            let _ = random_uuid(ns, context, 117, 10, 0, UNOWNED)?;
        }

        let stored_contexts = list_contexts(ns);

        for context in context_list {
            assert!(
                stored_contexts.contains(&context.to_string()),
                "{} not found in list of contexts",
                context
            );
        }

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn get_context_uuids() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_get_ctx_uuids";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        let id2 = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;

        let pairs = get_context_entries(ns, "ctx")?;

        assert!(pairs.contains(&(ns.to_string(), "ctx".to_string(), id1)));
        assert!(pairs.contains(&(ns.to_string(), "ctx".to_string(), id2)));
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn get_all_contexts() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_get_all_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx_a", 42, 10, 0, UNOWNED)?;
        let id2 = random_uuid(ns, "ctx_b", 42, 10, 0, UNOWNED)?;

        let pairs = get_namespace_entries(ns)?;

        assert!(pairs.contains(&(ns.to_string(), "ctx_a".to_string(), id1)));
        assert!(pairs.contains(&(ns.to_string(), "ctx_b".to_string(), id2)));
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn drain_context_uuids() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_drain_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        let id2 = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;

        let drained = drain_context(ns, "ctx", UNOWNED)?;

        assert!(drained.contains(&("ctx".to_string(), id1)));
        assert!(drained.contains(&("ctx".to_string(), id2)));
        assert!(get_context_entries(ns, "ctx").is_err());
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn drain_all_contexts_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_drain_all_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx_a", 42, 10, 0, UNOWNED)?;
        let id2 = random_uuid(ns, "ctx_b", 42, 10, 0, UNOWNED)?;

        let drained = drain_all_contexts(ns, UNOWNED)?;

        assert!(drained.contains(&(ns.to_string(), "ctx_a".to_string(), id1)));
        assert!(drained.contains(&(ns.to_string(), "ctx_b".to_string(), id2)));
        assert!(list_contexts(ns).is_empty());
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn clear_context_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_clear_ctx";
        let _ = remove_namespace(ns, UNOWNED);

        let _ = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        clear_context(ns, "ctx", UNOWNED)?;

        assert!(get_context_entries(ns, "ctx").is_err());
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn clear_all_contexts_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_clear_all_contexts";
        let _ = remove_namespace(ns, UNOWNED);

        let _ = random_uuid(ns, "ctx_a", 42, 10, 0, UNOWNED)?;
        let _ = random_uuid(ns, "ctx_b", 42, 10, 0, UNOWNED)?;

        clear_all_contexts(ns, UNOWNED)?;

        assert!(list_contexts(ns).is_empty());
        assert!(list_namespaces().contains(&ns.to_string()));
        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn clear_namespace_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_clear_namespace";
        let _ = remove_namespace(ns, UNOWNED);

        let _ = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        clear_namespace(ns, UNOWNED)?;

        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }

    #[test]
    fn drain_namespace_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_drain_namespace";
        let _ = remove_namespace(ns, UNOWNED);

        let id1 = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        let drained = drain_namespace(ns, UNOWNED)?;

        assert!(drained.contains(&(ns.to_string(), "ctx".to_string(), id1)));
        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }

    #[test]
    fn replace_namespace_test() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let old_ns = "test_rename_old";
        let new_ns = "test_rename_new";
        let _ = remove_namespace(old_ns, UNOWNED);
        let _ = remove_namespace(new_ns, UNOWNED);

        let id1 = random_uuid(old_ns, "ctx", 42, 10, 0, UNOWNED)?;
        replace_namespace(old_ns, new_ns, UNOWNED)?;

        assert!(!list_namespaces().contains(&old_ns.to_string()));
        let entries = get_context_entries(new_ns, "ctx")?;
        assert!(entries.contains(&(new_ns.to_string(), "ctx".to_string(), id1)));

        remove_namespace(new_ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn claim_absent_namespace_succeeds() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_claim_absent";
        let _ = release_owned_namespace(ns, &add_owned_namespace(ns).unwrap_or_else(|_| make_owner_signature()));
        let _ = remove_namespace(ns, UNOWNED);

        let signature = add_owned_namespace(ns)?;
        assert!(list_namespaces().contains(&ns.to_string()));

        release_owned_namespace(ns, &signature)?;
        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }

    #[test]
    fn claim_non_empty_namespace_fails() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_claim_non_empty";
        let _ = remove_namespace(ns, UNOWNED);

        let _ = random_uuid(ns, "ctx", 42, 10, 0, UNOWNED)?;
        assert!(add_owned_namespace(ns).is_err());

        remove_namespace(ns, UNOWNED)?;
        Ok(())
    }

    #[test]
    fn claim_already_owned_namespace_fails() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_claim_already_owned";
        let _ = remove_namespace(ns, UNOWNED);

        let signature = add_owned_namespace(ns)?;
        assert!(add_owned_namespace(ns).is_err());

        release_owned_namespace(ns, &signature)?;
        Ok(())
    }

    #[test]
    fn unauthorized_write_to_owned_namespace_is_rejected() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_owned_unauthorized";
        let _ = remove_namespace(ns, UNOWNED);

        let signature = add_owned_namespace(ns)?;

        assert!(random_uuid(ns, "ctx", 42, 10, 0, UNOWNED).is_err());
        assert!(add_namespace(ns, UNOWNED).is_err());
        assert!(remove_namespace(ns, UNOWNED).is_err());
        assert!(clear_context(ns, "ctx", UNOWNED).is_err());

        release_owned_namespace(ns, &signature)?;
        Ok(())
    }

    #[test]
    fn authorized_write_to_owned_namespace_succeeds() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_owned_authorized";
        let _ = remove_namespace(ns, UNOWNED);

        let signature = add_owned_namespace(ns)?;
        let access = WriteAccess::owned(&signature);

        let id = random_uuid(ns, "ctx", 42, 10, 0, access)?;
        let entries = get_context_entries(ns, "ctx")?;
        assert!(entries.contains(&(ns.to_string(), "ctx".to_string(), id)));

        remove_namespace(ns, access)?;
        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }

    #[test]
    fn release_with_mismatched_signature_is_noop() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let ns = "test_release_mismatch";
        let _ = remove_namespace(ns, UNOWNED);

        let signature = add_owned_namespace(ns)?;
        let bogus = make_owner_signature();

        release_owned_namespace(ns, &bogus)?;
        assert!(list_namespaces().contains(&ns.to_string()));

        release_owned_namespace(ns, &signature)?;
        assert!(!list_namespaces().contains(&ns.to_string()));
        Ok(())
    }

    #[test]
    fn bulk_operations_preserve_owned_namespaces() -> Result<(), UuidPoolError> {
        let _guard = TEST_LOCK.lock();
        let owned_ns = "test_bulk_owned";
        let unowned_ns = "test_bulk_unowned";
        let _ = remove_namespace(owned_ns, UNOWNED);
        let _ = remove_namespace(unowned_ns, UNOWNED);

        let signature = add_owned_namespace(owned_ns)?;
        let access = WriteAccess::owned(&signature);
        let owned_id = random_uuid(owned_ns, "ctx", 42, 10, 0, access)?;
        let _ = random_uuid(unowned_ns, "ctx", 42, 10, 0, UNOWNED)?;

        clear_all_namespaces();

        assert!(list_namespaces().contains(&owned_ns.to_string()));
        assert!(!list_namespaces().contains(&unowned_ns.to_string()));
        let entries = get_context_entries(owned_ns, "ctx")?;
        assert!(entries.contains(&(owned_ns.to_string(), "ctx".to_string(), owned_id)));

        remove_namespace(owned_ns, access)?;
        Ok(())
    }
}
