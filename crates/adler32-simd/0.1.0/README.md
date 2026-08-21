# adler32-simd

SIMD-accelerated Adler-32 checksum for Rust.

Vectorized implementations for **ARM64 NEON** and **x86/x86\_64 SSSE3**, with an automatic scalar fallback on other platforms. Zero dependencies and `no_std` compatible.

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
adler32-simd = "0.1"
```

One-shot:

```rust
let checksum = adler32_simd::adler32(b"Hello, world!");
```

Streaming:

```rust
let mut hasher = adler32_simd::Adler32::new();
hasher.write(b"Hello, ");
hasher.write(b"world!");
let checksum = hasher.checksum();
```

`Adler32` also implements `std::hash::Hasher` (with the default `std` feature):

```rust
use std::hash::Hasher;

let mut hasher = adler32_simd::Adler32::new();
hasher.write(b"data");
let checksum = hasher.finish(); // returns u64
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | Yes     | Enables runtime CPU feature detection and `Hasher` impl. Disable for `no_std`. |

To use in a `no_std` environment:

```toml
[dependencies]
adler32-simd = { version = "0.1", default-features = false }
```

When `std` is disabled, SIMD is used only if the target feature is enabled at compile time (e.g. `-C target-feature=+neon`). Otherwise the scalar fallback is used.

## Platform support

| Architecture | Instruction set | Detection |
|---|---|---|
| `aarch64` | NEON | Runtime (std) / compile-time (no\_std) |
| `x86` / `x86_64` | SSSE3 | Runtime (std) / compile-time (no\_std) |
| Everything else | Scalar | Automatic fallback |

## License

MIT
