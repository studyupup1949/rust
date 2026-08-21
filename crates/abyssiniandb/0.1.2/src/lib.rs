/*!
The simple local key-value store.

# Features

- key-value store.
- hash buckets algorithm.
- minimum support rustc 1.58.1 (db9d1b20b 2022-01-20)

# Compatibility

- Nothing?

# Todo

- [ ] more performance
- [ ] DB lock as support for multi-process-safe

# Low priority todo

- [ ] transaction support that handles multiple key-space at a time.
- [ ] thread-safe support
- [ ] non db lock multi-process-safe support

# Examples
*/
use std::fmt::Debug;
use std::hash::Hash;
use std::io::Result;
use std::path::Path;

pub mod filedb;

pub use filedb::{DbBytes, DbI64, DbString, DbU64, DbVu64};
pub use filedb::{DbXxxIter, DbXxxIterMut, DbXxxKeys, DbXxxValues};

/// Open the file db. This data is stored in file.
pub fn open_file<P: AsRef<Path>>(path: P) -> Result<filedb::FileDb> {
    filedb::FileDb::open(path)
}

/// base interface for generic key-value map store interface. this is not include `KT`
pub trait DbXxxBase {
    /// returns the number of elements in the map.
    fn len(&self) -> Result<u64>;

    /// returns the number of elements in the map.
    #[inline]
    fn is_empty(&self) -> Result<bool> {
        self.len().map(|a| a == 0)
    }

    /// read and fill buffer.
    fn read_fill_buffer(&mut self) -> Result<()>;

    /// flush file buffer, the dirty intermediate buffered content is written.
    fn flush(&mut self) -> Result<()>;

    /// synchronize all OS-internal metadata to storage.
    fn sync_all(&mut self) -> Result<()>;

    /// synchronize data to storage, except file metadabe.
    fn sync_data(&mut self) -> Result<()>;
}

/// generic key-value map store interface. the key type is `KT`. this is only object safe.
pub trait DbXxxObjectSafe<KT: DbMapKeyType>: DbXxxBase {
    /// returns the value corresponding to the key. this key is store raw data and type `&[u8]`.
    fn get_kt(&mut self, key: &KT) -> Result<Option<Vec<u8>>>;

    /// inserts a key-value pair into the db. this key is store raw data and type `&[u8]`.
    fn put_kt(&mut self, key: &KT, value: &[u8]) -> Result<()>;

    /// removes a key from the db. this key is store raw data and type `&[u8]`.
    fn del_kt(&mut self, key: &KT) -> Result<Option<Vec<u8>>>;

    /// returns true if the map contains a value for the specified key. this key is store raw data.
    fn includes_key_kt(&mut self, key: &KT) -> Result<bool>;
}

