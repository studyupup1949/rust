# advise

[![latest version][version.badge]][version.hyper]
[![dependency status][deps.badge]][deps.hyper]
[![documentation][docs.badge]][docs.hyper]
[![license][license.badge]](#license)

## About

This crate offers a streamlined way to keep users informed about your program's
progress and potential issues. It provides:

- Convenience macros: Advise at familiar logging levels `error!`, `warn!`,
  `info!`, `debug!`, and `trace!` to categorize your messages.
- Customizable tags: Design your own message tags using the `Render` trait,
  tailoring the specific needs to your application.

## Usage

`advise` provides a simple [API][docs.hyper], similar to the [`log`][log] crate,
for printing status messages that should be enough for the majority of
use-cases.

```rust
use advise::{info, warn};

fn main() {
    warn!("It's dangerous to go alone!");
    info!("Take this.");
}
```

Running the above code will print the following to stderr.

<pre>
<span style="color:orange">warn</span><b>:</b> It's dangeous to go alone!
<span style="color:green">info</span><b>:</b> Take this.
</pre>

> [!IMPORTANT]
>
> GitHub currently does not support coloured text in markdown. You can run this
> [example] with `cargo run --example=basic` to see the above output rendered in
> colour.

## License

This project is dual-licensed under both [MIT License](./LICENSE-MIT) and
[Apache License 2.0](./LICENSE-APACHE). You have permission to use this code
under the conditions of either license pursuant to the rights granted by the
chosen license.

<!--
  Reference-style links
-->

<!-- Badges -->
[deps.badge]:    https://deps.rs/repo/github/kaplanz/advise/status.svg
[deps.hyper]:    https://deps.rs/repo/github/kaplanz/advise
[docs.badge]:    https://docs.rs/advise/badge.svg
[docs.hyper]:    https://docs.rs/advise
[license.badge]: https://img.shields.io/crates/l/advise.svg
[version.badge]: https://img.shields.io/crates/v/advise.svg
[version.hyper]: https://crates.io/crates/advise

<!-- Usage -->
[example]: ./examples/basic.rs
[log]:     https://docs.rs/log
