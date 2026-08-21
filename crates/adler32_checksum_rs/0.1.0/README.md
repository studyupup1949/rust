# adler32_checksum_rs

![License](https://img.shields.io/github/license/Derghust/adler32_checksum_rs)
![Issues](https://img.shields.io/github/issues/Derghust/adler32_checksum_rs)
![CI](https://img.shields.io/github/workflow/status/Derghust/adler32_checksum_rs/Continuous%20integration/main)
![Activity](https://img.shields.io/github/commit-activity/m/Derghust/adler32_checksum_rs/main)
![Version](https://img.shields.io/badge/version-v0.1.0-4F4FFF)

Adler 32 checksum algorithm written for rust.

[Wikipedia](https://en.wikipedia.org/wiki/Adler-32)

## How to use

### Sequel

```rust
fn adler32_checksum_blocking(init: [u8; 8], data: Vec<u8>) -> Adler32Result {
    let adler = Adler32::new(init);
    adler.adler32_checksum(data);
}
```

### Batch

#### Synchronously

```rust
fn adler32_checksum_blocking(init: [u8; 8], data: Vec<Vec<u8>>) -> Vec<Adler32Result> {
    let adler = Adler32::new(init);
    data
    .iter()
    .map(|hash| {
        adler.adler32_checksum(hash);
    })
    .collect();
}
```

#### Asynchronously

```rust
fn adler32_checksum_parallel(init: [u8; 8], data: Vec<Vec<u8>>) -> Vec<Adler32Result> {
    Adler32Builder::new(Adler32::new(init))
        .push_vec(data)
        .finalize();
}
```
