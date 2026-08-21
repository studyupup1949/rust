# ades

[![CI](https://github.com/lfern/eidas-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/lfern/eidas-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/ades-rs.svg)](https://crates.io/crates/ades-rs)
[![docs.rs](https://docs.rs/ades-rs/badge.svg)](https://docs.rs/ades-rs)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/lfern/eidas-rs)

AdES digital signatures (CAdES, PAdES, XAdES, JAdES) for the eIDAS 2.0 ecosystem, written in pure Rust.

All signature formats are validated by the [EU DSS validator](https://dss.nowina.lu/validation).

## Supported formats

| Format | B-B | B-T | B-LT |
|--------|-----|-----|------|
| CAdES  | ✅  | ✅  | ✅   |
| PAdES  | ✅  | ✅  | ✅   |
| XAdES  | ✅  | ✅  | ✅   |
| JAdES  | ✅  | ✅  | ✅   |

- **B-B** — baseline signature with embedded certificate
- **B-T** — adds an RFC 3161 timestamp from a TSA
- **B-LT** — adds OCSP revocation data

## Quick start

```toml
[dependencies]
ades = "0.1"
```

```rust
use ades::{cades, signer::SoftSigner};

let signer = SoftSigner::generate(2048)?;
let data = b"hello world";
let signature = cades::sign(data, &signer)?;
```

## Features

| Feature  | Default | Description |
|----------|---------|-------------|
| `cades`  | ✅      | CAdES signatures (CMS/PKCS#7) |
| `pades`  | ✅      | PAdES signatures (PDF) |
| `soft`   | ✅      | Software RSA/ECDSA key generation for testing |
| `xades`  |         | XAdES signatures (XML) |
| `jades`  |         | JAdES signatures (JSON/JWS) |
| `tsp`    |         | RFC 3161 timestamp client (B-T, B-LT levels) |
| `ocsp`   |         | RFC 6960 OCSP client (B-LT level) |
| `pkcs11` |         | PKCS#11 backend for hardware tokens (DNIe, HSM) |

## Signing backends

The `Signer` trait abstracts the private key:

- **`soft`** — `SoftSigner`: RSA/ECDSA keys in memory (testing)
- **`pkcs11`** — `Pkcs11Signer`: hardware tokens via PKCS#11 (DNIe, HSM)

The private key never leaves the device.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
