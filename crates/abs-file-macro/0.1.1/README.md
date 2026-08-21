# abs-file-macro

[![made-with-rust][rust-logo]][rust-src-page]
[![crates.io][crates-badge]][crates-page]
[![Documentation][docs-badge]][docs-page]
[![MIT licensed][license-badge]][license-page]


| OS            | Status                                                                               |
|---------------|--------------------------------------------------------------------------------------|
| Ubuntu-latest | [![Ubuntu Tests][ubuntu-latest-badge]][ubuntu-latest-workflow]                       |
| macOS-latest  | [![macOS Tests][macos-latest-badge]][macos-latest-workflow]                          |
| Windows-latest| [![Windows Tests][windows-latest-badge]][windows-latest-workflow]                    |

A macro that returns the absolute file path of the Rust source file in which it is invoked.

This macro ensures that the correct absolute path is resolved, even when used within a Cargo workspace or a nested crate. It prevents issues with duplicated path segments by properly handling the crate root.

## Install

```sh
cargo add abs-file-macro
```

## Usage

### Example
```rust
use abs_file_macro::abs_file;

let path = abs_file!();

assert!(
    std::fs::metadata(&path).is_ok(),
    "abs_file!() should point to a real file, but {:?} does not exist",
    path
);
```

## Testing

To ensure correctness, test this macro in two ways:

1. **As part of the full workspace**

   ```sh
   cargo test --workspace
   ```
    This verifies that the macro functions correctly across all workspace members.

2. **Within an individual workspace member**

   ```sh
   cd test-workspace-1
   cargo test
   ```
    This helps catch cases where path resolution might differ based on the working directory.

## License

[MIT License](LICENSE) (c) 2025 Jeremy Harris.

[rust-src-page]: https://www.rust-lang.org/
[rust-logo]: https://img.shields.io/badge/Made%20with-Rust-black?&logo=Rust

[crates-page]: https://crates.io/crates/abs-file-macro
[crates-badge]: https://img.shields.io/crates/v/abs-file-macro.svg

[docs-page]: https://docs.rs/abs-file-macro
[docs-badge]: https://docs.rs/abs-file-macro/badge.svg

[license-page]: ./LICENSE
[license-badge]: https://img.shields.io/badge/license-MIT-blue.svg

[ubuntu-latest-badge]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20ubuntu-latest)
[ubuntu-latest-workflow]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml?query=branch%3Amain

[macos-latest-badge]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20macos-latest)
[macos-latest-workflow]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml?query=branch%3Amain

[windows-latest-badge]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml/badge.svg?branch=main&job=Run%20Rust%20Tests%20(OS%20=%20windows-latest)
[windows-latest-workflow]: https://github.com/jzombie/rust-abs-file-macro/actions/workflows/rust-tests.yml?query=branch%3Amain
