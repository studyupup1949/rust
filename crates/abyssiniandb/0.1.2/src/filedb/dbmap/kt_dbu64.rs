use super::super::super::{DbMapKeyType, HashValue};
use super::FileDbMap;
use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;

/// DbU64 Map in a file databse.
pub type FileDbMapDbU64 = FileDbMap<DbU64>;

/// db-key type. `u64` can be used as key.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DbU64(Vec<u8>);

impl DbMapKeyType for DbU64 {
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        DbU64(bytes.to_vec())
    }
    #[inline]
    fn signature() -> [u8; 8] {
        *b"u64_le\0\0"
    }
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
    #[inline]
    fn cmp_u8(&self, other: &[u8]) -> std::cmp::Ordering {
        self.0.as_slice().cmp(other)
    }
}
impl HashValue for DbU64 {}

impl Display for DbU64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ss = String::from_utf8_lossy(&self.0).to_string();
        write!(f, "'{ss}'")
    }
}

impl Deref for DbU64 {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

/*
impl Borrow<[u8]> for DbU64 {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
*/

impl From<&[u8]> for DbU64 {
    #[inline]
    fn from(a: &[u8]) -> Self {
        DbU64(a.to_vec())
    }
}

impl From<Vec<u8>> for DbU64 {
    #[inline]
    fn from(a: Vec<u8>) -> Self {
        DbU64(a)
    }
}

impl From<&str> for DbU64 {
    #[inline]
    fn from(a: &str) -> Self {
        DbU64(a.as_bytes().to_vec())
    }
}

impl From<String> for DbU64 {
    #[inline]
    fn from(a: String) -> Self {
        DbU64(a.into_bytes())
    }
}

impl From<&String> for DbU64 {
    #[inline]
    fn from(a: &String) -> Self {
        DbU64(a.as_bytes().to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for DbU64 {
    #[inline]
    fn from(a: &[u8; N]) -> Self {
        DbU64(a.to_vec())
    }
}

impl From<u64> for DbU64 {
    #[inline]
    fn from(a: u64) -> Self {
        DbU64(a.to_le_bytes().to_vec())
    }
}

impl From<&u64> for DbU64 {
    #[inline]
    fn from(a: &u64) -> Self {
        DbU64(a.to_le_bytes().to_vec())
    }
}

/*
impl From<DbU64> for DbU64 {
    #[inline]
    fn from(a: DbU64) -> Self {
        DbU64(a.0)
    }
}
*/

impl From<&DbU64> for DbU64 {
    #[inline]
    fn from(a: &DbU64) -> Self {
        DbU64(a.0.clone())
    }
}

impl From<DbU64> for u64 {
    #[inline]
    fn from(db_int: DbU64) -> u64 {
        u64::from(&db_int)
    }
}

impl From<&DbU64> for u64 {
    #[inline]
    fn from(db_int: &DbU64) -> u64 {
        let mut a = [0u8; 8];
        let len = db_int.0.len();
        if len < 8 {
            a[..len].copy_from_slice(db_int.0.as_slice());
        } else {
            a.copy_from_slice(&db_int.0[..8]);
        }
        u64::from_le_bytes(a)
    }
}
