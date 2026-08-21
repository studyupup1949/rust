use rand::{rngs::ThreadRng, Rng};
use siphasher::sip::SipHasher;
use std::hash::BuildHasher;

pub type SipHasherKeys = (u64, u64);

/// A hasher builder for the [`SipHasher`] hasher. The builder implements the [`BuildHasher`] trait.
/// We use the [`SipHasher`] as default hasher for the [PairHasher] combinator.
pub struct BuildSipHasher {
    key0: u64,
    key1: u64,
}

impl From<SipHasherKeys> for BuildSipHasher {
    fn from(keys: SipHasherKeys) -> Self {
        Self {
            key0: keys.0,
            key1: keys.1,
        }
    }
}

impl From<ThreadRng> for BuildSipHasher {
    fn from(mut rng: ThreadRng) -> Self {
        let key0 = rng.gen();
        let key1 = rng.gen();

        (key0, key1).into()
    }
}

impl BuildHasher for BuildSipHasher {
    type Hasher = SipHasher;

    fn build_hasher(&self) -> Self::Hasher {
        SipHasher::new_with_keys(self.key0, self.key1)
    }
}
