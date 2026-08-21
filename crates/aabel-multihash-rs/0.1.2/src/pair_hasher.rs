use crate::{Hash64, HasherExt};
use std::hash::Hasher;

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
/// assert!(hashes.into_iter().all(|h| h != Hash64::from(0)));
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
    fn finish_iter(self) -> impl Iterator<Item = Hash64> {
        let a = self.hasher1.finish();
        let b = self.hasher2.finish();

        PairHasherIterator::new(a, b)
    }
}

pub(crate) struct PairHasherIterator {
    a: u64,
    b: u64,
    c: u64,
}

impl PairHasherIterator {
    pub(crate) fn new(a: u64, b: u64) -> Self {
        Self {
            a,
            b,
            c: Default::default(),
        }
    }
}

impl Iterator for PairHasherIterator {
    type Item = Hash64;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.a;
        self.a = self.a.wrapping_add(self.b);
        self.b = self.b.wrapping_add(self.c);
        self.c += self.c.wrapping_add(1);

        Some(ret.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siphasher::sip::SipHasher;
    use std::hash::Hash;

    #[test]
    fn hash_finish() {
        let hasher1 = SipHasher::new_with_keys(0, 0);
        let hasher2 = SipHasher::new_with_keys(1, 1);
        let mut hasher = PairHasher::new(hasher1, hasher2);

        let item = "Hello world!";

        item.hash(&mut hasher);
        let hash = hasher.finish();
        assert_ne!(hash, 0);
    }

    #[test]
    fn hash_finish_iter() {
        let hasher1 = SipHasher::new_with_keys(0, 0);
        let hasher2 = SipHasher::new_with_keys(1, 1);
        let mut hasher = PairHasher::new(hasher1, hasher2);

        let item = "Hello world!";
        const HASHES_COUNT: usize = 10;

        item.hash(&mut hasher);

        // Calling hasher.finish here is optional.
        let _ = hasher.finish();

        let hashes = hasher.finish_iter().take(HASHES_COUNT).collect::<Vec<_>>();
        assert!(hashes.into_iter().all(|h| h != Hash64::from(0)))
    }
}
