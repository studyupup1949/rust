# acceptor-alloc

[![Crates.io](https://img.shields.io/crates/v/acceptor-alloc.svg)](https://crates.io/crates/acceptor-alloc)
[![Docs.rs](https://docs.rs/acceptor-alloc/badge.svg)](https://docs.rs/acceptor-alloc)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/acceptor-rs/acceptor-alloc/actions/workflows/ci.yml/badge.svg)](https://github.com/acceptor-rs/acceptor-alloc/actions/workflows/ci.yml)

`acceptor-alloc` is an `alloc`-powered bundle of acceptors built on the [`accepts`](https://crates.io/crates/accepts) core traits. It provides acceptors that require heap-backed collections without pulling in `std`.

> ⚠️ **Pre-release**: version 0.0.1 is experimental. APIs and crate layout may change without backward compatibility guarantees.

## Add the dependency

```toml
[dependencies]
acceptor-alloc = "0.0.1"
```

## Example: route `(Key, Value)` pairs using `BTreeKVRouter`

```rust
use alloc::collections::BTreeMap;
use acceptor_alloc::BTreeKVRouter;
use accepts::Accepts;

#[derive(Clone, Debug, Default)]
struct Sink;
impl Accepts<(char, i32)> for Sink {
    fn accept(&self, pair: (char, i32)) {
        core::hint::black_box(pair);
    }
}

let mut routes = BTreeMap::new();
routes.insert('a', Sink);

// fallback is a no-op `()`, which implements `Accepts`.
let router = BTreeKVRouter::new(routes);

router.accept(('a', 1)); // routed to Sink
router.accept(('z', 2)); // routed to fallback (no-op)
```

## Version map

| acceptor-alloc | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## More

See ARCHITECTURE.md for design notes and the broader acceptor series lineup.

## License

MIT OR Apache-2.0
