# `act2pal`

Converts Adobe Colour Tables to `.pal` files.

## Usage (CLI)

```bash
cargo install act2pal
act2pal -i input.act -o output.pal
```

## Usage (Rust)

```rust
use act2pal::Colors;

let bytes = std::fs::read("input.act")?;
let colors = Colors::from_act(&bytes)?;
let pal = colors.to_pal_string()?;

std::fs::write("output.pal", pal)?;
```