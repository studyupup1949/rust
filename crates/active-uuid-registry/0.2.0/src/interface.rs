use super::UuidPoolError;

use uuid::Uuid;

pub const DEFAULT_UUID_BASE: u32 = 64;
pub const DEFAULT_MAX_RETRIES: usize = 64;

#[inline(always)]
pub fn reserve(context: &str) -> Result<Uuid, UuidPoolError> {
    reserve_with(context, DEFAULT_UUID_BASE, DEFAULT_MAX_RETRIES)
}

#[inline(always)]
pub fn reserve_with_base(context: &str, base: u32) -> Result<Uuid, UuidPoolError> {
    reserve_with(context, base, DEFAULT_MAX_RETRIES)
}

#[inline(always)]
pub fn reserve_with(context: &str, base: u32, max_retries: usize) -> Result<Uuid, UuidPoolError> {
    crate::registry::random_uuid(context, base, max_retries, 0)
}

#[inline(always)]
pub fn add(context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::add_uuid_to_pool(context, &uuid)
}

#[inline(always)]
pub fn remove(context: &str, uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::remove_uuid_from_pool(context, &uuid)
}

#[inline(always)]
pub fn try_remove(context: &str, uuid: Uuid) -> bool {
    crate::registry::remove_uuid_from_pool(context, &uuid).is_ok()
}

#[inline(always)]
pub fn replace(context: &str, old_uuid: Uuid, new_uuid: Uuid) -> Result<(), UuidPoolError> {
    crate::registry::replace_uuid_in_pool(context, &old_uuid, &new_uuid)
}

#[inline(always)]
pub fn get(context: &str) -> Result<Vec<(String, Uuid)>, UuidPoolError> {
    crate::registry::get_context_uuids_from_pool(context)
}

#[inline(always)]
pub fn clear_context(context: &str) -> Result<(), UuidPoolError> {
    crate::registry::drain_context(context)
}

#[inline(always)]
pub fn clear_all() -> Result<(), UuidPoolError> {
    crate::registry::drain_all_contexts()
}
