use std::hash::BuildHasher;

use aabel_multihash_rs::{self, BuildHasherExt, BuildPairHasher, Hash64};

#[test]
fn get_hashes() {
    let keys1 = (0, 0);
    let keys2 = (1, 1);
    let builder = BuildPairHasher::new_with_keys(keys1, keys2);

    let item = "Hello world!";
    const HASHES_COUNT: usize = 10;

    let hash = builder.hash_one(item);
    assert_ne!(hash, 0);

    let hashes = builder
        .hashes_one(item)
        .take(HASHES_COUNT)
        .collect::<Vec<_>>();
    assert!(hashes.into_iter().all(|hash| hash != Hash64::from(0)));
}
