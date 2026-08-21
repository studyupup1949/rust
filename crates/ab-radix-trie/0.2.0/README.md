[![Latest Version]][crates.io]

[crates.io]: https://crates.io/crates/ab-radix-trie
[Latest Version]: https://img.shields.io/crates/v/ab.svg


# Radix-Trie
Radix-trie implementation i.e. compressed prefix-trie

https://en.wikipedia.org/wiki/Radix_tree

## Some nice features:

1. Compressed nodes
2. Fuzzy matching - match on whitespace, replacing characters, etc.
3. Supports all unicode characters
4. Arbitrarily associate values to text (i.e. map strings to values)
5. Serializable with `serde`

## Performance

Approximately:

1. insertion O(depth of trie)
2. retrieval O(depth of trie)
3. deletion O(depth of trie)
4. Space - I don't know really, but it will behave according to ~O(entropy of text) - Similar texts are compressed together - i.e. "ABC", "ABCD" will occupy O("ABCD") space split into "ABC" and "D"


# Usage:

I suggest checking out the examples and the tests for some patterns

The basic usage is along these lines
```rust
let mut trie: Trie<i32> = Trie::new();
trie.insert("romanus", None);
trie.insert("romulus", Some(10));
trie.insert("rubens", None);
trie.insert("ruber", None);
trie.insert("rubicon", None);
trie.insert("rubicundus", None);
```
