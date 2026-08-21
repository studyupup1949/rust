# accepts

[![Crates.io](https://img.shields.io/crates/v/accepts.svg)](https://crates.io/crates/accepts)
[![Docs.rs](https://docs.rs/accepts/badge.svg)](https://docs.rs/accepts)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/accepts-rs/accepts/actions/workflows/ci.yml/badge.svg)](https://github.com/accepts-rs/accepts/actions/workflows/ci.yml)

> ⚠️ **Pre-release**: version 0.0.2 is experimental. APIs and crate layout may change without backward compatibility guarantees.

`accepts` is the minimal core for composing synchronous and asynchronous
"acceptor" pipelines. It exposes only the traits and blanket implementations you
need to wire values through a chain of acceptors. There are no bundled acceptors;
those live in sibling crates.

For design details, see ARCHITECTURE.md in this repo.

## Naming

This crate only defines the core traits. See ARCHITECTURE.md for series lineup
and naming guidelines.

## Quickstart

### Add the dependency

```toml
[dependencies]
accepts = "0.0.2"
```

- Default features include `alloc`. Disable defaults for a core-only `no_std`
  build: `accepts = { version = "0.0.2", default-features = false }`.
  Re-enable `alloc` if you still want `Box`/`Arc` and dyn-async support.

### Implement and fan out

```rust
use accepts::Accepts;

struct Logger;
impl Accepts<&str> for Logger {
    fn accept(&self, value: &str) {
        println!("[log] {value}");
    }
}

struct Metrics;
impl Accepts<&str> for Metrics {
    fn accept(&self, value: &str) {
        println!("[metrics] saw {value}");
    }
}

// Tuples and collections forward to each element; `&str` is `Copy`, so cloning
// to fan out is cheap.
let pipeline = (Logger, Metrics);
pipeline.accept("ping");
```

Async pipelines follow the same shape with `AsyncAccepts`; enable `alloc` if you
want to box the returned futures via `DynAsyncAccepts`.

## Version map

| accepts | notes |
| --- | --- |
| 0.0.2 | current pre-release |

## More

See ARCHITECTURE.md for design notes, feature flags, and blanket impl coverage.

## Feature flags

| Feature | Default | Description |
| --- | --- | --- |
| `alloc` | ✅ | Adds heap-backed blanket impls (`Box`/`Rc`/`Arc`), `Vec`/`VecDeque` impls, and auto-implements `DynAsyncAccepts`. |

## Crate layout

- `Accepts`, `AsyncAccepts`, `DynAsyncAccepts` at the crate root.
- `implementations/core/*` for `no_std` blanket impls (tuples, `Option`,
  `Result`, slices/arrays, pointer types).
- `implementations/alloc/*` for heap-backed impls (`Vec`, `VecDeque`, pointers).

## License

This project is dual-licensed under MIT and Apache-2.0. You may use it under
either license.

## Contributing

Bug reports, feature requests, and pull requests are always welcome.

> I'm still new to GitHub and not very confident in English.
> For now, I'm doing my best with the help of ChatGPT,
> so I might misunderstand something. Thanks for your understanding!
