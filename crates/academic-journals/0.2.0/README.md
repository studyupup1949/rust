# academic-journals

[![Crates.io](https://img.shields.io/crates/v/academic-journals.svg)](https://crates.io/crates/academic-journals)
[![Docs.rs](https://docs.rs/academic-journals/badge.svg)](https://docs.rs/academic-journals)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

`academic-journals` is a Rust library for managing and accessing journal abbreviations and full names. It's designed to
efficiently handle a large dataset of journal entries and provides fast lookups to retrieve either the abbreviation
from a full journal name or vice versa.

## Acknowledgments

This crate makes use of data from abbrv.jabref.org provided by JabRef. The journal abbreviation data is released under
the CC0 1.0 Universal (CC0 1.0) Public Domain Dedication. We gratefully acknowledge their work and contributions to the
academic community.

## Features

- **Fast lookups**: Uses pre-built HashMaps for O(1) journal name and abbreviation lookups
- **Large dataset**: Includes thousands of journals from multiple academic disciplines
- **Multiple abbreviation styles**: Supports both dotted and dotless abbreviation formats
- **Zero-cost at runtime**: Journal data is embedded in the binary at compile time
- **Configurable**: Choose between online (fetch latest data) or offline mode

## Usage

Add `academic-journals` to your `Cargo.toml`:

```toml
[dependencies]
academic-journals = "0.2.0"
```

### Basic Example

```rust
use academic_journals::{get_abbreviation, get_full_name};

fn main() {
    // Get abbreviation from full name
    let full_name = "Critical Care Medicine";
    if let Some(abbreviation) = get_abbreviation(full_name) {
        println!("Abbreviation: {}", abbreviation);
    }

    // Get full name from abbreviation
    let abbreviation = "Crit Care Med";
    if let Some(name) = get_full_name(abbreviation) {
        println!("Full name: {}", name);
    }
}
```

## Features

### Default Features

- `dotless`: Uses dotless abbreviations (e.g., "Crit Care Med")
- `online`: Downloads latest journal data from JabRef during build

### Optional Features

- `dot`: Uses dot-formatted abbreviations (e.g., "Crit. Care Med.")

You can customize features in your `Cargo.toml`:

```toml
[dependencies]
# Use only dotted abbreviations, offline mode
academic-journals = { version = "0.1.3", default-features = false, features = ["dot"] }
```

## Building

```bash
# Build with default features (dotless + online)
cargo build

# Build offline with pre-packaged data
cargo build --no-default-features --features dotless

# Build with dot abbreviations
cargo build --no-default-features --features "dot,online"
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with specific features
cargo test --no-default-features --features dotless
cargo test --no-default-features --features dot
```

## Contributions

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the Apache 2.0 license. See the [LICENSE](LICENSE) file for more details.
