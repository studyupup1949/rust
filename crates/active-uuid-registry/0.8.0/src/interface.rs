//! # Active UUID Registry Interface
//! 
//! This module provides the user-facing API for the Active UUID Registry.
//! 
//! Be aware that in **concurrent-map** mode, any operations that interact with the registry will work only on data retrieved from the registry for a current snapshot.
//! This means that data may be written to the registry while you are working with it, and you may not see the changes you made until you retrieve/alter the data again via another function call.
//! 
//! This race condition is *not* present in the **default / single-threaded** mode due to the mutex lock protection. Use **concurrent-map** with caution.
//!
//! ## Owned namespaces
//!
//! A namespace can optionally be *claimed* via [`reserve_owned_namespace`], which returns a
//! cloneable [`crate::OwnedNamespace`] capability handle. Once claimed, the free functions in
//! this module (`add_id`, `remove_id`, `clear_namespace`, etc.) can no longer write to that
//! namespace. Only the [`crate::OwnedNamespace`] handle (and its clones) can write to it. 
//! Namespaces that are never claimed are globally accessible: open to any caller.
//!
//! [`clear_all_namespaces`] and [`drain_all_namespaces`] only ever affect unowned namespaces;
//! owned namespaces are always preserved by these bulk operations.

use super::{UuidPoolError, NamespaceString, ContextString};
use crate::registry::WriteAccess;
use crate::OwnedNamespace;
use uuid::Uuid;

/// The default base for UUID generation.
#[doc(alias = "constant")]
pub const DEFAULT_UUID_BASE: u32 = 64;

/// The default maximum number of retries for UUID generation.
#[doc(alias = "constant")]
pub const DEFAULT_MAX_RETRIES: usize = 64;

/// Adds a new namespace to the registry.
///
/// No-ops if the namespace is already owned by an [`OwnedNamespace`] handle. Use
/// [`try_reserve_namespace`] to observe that failure.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn reserve_namespace(namespace: &str) {
    let _ = crate::registry::add_namespace(namespace, WriteAccess::Unowned);
}

/// Adds a new namespace to the registry, reporting whether it was authorized.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if the namespace is owned by an [`OwnedNamespace`] handle
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn try_reserve_namespace(namespace: &str) -> Result<(), UuidPoolError> {
    crate::registry::add_namespace(namespace, WriteAccess::Unowned)
}

