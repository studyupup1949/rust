use super::super::super::{DbMapKeyType, HashValue};
use super::FileDbMap;
use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;

/// DbVu64 Map in a file databse.
pub type FileDbMapDbVu64 = FileDbMap<DbVu64>;

/// db-key type. `vu64` can be used as key.
#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DbVu64(Vec<u8>);

impl DbMapKeyType for DbVu64 {
    #[inline]
    fn from_bytes(bytes: &[u8]) -> Self {
        DbVu64(bytes.to_vec())
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
        //self.0.as_slice().cmp(other)
        let my: u64 = vu64::decode(self.0.as_slice()).unwrap();
        let other: u64 = vu64::decode(other).unwrap();
        my.cmp(&other)
    }
}
impl HashValue for DbVu64 {}

impl Display for DbVu64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let ss = String::from_utf8_lossy(&self.0).to_string();
        write!(f, "'{ss}'")
    }
}

impl Deref for DbVu64 {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

/*
impl Borrow<[u8]> for DbVu64 {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}
*/

impl From<&[u8]> for DbVu64 {
    #[inline]
    fn from(a: &[u8]) -> Self {
        DbVu64(a.to_vec())
    }
}

impl From<Vec<u8>> for DbVu64 {
    #[inline]
    fn from(a: Vec<u8>) -> Self {
        DbVu64(a)
    }
}
/*
impl From<&str> for DbVu64 {
    #[inline]
    fn from(a: &str) -> Self {
        DbVu64(a.as_bytes().to_vec())
    }
}

impl From<String> for DbVu64 {
    #[inline]
    fn from(a: String) -> Self {
        DbVu64(a.into_bytes())
    }
}

impl From<&String> for DbVu64 {
    #[inline]
    fn from(a: &String) -> Self {
        DbVu64(a.as_bytes().to_vec())
    }
}
*/
impl<const N: usize> From<&[u8; N]> for DbVu64 {
    #[inline]
    fn from(a: &[u8; N]) -> Self {
        DbVu64(a.to_vec())
    }
}

impl From<u64> for DbVu64 {
    #[inline]
    fn from(a: u64) -> Self {
        let a = vu64::encode(a);
        DbVu64(a.as_ref().to_vec())
    }
}

impl From<&u64> for DbVu64 {
    #[inline]
    fn from(a: &u64) -> Self {
        let a = vu64::encode(*a);
        DbVu64(a.as_ref().to_vec())
    }
}

/*
impl From<DbVu64> for DbVu64 {
    #[inline]
    fn from(a: DbVu64) -> Self {
        DbVu64(a.0)
    }
}
*/

impl From<&DbVu64> for DbVu64 {
    #[inline]
    fn from(a: &DbVu64) -> Self {
        DbVu64(a.0.clone())
    }
}

impl From<DbVu64> for u64 {
    #[inline]
    fn from(db_int: DbVu64) -> u64 {
        u64::from(&db_int)
    }
}

impl From<&DbVu64> for u64 {
    #[inline]
    fn from(db_int: &DbVu64) -> u64 {
        vu64::decode(&db_int.0).unwrap()
    }
}
