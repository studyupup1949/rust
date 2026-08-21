use ab_radix_trie::Trie;
use ab_radix_trie::MatchingOptions;
fn main() {
    let mut trie: Trie<String> = Trie::new();
    trie.insert("romanus", None);
    trie.insert("rom anus", None);
    trie.insert("romulus", None);
    trie.insert("rubens", None);
    trie.insert("ruber", None);
    trie.insert("rubicon", None);
    trie.insert("rubicundus", None);

    let results = trie.get_suffixes_with_matching_options("roma", &MatchingOptions::ignoring_white_space());
    // should return "romanus" and "rom anus"
    println!("{:?}", results);
}

