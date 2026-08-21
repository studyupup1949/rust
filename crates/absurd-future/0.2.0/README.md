# absurd-future

[![crates.io](https://img.shields.io/crates/v/absurd-future.svg)](https://crates.io/crates/absurd-future)
[![docs.rs](https://docs.rs/absurd-future/badge.svg)](https://docs.rs/absurd-future)

A future adapter that turns a future that never resolves (i.e., returns `Infallible`) into a future that can resolve to any type.

This is useful in scenarios where you have a task that runs forever (like a background service) but need to integrate it into an API that expects a specific return type, such as `tokio::task::JoinSet`.

For a detailed explanation of the motivation behind this crate and the concept of uninhabited types in Rust async code, see the blog post: [How to use Rust's never type (!) to write cleaner async code](https://academy.fpblock.com/blog/rust-never-type-async-code).

## Usage

For a complete, runnable example of how to use this crate with `tokio::task::JoinSet`, please see the example file: [`examples/tokio.rs`](./examples/tokio.rs).

## License

This project is licensed under the MIT license.
