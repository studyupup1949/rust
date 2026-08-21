# `act2pal`

Converts Adobe Color Tables to `.pal` files.

## Usage (CLI)

```bash
cargo install act2pal
act2pal -i input.act -o output.pal
```

## Usage (Rust)

```rust,ignore
use act2pal::Palette;

let act = std::fs::read("input.act")?;
let pal = Palette::from_act(&act)?;

std::fs::write("output.pal", pal.to_string())?;
```