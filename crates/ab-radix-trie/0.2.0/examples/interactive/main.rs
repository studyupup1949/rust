use ab_radix_trie::Trie;
use std::io::stdin;
use ab_radix_trie::MatchingOptions;
fn main() {
    let mut trie: Trie<i32> = Trie::new();
    trie.insert("romanus", None);
    trie.insert("romulus", Some(10));
    trie.insert("rubens", None);
    trie.insert("ruber", None);
    trie.insert("rubicon", None);
    trie.insert("rubicundus", None);

    loop {
        println!("enter search term:");
        let mut input_string = String::new();
        stdin()
            .read_line(&mut input_string)
            .expect("Failed to read line");

        let results = trie.get_suffixes_with_matching_options(
            input_string.trim_end(),
            &MatchingOptions::ignoring_white_space_and_new_lines(),
        );
        println!("{:#?}", results);
    }
}