/// Claims a namespace as owned, returning a cloneable capability handle that is the only
/// way to write to it going forward.
///
/// Fails if the namespace is already owned, or if it already contains data (only an absent
/// or empty namespace may be claimed).
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<OwnedNamespace, UuidPoolError>`: the ownership capability handle
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn reserve_owned_namespace(namespace: &str) -> Result<OwnedNamespace, UuidPoolError> {
    let signature = crate::registry::add_owned_namespace(namespace)?;
    Ok(OwnedNamespace::new(namespace.to_string(), signature))
}

/// Removes a namespace and all associated contexts from the registry.
///
/// No-ops if the namespace is owned by an [`OwnedNamespace`] handle. Use
/// [`try_remove_namespace`] to observe that failure, or [`OwnedNamespace::remove`] to remove
/// an owned namespace.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[doc(alias = "setter")]
#[doc(alias = "remover")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn remove_namespace(namespace: &str) {
    let _ = crate::registry::remove_namespace(namespace, WriteAccess::Unowned);
}

/// Removes a namespace and all associated contexts from the registry, reporting whether it was authorized.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if the namespace is owned by an [`OwnedNamespace`] handle
#[doc(alias = "setter")]
#[doc(alias = "remover")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn try_remove_namespace(namespace: &str) -> Result<(), UuidPoolError> {
    crate::registry::remove_namespace(namespace, WriteAccess::Unowned)
}

/// Replaces a namespace with a new namespace in the registry.
///
/// No-ops if `old_namespace` is owned by an [`OwnedNamespace`] handle. Use
/// [`try_replace_namespace`] to observe that failure, or [`OwnedNamespace::rename`] to rename
/// an owned namespace.
/// 
/// #### Arguments
/// * `old_namespace`: old namespace
/// * `new_namespace`: new namespace
#[doc(alias = "setter")]
#[doc(alias = "replacer")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn replace_namespace(old_namespace: &str, new_namespace: &str) {
    let _ = crate::registry::replace_namespace(old_namespace, new_namespace, WriteAccess::Unowned);
}

/// Replaces a namespace with a new namespace in the registry, reporting whether it was authorized.
///
/// #### Arguments
/// * `old_namespace`: old namespace
/// * `new_namespace`: new namespace
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if `old_namespace` is owned by an [`OwnedNamespace`] handle, or `new_namespace` is owned/non-empty
#[doc(alias = "setter")]
#[doc(alias = "replacer")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn try_replace_namespace(old_namespace: &str, new_namespace: &str) -> Result<(), UuidPoolError> {
    crate::registry::replace_namespace(old_namespace, new_namespace, WriteAccess::Unowned)
}

/// Reserves a new UUID in the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "id")]
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
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "id")]
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
#[doc(alias = "setter")]
#[doc(alias = "reservation")]
#[doc(alias = "id")]
pub fn reserve_id_with(
    namespace: &str,
    context: &str,
    base: u32,
    max_retries: usize,
) -> Result<Uuid, UuidPoolError> {
    crate::registry::random_uuid(namespace, context, base, max_retries, 0, WriteAccess::Unowned)
}

/// Adds an existing UUID to the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[doc(alias = "setter")]
#[doc(alias = "adder")]
#[doc(alias = "id")]
#[inline(always)]
pub fn add_id(namespace: &str, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::add_uuid_to_pool(namespace, context, &uuid, WriteAccess::Unowned)
}

/// Removes an existing UUID from the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[doc(alias = "setter")]
#[doc(alias = "remover")]
#[doc(alias = "id")]
#[inline(always)]
pub fn remove_id(namespace: &str, context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::remove_uuid_from_pool(namespace, context, &uuid, WriteAccess::Unowned)
}

/// Tries to remove an existing UUID from the given namespace and context space.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `bool`: true if the UUID was removed, false otherwise
#[doc(alias = "setter")]
#[doc(alias = "remover")]
#[doc(alias = "id")]
#[inline(always)]
pub fn try_remove_id(namespace: &str, context: &str, uuid: Uuid) -> bool {
    crate::registry::remove_uuid_from_pool(namespace, context, &uuid, WriteAccess::Unowned).is_ok()
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
#[doc(alias = "setter")]
#[doc(alias = "replacer")]
#[doc(alias = "id")]
#[inline(always)]
pub fn replace_id(
    namespace: &str,
    context: &str,
    old_uuid: Uuid,
    new_uuid: Uuid,
) -> Result<(), UuidPoolError> {
    crate::registry::replace_uuid_in_pool(namespace, context, &old_uuid, &new_uuid, WriteAccess::Unowned)
}

/// Creates a random UUID given a base value.
/// 
/// This does *not* add the UUID to the pool.
/// 
/// #### Arguments
/// * `base`: basis for UUID generation
/// #### Returns
/// * `Uuid`: the new UUID
#[doc(alias = "getter")]
#[doc(alias = "id")]
#[inline(always)]
pub fn get_random_uuid_with_base(base: u32) -> Uuid {
    crate::registry::make_uuid_with_base(base)
}

/// Gets all context-UUID entries from a specific context space in a given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-associated UUID entries currently in the pool.
#[inline(always)]
#[doc(alias = "getter")]
#[doc(alias = "context")]
pub fn get_context_entries(namespace: &str, context: &str) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_context_entries(namespace, context)
}

/// Gets all context-UUID entries from all context spaces for a given namespace.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-UUID entries in the namespace currently in the pool.
#[inline(always)]
#[doc(alias = "getter")]
#[doc(alias = "namespace")]
pub fn get_namespace_entries(
    namespace: &str,
) -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_namespace_entries(namespace)
}


/// Gets all context-UUID entries in all namespaces and all context spaces.
/// 
/// #### Returns
/// * `Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError>`: all context-UUID entries in all namespaces and all context spaces currently in the pool.
/// 
#[inline(always)]
#[doc(alias = "getter")]
#[doc(alias = "namespace")]
pub fn get_all_namespace_entries() -> Result<Vec<(NamespaceString, ContextString, Uuid)>, UuidPoolError> {
    crate::registry::get_all_namespace_entries()
}

/// Gets all namespaces currently registered in the pool.
///
/// #### Returns
/// * `Vec<String>`: all namespace names
#[doc(alias = "getter")]
#[doc(alias = "namespace")]
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
#[doc(alias = "getter")]
#[doc(alias = "context")]
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
#[doc(alias = "getter")]
#[doc(alias = "id")]
#[inline(always)]
pub fn list_ids(namespace: &str, context: &str) -> Vec<Uuid> {
    crate::registry::list_ids(namespace, context)
}

/// Clears  all contexts within a namespace and all associated UUIDs from memory.
///
/// No-ops if the namespace is owned by an [`OwnedNamespace`] handle. Use
/// [`try_clear_namespace`] to observe that failure.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[doc(alias = "clear")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn clear_namespace(namespace: &str) {
    let _ = crate::registry::clear_namespace(namespace, WriteAccess::Unowned);
}

/// Clears all contexts within a namespace and all associated UUIDs from memory, reporting whether it was authorized.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if the namespace is owned by an [`OwnedNamespace`] handle
#[doc(alias = "clear")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn try_clear_namespace(namespace: &str) -> Result<(), UuidPoolError> {
    crate::registry::clear_namespace(namespace, WriteAccess::Unowned)
}

/// Clears all *unowned* namespaces and all associated UUIDs from memory.
///
/// Namespaces owned by an [`OwnedNamespace`] handle are always preserved.
#[doc(alias = "clear")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn clear_all_namespaces() {
    crate::registry::clear_all_namespaces()
}

/// Clears the given context and all associated UUIDs from the given namespace.
///
/// No-ops if the namespace is owned by an [`OwnedNamespace`] handle. Use
/// [`try_clear_context`] to observe that failure.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
#[doc(alias = "clear")]
#[doc(alias = "context")]
#[inline(always)]
pub fn clear_context(namespace: &str, context: &str) {
    let _ = crate::registry::clear_context(namespace, context, WriteAccess::Unowned);
}

/// Clears the given context and all associated UUIDs from the given namespace, reporting whether it was authorized.
///
/// #### Arguments
/// * `namespace`: named namespace
/// * `context`: named context space
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if the namespace is owned by an [`OwnedNamespace`] handle
#[doc(alias = "clear")]
#[doc(alias = "context")]
#[inline(always)]
pub fn try_clear_context(namespace: &str, context: &str) -> Result<(), UuidPoolError> {
    crate::registry::clear_context(namespace, context, WriteAccess::Unowned)
}

/// Clears all namespaces, contexts, and associated UUIDs from memory.
///
/// No-ops if the namespace is owned by an [`OwnedNamespace`] handle. Use
/// [`try_clear_all_contexts`] to observe that failure.
/// 
/// #### Arguments
/// * `namespace`: named namespace
#[doc(alias = "clear")]
#[doc(alias = "context")]
#[inline(always)]
pub fn clear_all_contexts(namespace: &str) {
    let _ = crate::registry::clear_all_contexts(namespace, WriteAccess::Unowned);
}

/// Clears all contexts and associated UUIDs within a namespace, reporting whether it was authorized.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<(), UuidPoolError>`: success, or an error if the namespace is owned by an [`OwnedNamespace`] handle
#[doc(alias = "clear")]
#[doc(alias = "context")]
#[inline(always)]
pub fn try_clear_all_contexts(namespace: &str) -> Result<(), UuidPoolError> {
    crate::registry::clear_all_contexts(namespace, WriteAccess::Unowned)
}

