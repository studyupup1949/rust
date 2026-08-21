# acmap

[English](README.md) | [中文](README.zh-CN.md)

`acmap` is an actor-style, sharded async map for Rust, built on `tokio` channels.

It provides a DashMap-like API surface for common operations, with two write paths:
- `insert`: request/response write (returns previous value)
- `insert_fast`: fire-and-forget write for high-throughput scenarios

## Features

- Sharded actor model for parallel write handling
- Async API for map operations
- Fast-path write API for lower overhead
- Simple benchmark example in `examples/benchmark.rs`

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
acmap = { path = "." }
```

## Quick Start

```rust
use acmap::AcMap;

#[tokio::main]
async fn main() {
    let map = AcMap::<u64, u64>::new();

    assert_eq!(map.insert(1, 10).await, None);
    assert_eq!(map.get(1).await, Some(10));

    map.insert_fast(2, 20);
    assert!(map.contains_key(2).await);

    assert_eq!(map.len().await, 2);
}
```

## Run

```bash
cargo test -q
cargo run --example benchmark -q
```

## Project Layout

- `src/acmap/mod.rs`: public API and sharding logic
- `src/acmap/messages.rs`: actor message definitions
- `src/acmap/shard.rs`: shard actor runtime loop
- `examples/benchmark.rs`: benchmark-like demo

## License

This project is licensed under the [MIT License](LICENSE).
