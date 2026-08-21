use crate::{Error, Result};

mod serial;
mod sql;

pub use serial::*;
pub use sql::*;

/// Deduplicates the provided list, returning an error if any duplicates found.
pub fn dedup_list<T: Ord + PartialEq, I: Into<Vec<T>>>(key: &str, list: I) -> Result<Vec<T>> {
    let mut list = list.into();
    let len = list.len();

    list.sort();
    list.dedup();

    if len == list.len() {
        Ok(list)
    } else {
        Err(Error::io(format!("{key}: contained duplicates")))
    }
}

/// Creates a random UUID.
pub fn rand_uuid() -> sqlx::types::uuid::Uuid {
    let mut uuid = [0u8; 16];
    rand::fill(&mut uuid);
    sqlx::types::uuid::Builder::from_random_bytes(uuid).into_uuid()
}
