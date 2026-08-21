# AAML (Advanced/Another Abstract Markup Language)

A robust and lightweight configuration parser for Rust that supports simple key-value pairs, recursive dependency resolution, and bidirectional lookups. Designed for applications that need flexible configuration files with references and aliases.

## 🚀 Features

* **Simple Syntax**: specific `key = value` format that is easy to read and write.
* **Comments Support**: Lines starting with `#` are treated as comments.
* **Deep Resolution (`find_deep`)**: Automatically resolves chains of references (e.g., `A -> B -> C`) to find the final value.
* **Loop Detection**: Safely handles circular dependencies (e.g., `A -> B -> A`) without stack overflows.
* **Bidirectional Lookup (`find_obj`)**: Can find a value by key, or essentially "reverse lookup" a key by its value if the direct key doesn't exist.
* **Mutable Result Wrapper**: Returns a `FoundValue` struct that acts like a String but includes helper methods for in-place modification.

## 📦 Installation

Add the library to your `Cargo.toml`.

```toml
[dependencies]
aaml = "1.0.0"
```

# Configuration Syntax (.aam)

The format is line-based. Whitespace around keys and values is trimmed.
Properties
```aam
# This is a comment
host = localhost
port = 8080

# You can define aliases for deep lookup
base_path = /var/www
current_path = base_path

# Circular references are handled safely
loop_a = loop_b
loop_b = loop_a
```

# Usage Guide
## 1. Parsing Configuration

You can parse configuration directly from a string or load it from a file.
Rust

```rust
use aaml::aaml::AAML;

fn main() {
    // 1. Parse from string
    let content = "
        username = admin
        timeout = 30
    ";
    let config = AAML::parse(content);

    // 2. Load from file
    // let config = AAML::load("config.aaml").expect("Failed to read file");
}
```

## 2. Smart Lookup (find_obj)

The find_obj method is a hybrid lookup tool. It prioritizes finding a value by the given key. If the key is not found, it searches for a key that holds the provided string as a value.
```rust
let content = "
    # Key = Value
    app_mode = production
    debug    = false
";
let config = AAML::parse(content);

// Scenario A: Direct Key Lookup
// Finds value "production" for key "app_mode"
let mode = config.find_obj("app_mode").unwrap();
assert_eq!(mode, "production");

// Scenario B: Reverse Lookup
// "production" is not a key, so it looks for a key with value "production"
let key = config.find_obj("production").unwrap();
assert_eq!(key, "app_mode");
```

## 3. Deep Recursive Lookup (find_deep)

This feature is useful for aliasing. It follows the values as keys until it reaches a value that doesn't exist as a key in the map, or until a loop is detected.
```rust

let content = "
    root = /usr/bin
    executable = root
    service = executable
";
let config = AAML::parse(content);

// Traces: "service" -> "executable" -> "root" -> "/usr/bin"
let final_val = config.find_deep("service").unwrap();

assert_eq!(final_val, "/usr/bin");

Handling Loops:
If the configuration contains a loop (e.g., a=b, b=a), find_deep will return the last unique value visited before the loop closed, preventing infinite recursion.
Rust

// a -> b -> a ...
let val = config.find_deep("a").unwrap();
assert_eq!(val, "b"); // Returns the last valid step
```

## 4. Working with FoundValue

The results are wrapped in a FoundValue struct. It implements Deref<Target=String> and Display, so you can use it just like a regular &str or String. It also provides methods for modification.
Rust

```rust

let config = AAML::parse("greeting = Hello World");
let mut value = config.find_obj("greeting").unwrap();

// Use as a string
println!("Original: {}", value); // Prints: Hello World

// Modify in-place using the helper method
value.remove(" World");

assert_eq!(value.as_str(), "Hello");
```


📚 API Reference
AAML Struct

    parse(content: &str) -> Self: Parses a string into the AAML map.

    load<P: AsRef<Path>>(file_path: P) -> Result<Self, String>: Loads and parses a file.

    find_obj(&self, key: &str) -> Option<FoundValue>: Smart bidirectional search.

    find_deep(&self, key: &str) -> Option<FoundValue>: Recursive search with loop detection.

    find_key(&self, value: &str) -> Option<FoundValue>: Strict reverse lookup (find key by value).

FoundValue Struct

    new(value: &str) -> FoundValue: Creates a new instance.

    remove(&mut self, target: &str) -> &mut Self: Removes occurrences of target string from the value.

    as_str(&self) -> &str: Returns the inner string slice.

    Traits: Implements Display, Debug, Clone, PartialEq, Eq, Deref.

# Testing

The library comes with a comprehensive test suite covering parsing, deep loops, and modification logic.

```shell
cargo test
```
# License

License can be found on ```LICENSE``` file.