# academic-journals

[![Crates.io](https://img.shields.io/crates/v/academic-journals.svg)](https://crates.io/crates/academic-journals)
[![Docs.rs](https://docs.rs/academic-journals/badge.svg)](https://docs.rs/academic-journals)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)


> :warning: **DISCLAIMER: This crate is currently a work in progress (WIP).** It is not yet recommended for use in production environments. Features and functionality may change, and certain parts may not be fully implemented or tested.

`academic-journals` is a Rust library for managing and accessing journal abbreviations and full names. It's designed to efficiently handle a large dataset of journal entries and provides functionalities to retrieve either the abbreviation from a full journal name or vice versa.

## Acknowledgments

This crate makes use of data from abbrv.jabref.org provided by JabRef. The journal abbreviation data is released under the CC0 1.0 Universal (CC0 1.0) Public Domain Dedication. We gratefully acknowledge their work and contributions to the academic community.



## Features

- Efficiently handle large datasets of journal names and their abbreviations.
- Retrieve the abbreviation for a given full journal name.
- Retrieve the full journal name from a given abbreviation.

## Usage

Add `academic-journals` to your `Cargo.toml`:

```toml
[dependencies]
academic-journals = "0.1.1"
```

In your Rust file:

```rust
use rust_journals::{get_abbreviation, get_full_name};

fn main() {
	let full_name = "Journal of Artificial Intelligence Research";
	if let Some(abbreviation) = get_abbreviation(full_name) {
		println!("Abbreviation for {}: {}", full_name, abbreviation);
	}

	let abbreviation = "JAIR";
	if let Some(name) = get_full_name(abbreviation) {
		println!("Full name for {}: {}", abbreviation, name);
	}
}
```

## Build Instructions

To build the crate, run:

```bash
cargo build
```

## Contributions

Contributions are welcome! Please feel free to submit a pull request.

## License

This project is licensed under the Apache 2.0 license. See the [LICENSE](LICENSE) file for more details.
