### 0.2.2

- test: Add snapshot tests
- docs: Add section layout diagram to `VendorHeader`
- feat: Add functions to calculate section positions in `VendorHeader`
- feat: Add `no_std` with `alloc` support
- feat: Add `EitherHeader`

### 0.2.1

- fix: Add stricter Clippy lints
  * Added `const` and `#[must_use]` when recommended
- build: Add MSRV (minimum supported Rust version)
- fix: Truncate first part of `OsVersion` to 7 bits
  * This bug was caught by Clippy!

### 0.2.0

- Combine `HeaderV0`'s cmdline fields
- Add `Header::cmdline()`
- Add warning about unreliable OS versioning

### 0.1.2

- Re-export `binrw`'s `BufReader`
- Add example
- Add description to `unpack_bootimg`'s Cargo.toml
- Expose `VendorHeaderV4`

### 0.1.1

- Handle unknown header version properly instead of panicking

### 0.1.0

- Initial release
