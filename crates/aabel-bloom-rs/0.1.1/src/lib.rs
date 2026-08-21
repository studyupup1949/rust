//! A crate which exposes [`BloomFilter`], an implementation of the [bloom filter]() algorithm.
//!
//! # Example
//!
//!```
//! use aabel_bloom_rs::BloomFilter;
//! use aabel_multihash_rs::{BuildHasherExt, BuildPairHasher};
//! use std::hash::{BuildHasher, Hash};
//!
//! let keys1 = (0, 0);
//! let keys2 = (1, 1);
//! let builder = BuildPairHasher::new_with_keys(keys1, keys2);
//!
//! let mut filter = BloomFilter::<&str, _>::new(builder);
//!
//! // The filter is empty
//! let item = "Hello world!";
//! let contains = filter.contains(item);
//! assert!(!contains);
//!
//! // Insert several items in the filter.
//! filter.insert(item);
//! filter.insert("Tessting testing");
//! filter.insert("Rust rocks");
//! filter.insert("In Rust we trust");
//!
//! // Check if the item is in the filter
//! let contains = filter.contains(item);
//! assert!(contains)
//!```

use aabel_multihash_rs::{BuildHasherExt, Hash64, HasherExt};
use bitvec::array::BitArray;
use std::{
    borrow::Borrow,
    hash::{BuildHasher, Hash},
    marker::PhantomData,
};

/// Implements the [bloom filter](https://en.wikipedia.org/wiki/Bloom_filter).
/// [`B`] is an instance of [`BuildHasherExt`] trait which helps generating multiple hash values for any given item [`T`].
/// The [`K`] generic argument represents the number of usize cells in the inner array.
/// The [`H`] generic argument represents the number of hash values computed for each item.
///
/// # Example
pub struct BloomFilter<T, B, const K: usize = 100, const H: usize = 10>
where
    T: ?Sized,
{
    builder: B,
    bits: BitArray<[usize; K]>,
    _marker: PhantomData<T>,
}

impl<T, B, const K: usize, const H: usize> BloomFilter<T, B, K, H>
where
    T: ?Sized,
    B: BuildHasher + BuildHasherExt,
{
    /// Creates a new [`BloomFilter`] instance based on a given [`BuildHasherExt`] instance and a give number of hash values for each item.
    pub fn new(builder: B) -> Self {
        Self {
            builder,
            bits: BitArray::ZERO,
            _marker: PhantomData,
        }
    }
}

impl<T, B, const K: usize, const H: usize> BloomFilter<T, B, K, H>
where
    B: BuildHasher + BuildHasherExt,
    <B as BuildHasher>::Hasher: HasherExt,
    T: Hash + ?Sized,
{
    /// Inserts in the filter a new item.
    ///
    /// # Example
    ///```
    /// use aabel_bloom_rs::*;
    /// use aabel_multihash_rs::*;
    ///
    /// // Create the hasher builder
    /// let keys1 = (0, 0);
    /// let keys2 = (1, 1);
    /// let builder = BuildPairHasher::new_with_keys(keys1, keys2);
    ///
    /// // Create the bloom filter
    /// let mut filter = BloomFilter::<[u8], _>::new(builder);
    ///
    /// // Insert in the filter
    /// filter.insert(&[1u8, 2, 3]);
    ///```
    pub fn insert<U>(&mut self, item: &U)
    where
        T: Borrow<U>,
        U: Hash + ?Sized,
    {
        let set_bit_for_hash = |hash: Hash64| {
            let hash: u64 = hash.into();
            let index = hash as usize % self.bits.len();
            self.bits.set(index, true);
        };

        self.builder
            .hashes_one(item)
            .take(H)
            .for_each(set_bit_for_hash);
    }

    /// Checks if a given item is present in the filter.
    ///
    /// # Example
    ///
    ///```
    /// use aabel_bloom_rs::*;
    /// use aabel_multihash_rs::*;
    ///
    /// // Create the hasher builder
    /// let keys1 = (0, 0);
    /// let keys2 = (1, 1);
    /// let builder = BuildPairHasher::new_with_keys(keys1, keys2);
    ///
    /// // Create the bloom filter
    /// let mut filter = BloomFilter::<[u8], _>::new(builder);
    ///
    /// // Insert and contains operations on the filter
    /// filter.insert(&[1u8, 2, 3]);
    /// let res = filter.contains(&[1u8, 2, 3].as_slice());
    /// assert!(res)
    ///
    ///```
    pub fn contains<U>(&mut self, item: &U) -> bool
    where
        T: Borrow<U>,
        U: Hash + ?Sized,
    {
        let get_bit_for_hash = |hash: Hash64| {
            let hash: u64 = hash.into();
            let index = hash as usize % self.bits.len();
            self.bits[index]
        };

        self.builder.hashes_one(item).take(H).all(get_bit_for_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aabel_multihash_rs::BuildPairHasher;

    #[test]
    fn insert_contains() {
        let keys1 = (0, 0);
        let keys2 = (1, 1);
        let builder = BuildPairHasher::new_with_keys(keys1, keys2);

        let mut filter = BloomFilter::<&str, _>::new(builder);

        // Insert an item in the bloom filter.
        let item = "Hello world!";
        filter.insert(item);

        // Check if the item is in the filter
        let res = filter.contains(item);
        assert!(res)
    }
}
