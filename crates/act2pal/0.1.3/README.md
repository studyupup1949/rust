# act2pal

[![Crates.io](https://img.shields.io/crates/v/act2pal)](https://crates.io/crates/act2pal)
[![docs](https://docs.rs/act2pal/badge.svg)](https://docs.rs/act2pal)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/tipsypastels/act2pal/blob/main/LICENSE)

Converts Adobe Color Tables to `.pal` files.

## Usage (CLI)

```bash
cargo install act2pal
act2pal -i input.act -o output.pal
```

The `--assert-len` flag can be used to early exit if the number of colors in the palette is not as expected.

```bash
act2pal -i input.act -o output.pal --assert-len 256
```

## Usage (Rust)

```rust,ignore
use act2pal::Palette;

let act = std::fs::read("input.act")?;
let pal = Palette::from_act(&act)?;

std::fs::write("output.pal", pal.to_string())?;
```

The `Palette` type implements `Deref<Target = [Color]>` and `FromIterator<Color>` and can thus be manipulated freely.