/// generic key-value map store interface. the key type is `KT`.
pub trait DbXxx<KT: DbMapKeyType>: DbXxxObjectSafe<KT> {
    /// returns the value corresponding to the key.
    #[inline]
    fn get<'a, Q>(&mut self, key: &'a Q) -> Result<Option<Vec<u8>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let key_kt: KT = From::from(key);
        self.get_kt(&key_kt)
    }

    /// returns the value corresponding to the key. the value is converted to `String`.
    #[inline]
    fn get_string<'a, Q>(&mut self, key: &'a Q) -> Result<Option<String>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        self.get(key)
            .map(|opt| opt.map(|val| String::from_utf8_lossy(&val).to_string()))
    }

    /// gets bulk key-value paires from the db.
    fn bulk_get<'a, Q>(&mut self, bulk_keys: &[&'a Q]) -> Result<Vec<Option<Vec<u8>>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let mut result: Vec<(usize, Option<Vec<u8>>)> = Vec::new();
        let mut vec: Vec<(usize, &Q)> =
            bulk_keys.iter().enumerate().map(|(i, &a)| (i, a)).collect();
        vec.sort_unstable_by(|a, b| b.1.cmp(a.1));
        while let Some(ik) = vec.pop() {
            let result_value = self.get(ik.1)?;
            result.push((ik.0, result_value));
        }
        result.sort_by(|a, b| a.0.cmp(&(b.0)));
        let ret: Vec<Option<Vec<u8>>> = result.iter().map(|a| a.1.clone()).collect();
        Ok(ret)
    }

    /// gets bulk key-value paires from the db.
    #[inline]
    fn bulk_get_string<'a, Q>(&mut self, bulk_keys: &[&'a Q]) -> Result<Vec<Option<String>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let vec = self.bulk_get(bulk_keys)?;
        let mut ret = Vec::new();
        for opt in vec {
            let b = opt.map(|val| String::from_utf8_lossy(&val).to_string());
            ret.push(b);
        }
        Ok(ret)
    }

    /// inserts a key-value pair into the db.
    #[inline]
    fn put<'a, Q>(&mut self, key: &'a Q, value: &[u8]) -> Result<()>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let key_kt: KT = From::from(key);
        self.put_kt(&key_kt, value)
    }

    /// inserts a key-value pair into the db-map. the value is `&str` and it is converted to `&[u8]`
    #[inline]
    fn put_string<'a, Q>(&mut self, key: &'a Q, value: &str) -> Result<()>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        self.put(key, value.as_bytes())
    }

    /// inserts bulk key-value pairs into the db.
    fn bulk_put<'a, Q>(&mut self, bulk: &[(&'a Q, &[u8])]) -> Result<()>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let mut vec = bulk.to_vec();
        vec.sort_by(|a, b| b.0.cmp(a.0));
        while let Some(kv) = vec.pop() {
            self.put(kv.0, kv.1)?;
        }
        Ok(())
    }

    /// inserts bulk key-value pairs into the db.
    #[inline]
    fn bulk_put_string<'a, Q>(&mut self, bulk: &[(&'a Q, String)]) -> Result<()>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let mut vec = bulk.to_vec();
        vec.sort_unstable_by(|a, b| b.0.cmp(a.0));
        while let Some(kv) = vec.pop() {
            self.put(kv.0, kv.1.as_bytes())?;
        }
        Ok(())
    }

    /// removes a key from the db.
    #[inline]
    fn delete<'a, Q>(&mut self, key: &'a Q) -> Result<Option<Vec<u8>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let key_kt: KT = From::from(key);
        self.del_kt(&key_kt)
    }

    /// removes a key from the db.
    #[inline]
    fn delete_string<'a, Q>(&mut self, key: &'a Q) -> Result<Option<String>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        self.delete(key)
            .map(|opt| opt.map(|val| String::from_utf8_lossy(&val).to_string()))
    }

    /// delete bulk key-value paires from the db.
    fn bulk_delete<'a, Q>(&mut self, bulk_keys: &[&'a Q]) -> Result<Vec<Option<Vec<u8>>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let mut result: Vec<(usize, Option<Vec<u8>>)> = Vec::new();
        let mut vec: Vec<(usize, &Q)> =
            bulk_keys.iter().enumerate().map(|(i, &a)| (i, a)).collect();
        vec.sort_unstable_by(|a, b| b.1.cmp(a.1));
        while let Some(ik) = vec.pop() {
            let result_value = self.delete(ik.1)?;
            result.push((ik.0, result_value));
        }
        result.sort_by(|a, b| a.0.cmp(&(b.0)));
        let ret: Vec<Option<Vec<u8>>> = result.iter().map(|a| a.1.clone()).collect();
        Ok(ret)
    }

    /// delete bulk key-value paires from the db.
    #[inline]
    fn bulk_delete_string<'a, Q>(&mut self, bulk_keys: &[&'a Q]) -> Result<Vec<Option<String>>>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let vec = self.bulk_delete(bulk_keys)?;
        let mut ret = Vec::new();
        for opt in vec {
            let b = opt.map(|val| String::from_utf8_lossy(&val).to_string());
            ret.push(b);
        }
        Ok(ret)
    }

    /// returns true if the map contains a value for the specified key.
    #[inline]
    fn includes_key<'a, Q>(&mut self, key: &'a Q) -> Result<bool>
    where
        KT: From<&'a Q>,
        Q: Ord + ?Sized,
    {
        let key_kt: KT = From::from(key);
        self.includes_key_kt(&key_kt)
    }

    /// extends a collection with the contents of an iterator
    #[inline]
    fn put_from_iter<T>(&mut self, iter: T) -> Result<()>
    where
        T: Iterator<Item = (KT, Vec<u8>)>,
    {
        for (key, value) in iter {
            self.put_kt(&key, &value)?;
        }
        Ok(())
    }
}

