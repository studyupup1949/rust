# abbreviator

A Rust library for abbreviating long words.


## Example usage
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

