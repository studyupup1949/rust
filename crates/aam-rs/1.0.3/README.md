# AAM (Abstract Alias Mapping)

A robust and lightweight configuration parser for Rust that supports key-value pairs, recursive dependency resolution, file imports, and bidirectional lookups. Designed for applications that need flexible configuration files with references, aliases, and a modular structure.

## 🚀 Features

- **Simple syntax**: A `key = value` format that is easy to read and write.
- **Import support**: The `@import` directive lets you split configuration into multiple files.
- **Comments support**: Lines starting with `#` are treated as comments.
- **Deep resolution (`find_deep`)**: Automatically resolves chains of references (e.g., `A -> B -> C`) to find the final value.
- **Loop detection**: Safely handles circular dependencies (e.g., `A -> B -> A`) without stack overflows.
- **Bidirectional lookup (`find_obj`)**: Looks up a value by key, or performs a reverse lookup (finds a key by value) when the key is missing.
- **Config builder (`AAMBuilder`)**: Programmatically generate and save `.aam` files.
- **Configuration merging**: Supports the `+` operator to combine two `AAML` instances.
- **Typed errors**: Detailed parsing and I/O error handling via `AamlError`.

## 📦 Installation

Add the library to your `Cargo.toml`:

```toml
[dependencies]
aaml = "1.0.0"
```

## Configuration syntax (.aam)

The format is line-based. Whitespace around keys and values is trimmed. Strings can be quoted.

```aam
# This is a comment
host = "localhost"
port = 8080

# Import other configuration files
@import "database.aam"
@import "theme.aam"

# You can define aliases for deep lookup
base_path = /var/www
current_path = base_path

# Circular references are handled safely
loop_a = loop_b
loop_b = loop_a
```

## Usage guide

### 1) Parsing and loading

You can parse configuration from a string, load it from a file, or merge multiple sources. Errors are handled via `AamlError`.

```rust
use aaml::aaml::AAML;
use aaml::error::AamlError;

fn main() -> Result<(), AamlError> {
    // 1. Parse from string
    let content = "
        username = admin
        timeout = 30
    ";
    let config = AAML::parse(content)?;

    // 2. Load from file (supports @import directives)
    let file_config = AAML::load("config.aam")?;

    Ok(())
}
```

### 2) Merging configurations

Combine different `AAML` objects using the addition operator.

```rust
let mut config1 = AAML::parse("a = 1")?;
let config2 = AAML::parse("b = 2")?;

// Merge (config2 overwrites matching keys in config1)
config1 += config2;
// or: let config3 = config1 + config2;
```

### 3) Smart lookup (find_obj)

`find_obj` is a hybrid lookup method. It first tries to find a value by the given key. If the key does not exist, it searches for a key whose value matches the provided string.

```rust
let content = "
    # Key = Value
    app_mode = production
    debug    = false
";
let config = AAML::parse(content)?;

// Scenario A: Direct key lookup
let mode = config.find_obj("app_mode").unwrap();
assert_eq!(mode, "production");

// Scenario B: Reverse lookup
// "production" is not a key, so it looks for a key with value "production"
let key = config.find_obj("production").unwrap();
assert_eq!(key, "app_mode");
```

### 4) Deep recursive lookup (find_deep)

This is useful for aliasing. It follows values as keys until it reaches a value that is not present as a key, or until a loop is detected.

```rust
let content = "
    root = /usr/bin
    executable = root
    service = executable
";
let config = AAML::parse(content)?;

// Traces: "service" -> "executable" -> "root" -> "/usr/bin"
let final_val = config.find_deep("service").unwrap();
assert_eq!(final_val, "/usr/bin");
```

**Handling loops**: If the configuration contains a loop (e.g., `a=b`, `b=a`), `find_deep` returns the last unique value visited before the loop closes, preventing infinite recursion.

### 5) Building configurations (AAMBuilder)

Use `AAMBuilder` to generate configuration files programmatically.

```rust
use aaml::builder::AAMBuilder;

let mut builder = AAMBuilder::new();
builder.add_line("host", "127.0.0.1");
builder.add_line("port", "8000");
builder.add_raw("# Custom comment section");
builder.add_line("debug", "true");

// Save to file
builder.to_file("generated_config.aam");

// Or convert to string
println!("{}", builder);
```

### 6) Working with FoundValue

Lookup results are wrapped in a `FoundValue` struct. It implements `Deref<Target=String>` and `Display`, so you can use it like a regular `&str` or `String`. It also provides helper methods for in-place modification.

```rust
let config = AAML::parse("greeting = Hello World")?;
let mut value = config.find_obj("greeting").unwrap();

// Use as a string
println!("Original: {}", value); // Prints: Hello World

// Modify in-place using the helper method
value.remove(" World");
assert_eq!(value.as_str(), "Hello");
```

## API reference

### AAML

- `parse(content: &str) -> Result<Self, AamlError>`: Parses a string into an AAML map.
- `load<P: AsRef<Path>>(file_path: P) -> Result<Self, AamlError>`: Loads and parses a file, handling imports.
- `merge_content(&mut self, content: &str) -> Result<(), AamlError>`: Merges content into the current instance.
- `merge_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), AamlError>`: Reads a file and merges it.
- `find_obj(&self, key: &str) -> Option<FoundValue>`: Smart bidirectional lookup.
- `find_deep(&self, key: &str) -> Option<FoundValue>`: Recursive lookup with loop detection.
- `find_key(&self, value: &str) -> Option<FoundValue>`: Strict reverse lookup (find key by value).

### AAMBuilder

- `new() -> Self`: Creates a new builder.
- `add_line(key: &str, value: &str)`: Adds a `key = value` pair.
- `add_raw(raw_line: &str)`: Adds a raw line (e.g., a comment).
- `to_file<P: AsRef<Path>>(&self, path: P)`: Writes the buffer to a file.

### AamlError

- `IoError`: Wraps standard I/O errors.
- `ParseError`: Syntax errors (includes line number and details).
- `NotFound`: Key not found (internal use).

## License

See the `LICENSE` file.
