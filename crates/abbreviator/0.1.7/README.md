[![Actions Status](https://github.com/Vagelis-Prokopiou/abbreviator/workflows/Rust/badge.svg)](https://github.com/Vagelis-Prokopiou/abbreviator/actions)

# abbreviator

A Rust library for abbreviating long words.

## Example usage

Add the library to your dependencies
```toml
[dependencies]
abbreviator = "0.1.7"
```
and then use it.
```rust
use abbreviator::abbreviate;

fn main() {
    println!("{}", abbreviate("")); // prints ""
    println!("{}", abbreviate("a")); // prints "a"
    println!("{}", abbreviate("ab")); // prints "ab"
    println!("{}", abbreviate("abc")); // prints "a1c"
    println!("{}", abbreviate("word")); // prints "w2d"
    println!("{}", abbreviate("localization")); // prints "l10n"
    println!("{}", abbreviate("internationalization")); // prints "i18n"
    println!("{}", abbreviate("pneumonoultramicroscopicsilicovolcanoconiosis")); // prints "p43s"
}
```

