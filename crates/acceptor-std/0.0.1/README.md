# acceptor-std

[![Crates.io](https://img.shields.io/crates/v/acceptor-std.svg)](https://crates.io/crates/acceptor-std)
[![Docs.rs](https://docs.rs/acceptor-std/badge.svg)](https://docs.rs/acceptor-std)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/acceptor-rs/acceptor-std/actions/workflows/ci.yml/badge.svg)](https://github.com/acceptor-rs/acceptor-std/actions/workflows/ci.yml)

`acceptor-std` is a `std`-powered bundle of acceptors built on the [`accepts`](https://crates.io/crates/accepts) core traits. It adds acceptors that require the standard library (e.g., `std::sync::mpsc`).

> ⚠️ **Pre-release**: version 0.0.1 is experimental. APIs and crate layout may change without backward compatibility guarantees.

## Add the dependency

```toml
[dependencies]
acceptor-std = "0.0.1"
```

## Example: forward through `std::sync::mpsc::Sender` and print the send result

```rust
use std::sync::mpsc::channel;

use acceptor_std::{MpscSender, DebugPrinter};
use accepts::Accepts;

let (tx, rx) = channel();
let sender = MpscSender::new(tx, DebugPrinter);

sender.accept(10);
sender.accept(20);
drop(sender); // close the channel

assert_eq!(rx.recv().unwrap(), 10);
assert_eq!(rx.recv().unwrap(), 20);
```

## Version map

| acceptor-std | accepts |
| --- | --- |
| 0.0.1 | 0.0.2 |

## More

See ARCHITECTURE.md for design notes and the broader acceptor series lineup.

## License

MIT OR Apache-2.0
