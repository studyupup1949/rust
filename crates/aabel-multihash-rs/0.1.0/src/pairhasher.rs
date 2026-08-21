use crate::{
    build_siphasher::{BuildSipHasher, SipHasherKeys},
    hash_iter::MultiHashIterator,
    HasherExt,
};
use rand::rngs::ThreadRng;
use std::hash::{BuildHasher, Hasher};

//
// PairHasher - a combinator of two hashers which
// can generate a sequence of hash values.
//

/// A [`Hasher`] which combines two [`Hasher`] instances. The hasher combinator
/// uses the two [`Hasher`] instances to generate sequences of hash values.
///
/// # Example
///
///```
/// use aabel_multihash_rs::*;
/// use std::hash::{BuildHasher, Hash};
///
/// let keys1 = (0, 0);
/// let keys2 = (1, 1);
/// let mut hasher = BuildPairHasher::new_with_keys(keys1, keys2).build_hasher();
///
/// let item = "Hello world";
/// item.hash(&mut hasher);
///
/// const HASHES_COUNT: usize = 10;
/// let hashes = hasher.finish_iter().take(HASHES_COUNT).collect::<Vec<_>>();
/// assert!(hashes.iter().all(|h| *h != 0));
///```
pub struct PairHasher<H1, H2> {
    hasher1: H1,
    hasher2: H2,
}

impl<H1, H2> PairHasher<H1, H2> {
    pub(crate) fn new(hasher1: H1, hasher2: H2) -> Self {
        Self { hasher1, hasher2 }
    }
}

impl<H1, H2> Hasher for PairHasher<H1, H2>
where
    H1: Hasher,
    H2: Hasher,
{
    fn finish(&self) -> u64 {
        let a = self.hasher1.finish();
        let b = self.hasher2.finish();
        a.wrapping_add(b)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.hasher1.write(bytes);
        self.hasher2.write(bytes);
    }
}

impl<H1, H2> HasherExt for PairHasher<H1, H2>
where
    H1: Hasher,
    H2: Hasher,
{
    fn finish_iter(self) -> impl Iterator<Item = u64> {
        let a = self.hasher1.finish();
        let b = self.hasher2.finish();
        MultiHashIterator::new(a, b)
    }
}

//
// A builder for PairHasher.
//

/// An instance of [`BuildHasher`] trait which builds [PairHasher] instances.
///
/// # Example
///
///```
/// use aabel_multihash_rs::*;
/// use std::hash::{BuildHasher, Hash};
///
/// let keys1 = (0, 0);
/// let keys2 = (1, 1);
/// let builder = BuildPairHasher::new_with_keys(keys1, keys2);
///
/// const HASHE_COUNT: usize = 10;
/// let item = "Hello world!";
///
/// let hashes = builder
///     .hashes_one(item)
///     .take(HASHE_COUNT)
///     .collect::<Vec<_>>();
/// assert_eq!(hashes.len(), HASHE_COUNT)
///```
pub struct BuildPairHasher<B1, B2> {
    builder1: B1,
    builder2: B2,
}

impl<B1, B2> BuildPairHasher<B1, B2> {
    pub fn new(builder1: B1, builder2: B2) -> Self {
        Self { builder1, builder2 }
    }
}

impl BuildPairHasher<BuildSipHasher, BuildSipHasher> {
    pub fn new_with_keys(keys1: SipHasherKeys, keys2: SipHasherKeys) -> Self {
        let builder1 = BuildSipHasher::from(keys1);
        let builder2 = BuildSipHasher::from(keys2);
        Self::new(builder1, builder2)
    }

    pub fn new_with_rng(rng: ThreadRng) -> Self {
        let builder1 = BuildSipHasher::from(rng.clone());
        let builder2 = BuildSipHasher::from(rng);
        Self::new(builder1, builder2)
    }
}

impl<B1, B2> BuildHasher for BuildPairHasher<B1, B2>
where
    B1: BuildHasher,
    B2: BuildHasher,
{
    type Hasher = PairHasher<B1::Hasher, B2::Hasher>;

    fn build_hasher(&self) -> Self::Hasher {
        let hasher1 = self.builder1.build_hasher();
        let hasher2 = self.builder2.build_hasher();
        PairHasher::new(hasher1, hasher2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BuildHasherExt, BuildPairHasher};
    use std::hash::{BuildHasher, Hash};

    #[test]
    fn build_hasherext() {
        let keys1 = (0, 0);
        let keys2 = (1, 1);
        let mut hasher = BuildPairHasher::new_with_keys(keys1, keys2).build_hasher();

        let item = "Hello world";
        item.hash(&mut hasher);
        const HASHES_COUNT: usize = 10;

        let hashes = hasher.finish_iter().take(HASHES_COUNT).collect::<Vec<_>>();
        assert!(hashes.iter().all(|h| *h != 0));
    }

    #[test]
    fn hashes_one() {
        let keys1 = (0, 0);
        let keys2 = (1, 1);

        let builder = BuildPairHasher::new_with_keys(keys1, keys2);
        const HASHE_COUNT: usize = 10;

        let item = "Hello world!";
        let hashes = builder
            .hashes_one(item)
            .take(HASHE_COUNT)
            .collect::<Vec<_>>();
        assert_eq!(hashes.len(), HASHE_COUNT)
    }

    #[test]
    fn hashes_eq() {
        let keys1 = (0, 0);
        let keys2 = (1, 1);
        let item = "Hello world!";
        const HASH_COUNT: usize = 10;

        let hashes1 = BuildPairHasher::new_with_keys(keys1, keys2)
            .hashes_one(item)
            .take(HASH_COUNT)
            .collect::<Vec<_>>();

        let hashes2 = BuildPairHasher::new_with_keys(keys1, keys2)
            .hashes_one(item)
            .take(HASH_COUNT)
            .collect::<Vec<_>>();

        assert_eq!(hashes1, hashes2)
    }
}
