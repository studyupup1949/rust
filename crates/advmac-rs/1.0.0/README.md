![advMAC](https://raw.githubusercontent.com/GamePad64/advmac/main/logo.svg)

Advanced MAC address library.

[![Crates.io](https://img.shields.io/crates/v/advmac-rs?style=flat-square)](https://crates.io/crates/advmac-rs)
[![docs.rs](https://img.shields.io/docsrs/advmac-rs?style=flat-square)](https://docs.rs/advmac-rs/latest)

# Usage
Add this to your `Cargo.toml`:

```toml
[dependencies]
advmac-rs = "1.0.0"
```

## Features:
- EUI-48 and EUI-64.
- Extensive `no_std` support.
- `serde` support (even on `no_std`).
- `const fn` address parser with convenience macros for compile-time address handling: `mac6!`, `mac8!`.
- MAC address generation and editing.
