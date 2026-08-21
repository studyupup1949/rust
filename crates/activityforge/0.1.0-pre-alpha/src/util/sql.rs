use crate::db::Uuid;
use crate::{Error, Result};

/// Performs checks on the UUID.
///
/// Returns [Error] on check failure.
pub fn check_uuid(key: &str, uuid: &Uuid) -> Result<()> {
    if uuid.is_nil() {
        Err(Error::sql(format!("{key}: nil UUID")))
    } else {
        Ok(())
    }
}
