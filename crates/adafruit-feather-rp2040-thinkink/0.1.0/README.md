# [adafruit-feather-rp2040-thinkink] - Board Support for the [Adafruit Feather RP2040 ThinkInk]

You should include this crate if you are writing code that you want to run on
an [Adafruit Feather RP2040 ThinkInk] - a Feather form-factor RP2040 board from Adafruit, that's designed to make it easy to add almost any common e-Ink/e-Paper display.

This crate includes the [rp2040-hal], but also configures each pin of the
RP2040 chip according to how it is connected up on the Feather.

[Adafruit Feather RP2040 ThinkInk]: https://learn.adafruit.com/adafruit-rp2040-feather-thinkink/overview
[adafruit-feather-rp2040-thinkink]: https://github.com/brrastak/adafruit-feather-rp2040-thinkink.git
[rp2040-hal]: https://github.com/rp-rs/rp-hal/tree/main/rp2040-hal

## Examples

### General Instructions

To compile an example, clone the _adafruit-feather-rp2040-thinkink_ repository. Replace `hello` with the example name (without the `.rs` extension), then run:

```console
cargo build --release --example hello
```

To flash the example using the RP2040 UF2 bootloader, install `elf2uf2-rs`:

```console
cargo install elf2uf2-rs
```

Then run:

```console
cargo run --release --example hello
```

A `.cargo/config.toml` file is included.

### [Hello](./examples/hello.rs)

Draws a text message on an e-paper display.
