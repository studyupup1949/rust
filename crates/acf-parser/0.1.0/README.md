# `acf-parser`

[![crates.io](https://img.shields.io/crates/v/acf-parser.svg)](https://crates.io/crates/acf-parser)
[![docs](https://docs.rs/acf-parser/badge.svg)](https://docs.rs/acf-parser)
[![license](https://img.shields.io/crates/l/acf-parser)](https://github.com/DMoore12/acf-parser#license)
[![crates.io](https://img.shields.io/crates/d/acf-parser.svg)](https://crates.io/crates/acf-parser)

*A simple ACF parser, targeted at reading Valve configuration files*

`acf-parser` is a lightweight Rust parser leveraging `Chumsky` for performant file parsing.
The parser returns a vector of entries, each containing a `HashMap` with all listed elements.

**NOTE: The Valve ACF format is not openly published. This project takes a stab at parsing the format based on a small selection of `.acf` inputs. Accuracy cannot be guaranteed for all input files!**

## Usage

```rust
use acf_parser::prelude::*;

fn main() {
    let result = parse_acf("./acfs/simple.acf");

    let result = result.unwrap();
    let root_entry = &result.entries[0];
    let root_contents = &root_entry.expressions;

    println!("Found root entry '{}'", root_entry.name);
    println!("App name: {}", root_contents["name"]);
    println!("App ID: {}", root_contents["appid"]);
}
```