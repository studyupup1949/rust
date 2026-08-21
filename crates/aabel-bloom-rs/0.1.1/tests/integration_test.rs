use aabel_bloom_rs::BloomFilter;
use aabel_multihash_rs::*;

#[test]
fn bloom_filter() {
    let keys1 = (0, 0);
    let keys2 = (1, 1);
    let builder = BuildPairHasher::new_with_keys(keys1, keys2);

    let mut filter = BloomFilter::<&str, _>::new(builder);

    let item = "Hello world!";

    // Insert an item in the bloom filter.
    filter.insert(item);
    filter.insert("Tessting testing");
    filter.insert("Rust rocks");
    filter.insert("In Rust we trust");

    // Check if the item is in the filter
    let res = filter.contains(item);
    assert!(res)
}