/// Clears and returns all contexts within a namespace and all associated UUIDs from memory.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples currently in the pool.
#[doc(alias = "drain")]
#[doc(alias = "namespace")]
#[inline(always)]
pub fn drain_namespace(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    crate::registry::drain_namespace(namespace, WriteAccess::Unowned)
}

/// Clears and returns all *unowned* namespaces and all associated UUIDs from memory.
///
/// Namespaces owned by an [`OwnedNamespace`] handle are always preserved.
///
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples currently in the pool.
#[doc(alias = "drain")]
#[doc(alias = "namespace")]
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
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-associated UUID pairs currently in the pool.
#[doc(alias = "drain")]
#[doc(alias = "context")]
#[inline(always)]
pub fn drain_context(
    namespace: &str,
    context: &str,
) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::drain_context(namespace, context, WriteAccess::Unowned)
}

/// Clears and returns all contexts within the given namespace and all associated UUIDs from memory.
///
/// #### Arguments
/// * `namespace`: named namespace
/// #### Returns
/// * `Result<Vec<(String, String, Uuid)>, UuidPoolError>`: all (namespace, context, uuid) triples currently in the pool.
#[doc(alias = "drain")]
#[doc(alias = "context")]
#[inline(always)]
pub fn drain_all_contexts(
    namespace: &str,
) -> Result<Vec<(String, String, Uuid)>, UuidPoolError> {
    crate::registry::drain_all_contexts(namespace, WriteAccess::Unowned)
}