/// key-value db map store interface.
pub trait DbMap<KT: DbMapKeyType>: DbXxx<KT> {
    /// An iterator visiting all key-value pairs in arbitrary order.
    /// The iterator element type is (&'a K, &'a V).
    fn iter(&self) -> DbXxxIter<KT>;

    /// An iterator visiting all key-value pairs in arbitrary order,
    /// with mutable references to the values.
    /// The iterator element type is (&'a K, &'a mut V).
    fn iter_mut(&mut self) -> DbXxxIterMut<KT>;

    // Clears the map, removing all key-value pairs. Keeps the allocated memory for reuse.
    //fn clear(&mut self) -> Result<()>;

    /// An iterator visiting all keys in arbitrary order. The iterator element type is KT.
    fn keys(&self) -> DbXxxKeys<KT>;

    /// An iterator visiting all values in arbitrary order. The iterator element type is Vec<u8>.
    fn values(&self) -> DbXxxValues<KT>;
}

/// key-value map store interface. the key type is `String`.
pub trait DbMapDbString: DbXxx<DbString> {}

/// key-value map store interface. the key type is `Vec<u8>`.
pub trait DbMapDbBytes: DbXxx<DbBytes> {}

/// key-value map store interface. the key type is `i64`.
pub trait DbMapDbI64: DbXxx<DbI64> {}

/// key-value map store interface. the key type is `u64`.
pub trait DbMapDbU64: DbXxx<DbU64> {}

/// key-value map store interface. the key type is `vu64`.
pub trait DbMapDbVu64: DbXxx<DbVu64> {}

/// key type
pub trait DbMapKeyType: 'static + Ord + Clone + Default + HashValue + Debug {
    /// Convert a byte slice to Key.
    fn from_bytes(bytes: &[u8]) -> Self;
    /// Signature in header of database file.
    fn signature() -> [u8; 8];
    /// Byte slice of data to be saved.
    fn as_bytes(&self) -> &[u8];
    /// Compare with stored data
    fn cmp_u8(&self, other: &[u8]) -> std::cmp::Ordering;
    /// Short byte slice of data to be saved node.
    #[cfg(feature = "tr_has_short_key")]
    fn as_short_bytes(&self) -> Option<&[u8]> {
        let b_sl = self.as_bytes();
        if b_sl.len() <= 32 {
            Some(b_sl)
        } else {
            None
        }
    }
}

/// hash value for htx
pub trait HashValue: Hash {
    /// hash value for htx
    fn hash_value(&self) -> u64 {
        use std::hash::Hasher;
        #[cfg(feature = "std_default_hasher")]
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        #[cfg(not(feature = "std_default_hasher"))]
        let mut hasher = MyHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Default)]
struct MyHasher(u64);

impl std::hash::Hasher for MyHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for chunk8 in bytes.chunks(8) {
            let len = chunk8.len();
            if len == 8 {
                let mut ary = [0u8; 8];
                ary.copy_from_slice(chunk8);
                //let a = u64::from_le_bytes(ary);
                let a = u64::from_be_bytes(ary);
                self.0 = _xorshift64s(self.0.wrapping_add(a));
            } else {
                let mut a = 0;
                for b in chunk8 {
                    a = (a << 8) | *b as u64;
                }
                self.0 = _xorshift64s(self.0.wrapping_add(a));
            }
        }
    }
}

// ref.) https://en.wikipedia.org/wiki/Xorshift
#[inline]
fn _xorshift64s(a: u64) -> u64 {
    //let mut x = a.rotate_right(12);
    let mut x = a;
    #[cfg(not(any(feature = "myhasher_george1", feature = "myhasher_george2")))]
    {
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
    }
    #[cfg(feature = "myhasher_george1")]
    {
        x ^= x >> 13;
        x ^= x << 7;
        x ^= x >> 17;
    }
    #[cfg(feature = "myhasher_george2")]
    {
        x ^= x << 7;
        x ^= x >> 9;
    }
    x
}
