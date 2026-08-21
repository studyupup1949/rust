use super::{UuidPoolError, NamespaceString, ContextString};

use uuid::Uuid;

pub const DEFAULT_UUID_BASE: u32 = 64;
pub const DEFAULT_MAX_RETRIES: usize = 64;

/// Adds a new namespace to the registry.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[inline(always)]
pub fn reserve_namespace(namespace: &str) {
    crate::registry::add_namespace(namespace)
}

/// Removes a namespace and all associated contexts from the registry.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[inline(always)]
pub fn remove_namespace(namespace: &str) {
    crate::registry::remove_namespace(namespace)
}

/// Replaces a namespace with a new namespace in the registry.
/// 
/// #### Arguments
/// * `old_namespace`: old namespace
/// * `new_namespace`: new namespace
#[inline(always)]
pub fn replace_namespace(old_namespace: &str, new_namespace: &str) {
    crate::registry::replace_namespace(old_namespace, new_namespace)
}

/// Reserves a new UUID in the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve_id(namespace: &str, context: &str) -> Result<Uuid, UuidPoolError> {
    reserve_id_with(namespace, context, DEFAULT_UUID_BASE, DEFAULT_MAX_RETRIES)
}

/// Reserves a new UUID in the given namespace and context space with a custom base.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `base`: basis for UUID generation
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve_id_with_base(
    namespace: &str,
    context: &str,
    base: u32,
) -> Result<Uuid, UuidPoolError> {
    reserve_id_with(namespace, context, base, DEFAULT_MAX_RETRIES)
}

/// Reserves a new UUID in the given namespace and context space with a custom base and retry count.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `base`: basis for UUID generation
/// * `max_retries`: maximum number of retries
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve_id_with(
    namespace: &str,
    context: &str,
    base: u32,
    max_retries: usize,
) -> Result<Uuid, UuidPoolError> {
    crate::registry::random_uuid(namespace, context, base, max_retries, 0)
}

/// Adds an existing UUID to the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn add_id(namespace: &str, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::add_uuid_to_pool(namespace, context, &uuid)
}

/// Removes an existing UUID from the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn remove_id(namespace: &str, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::remove_uuid_from_pool(namespace, context, &uuid)
}

/// Tries to remove an existing UUID from the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `bool`: true if the UUID was removed, false otherwise
#[inline(always)]
pub fn try_remove_id(namespace: &str, context: &str, uuid: Uuid) -> bool {
    crate::registry::remove_uuid_from_pool(namespace, context, &uuid).is_ok()
}

/// Replaces an existing UUID with a new UUID in the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `old_uuid`: existing UUID
/// * `new_uuid`: new UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn replace_id(
    namespace: &str,
    context: &str,
    old_uuid: Uuid,
    new_uuid: Uuid,
) -> Result<(), UuidPoolError> {
    crate::registry::replace_uuid_in_pool(namespace, context, &old_uuid, &new_uuid)
}

/// Gets all context-UUID entries from a specific context space in a given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-associated UUID entries
#[inline(always)]
pub fn get_context_entries(namespace: &str, context: &str) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_context_entries(namespace, context)
}

/// Gets all context-UUID entries from all context spaces for a given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-UUID entries in the namespace
#[inline(always)]
pub fn get_namespace_entries(
    namespace: &str,
) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_namespace_entries(namespace)
}


/// Gets all context-UUID entries in all namespaces and all context spaces.
/// 
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-UUID entries in all namespaces and all context spaces
#[inline(always)]
pub fn get_all_namespace_entries() -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_all_namespace_entries()
}

/// Gets all namespaces currently registered in the pool.
///
/// #### Returns
/// * `Vec<String>`: all namespace names
#[inline(always)]
pub fn list_namespaces() -> Vec<NamespaceString> {
    crate::registry::list_namespaces()
}

/// Gets all contexts currently registered within the given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Vec<String>`: all context names
#[inline(always)]
pub fn list_contexts(namespace: &str) -> Vec<ContextString> {
    crate::registry::list_contexts(namespace)
}

/// Gets all UUIDs currently registered within the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Vec<Uuid>`: all UUIDs
#[inline(always)]
pub fn list_ids(namespace: &str, context: &str) -> Vec<Uuid> {
    crate::registry::list_ids(namespace, context)
}

/// Clears  all contexts within a namespace and all associated UUIDs from memory.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[inline(always)]
pub fn clear_namespace(namespace: &str) {
    crate::registry::clear_namespace(namespace)
}

/// Clears all namespaces and all associated UUIDs from memory.
#[inline(always)]
pub fn clear_all_namespaces() {
    crate::registry::clear_all_namespaces()
}

/// Clears the given context and all associated UUIDs from the given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
#[inline(always)]
pub fn clear_context(namespace: &str, context: &str) {
    crate::registry::clear_context(namespace, context)
}

/// Clears all namespaces, contexts, and associated UUIDs from memory.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[inline(always)]
pub fn clear_all_contexts(namespace: &str) {
    crate::registry::clear_all_contexts(namespace)
}

/// Clears and returns all contexts within a namespace and all associated UUIDs from memory.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples
#[inline(always)]
pub fn drain_namespace(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    crate::registry::drain_namespace(namespace)
}

/// Clears and returns all namespaces and all associated UUIDs from memory.
///
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples
#[inline(always)]
pub fn drain_all_namespaces() -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    crate::registry::drain_all_namespaces()
}

/// Clears and returns the given context and all associated UUIDs from the given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-associated UUID pairs
#[inline(always)]
pub fn drain_context(
    namespace: &str,
    context: &str,
) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::drain_context(namespace, context)
}

/// Clears and returns all contexts within the given namespace and all associated UUIDs from memory.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples
#[inline(always)]
pub fn drain_all_contexts(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    crate::registry::drain_all_contexts(namespace)
}
