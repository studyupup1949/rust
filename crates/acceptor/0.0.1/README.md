# acceptor

[![Crates.io](https://img.shields.io/crates/v/acceptor.svg)](https://crates.io/crates/acceptor)
[![Docs.rs](https://docs.rs/acceptor/badge.svg)](https://docs.rs/acceptor)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/acceptor-rs/acceptor/actions/workflows/ci.yml/badge.svg)](https://github.com/acceptor-rs/acceptor/actions/workflows/ci.yml)

`acceptor` is a no_std bundle of thin acceptors built on the [`accepts`](https://crates.io/crates/accepts) core traits (`Accepts`, `AsyncAccepts`, `DynAsyncAccepts`). It has no standard library or external dependencies. The singular *acceptor* here means “a bundle of acceptors” without implying a specific count.

> ⚠️ **Pre-release**: version 0.0.1 is experimental. APIs and crate layout may change without backward compatibility guarantees.

## Add the dependency

```toml
[dependencies]
accepts = { version = "0.0.2", default-features = false } # no_std
acceptor = { version = "0.0.1" }
```

## Example: filter + map chain

```rust
use acceptor::{Filter, Map, StatefulCallback};

let sink = StatefulCallback::new((), |_state: &(), v: i32| {
    // terminal example without shared state
    core::hint::black_box(v);
});
let pipeline = Map::new(|v: i32| v * 2, Filter::new(|v: &i32| v % 2 == 0, sink));

pipeline.accept(3); // filtered out
pipeline.accept(4); // 8 is processed
```

## Version map

| acceptor | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## More

See ARCHITECTURE.md for naming guidelines, layout, and design notes.

## License

MIT OR Apache-2.0
