use super::UuidPoolError;

use uuid::Uuid;

pub const DEFAULT_UUID_BASE: u32 = 64;
pub const DEFAULT_MAX_RETRIES: usize = 64;

/// Reserves a new UUID in the given context space.
/// 
/// #### Argument
/// * `context`: named context space
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve(context: &str) -> Result<Uuid, UuidPoolError> {
    reserve_with(context, DEFAULT_UUID_BASE, DEFAULT_MAX_RETRIES)
}

/// Reserves a new UUID in the given context space with a custom base.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `base`: basis for UUID generation
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve_with_base(context: &str, base: u32) -> Result<Uuid, UuidPoolError> {
    reserve_with(context, base, DEFAULT_MAX_RETRIES)
}

/// Reserves a new UUID in the given context space with a custom base and retry count.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `base`: basis for UUID generation
/// * `max_retries`: maximum number of retries
/// #### Returns
/// * `Result<Uuid, UuidPoolError>`: the new UUID
#[inline(always)]
pub fn reserve_with(context: &str, base: u32, max_retries: usize) -> Result<Uuid, UuidPoolError> {
    crate::registry::random_uuid(context, base, max_retries, 0)
}

/// Adds an existing UUID to the given context space.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn add(context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::add_uuid_to_pool(context, &uuid)
}

/// Removes an existing UUID from the given context space.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn remove(context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::remove_uuid_from_pool(context, &uuid)
}

/// Tries to remove an existing UUID from the given context space.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `uuid`: existing UUID
/// #### Returns
/// * `bool`: true if the UUID was removed, false otherwise
#[inline(always)]
pub fn try_remove(context: &str, uuid: Uuid) -> bool {
    crate::registry::remove_uuid_from_pool(context, &uuid).is_ok()
}

/// Replaces an existing UUID with a new UUID in the given context space.
/// 
/// #### Arguments
/// * `context`: named context space
/// * `old_uuid`: existing UUID
/// * `new_uuid`: new UUID
/// #### Returns
/// * `Result<(), UuidPoolError>`: success or error result
#[inline(always)]
pub fn replace(context: &str, old_uuid: Uuid, new_uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::replace_uuid_in_pool(context, &old_uuid, &new_uuid)
}

/// Gets all UUIDs for the given context space.
/// 
/// #### Arguments
/// * `context`: named context space
/// #### Returns
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-associated UUID pairs
#[inline(always)]
pub fn get(context: &str) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::get_context_uuids_from_pool(context)
}

/// Gets all UUIDs for all context spaces.
/// 
/// #### Returns
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-UUID pairs
#[inline(always)]
pub fn get_all() -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::get_all_contexts_uuids_from_pool()
}

/// Clear the given context and all associated UUIDs from memory.
/// 
/// #### Arguments
/// * `context`: named context space
#[inline(always)]
pub fn clear_context(context: &str) {
    crate::registry::clear_context(context)
}

/// Clears all contexts and all associated UUIDs from memory.
#[inline(always)]
pub fn clear_all_contexts() {
    crate::registry::clear_all_contexts()
}

/// Clears and returns the given context and all associated UUIDs from memory.
/// 
/// #### Arguments
/// * `context`: named context space
/// #### Returns
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-associated UUID pairs
#[inline(always)]
pub fn drain_context(context: &str) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::drain_context(context)
}

/// Clears and returns all contexts and all associated UUIDs from memory.
/// 
/// #### Returns
/// * `Result<Vec<(String, Uuid)>, UuidPoolError>`: all context-associated UUID pairs
#[inline(always)]
pub fn drain_all_contexts() -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::drain_all_contexts()
}
