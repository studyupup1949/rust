use super::super::super::{DbMapKeyType, HashValue};
use super::FileDbMap;
use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;

/// DbI64 Map in a file databse.
pub type FileDbMapDbI64 = FileDbMap<DbI64>;

/// db-key type. `u64` can be used as key.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DbI64(Vec<u8>);

impl DbMapKeyType for DbI64 {
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        DbI64(bytes.to_vec())
    }
    #[inline]
    fn signature() -> [u8; 8] {
        *b"i64_le\0\0"
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
impl HashValue for DbI64 {}

impl Display for DbI64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ss = String::from_utf8_lossy(&self.0).to_string();
        write!(f, "'{ss}'")
    }
}

impl Deref for DbI64 {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

/*
impl Borrow<[u8]> for DbI64 {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
*/

impl From<&[u8]> for DbI64 {
    #[inline]
    fn from(a: &[u8]) -> Self {
        DbI64(a.to_vec())
    }
}

impl From<Vec<u8>> for DbI64 {
    #[inline]
    fn from(a: Vec<u8>) -> Self {
        DbI64(a)
    }
}

impl From<&str> for DbI64 {
    #[inline]
    fn from(a: &str) -> Self {
        DbI64(a.as_bytes().to_vec())
    }
}

impl From<String> for DbI64 {
    #[inline]
    fn from(a: String) -> Self {
        DbI64(a.into_bytes())
    }
}

impl From<&String> for DbI64 {
    #[inline]
    fn from(a: &String) -> Self {
        DbI64(a.as_bytes().to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for DbI64 {
    #[inline]
    fn from(a: &[u8; N]) -> Self {
        DbI64(a.to_vec())
    }
}

impl From<i64> for DbI64 {
    #[inline]
    fn from(a: i64) -> Self {
        DbI64(a.to_le_bytes().to_vec())
    }
}

impl From<&i64> for DbI64 {
    #[inline]
    fn from(a: &i64) -> Self {
        DbI64(a.to_le_bytes().to_vec())
    }
}

/*
impl From<DbI64> for DbI64 {
    #[inline]
    fn from(a: DbI64) -> Self {
        DbI64(a.0)
    }
}
*/

impl From<&DbI64> for DbI64 {
    #[inline]
    fn from(a: &DbI64) -> Self {
        DbI64(a.0.clone())
    }
}

impl From<DbI64> for i64 {
    #[inline]
    fn from(db_int: DbI64) -> i64 {
        i64::from(&db_int)
    }
}

impl From<&DbI64> for i64 {
    #[inline]
    fn from(db_int: &DbI64) -> i64 {
        let mut a = [0u8; 8];
        let len = db_int.0.len();
        if len < 8 {
            a[..len].copy_from_slice(db_int.0.as_slice());
        } else {
            a.copy_from_slice(&db_int.0[..8]);
        }
        i64::from_le_bytes(a)
    }
}
