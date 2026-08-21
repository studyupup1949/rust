use ab_radix_trie::Trie;

fn main() {
    let mut trie: Trie<String> = Trie::new();
    trie.insert("romanus", None);
    trie.insert("romulus", None);
    trie.insert("rubens", None);
    trie.insert("ruber", None);
    trie.insert("rubicon", None);
    trie.insert("rubicundus", None);

// get suffix_tree under "rom"
    let suffix_trie = trie.suffix_tree("rom");
    println!("suffix tree = {:#?}", suffix_trie);

// get all suffixes under "rom" (flattened)
    let results = trie.get_suffixes_values("rom");
    println!("suffixes = {:?}", results);
}

