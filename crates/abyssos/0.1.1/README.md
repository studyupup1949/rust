# Abyssos

> *Greek: bottomless depth* — a very dark navy/charcoal terminal theme with a teal/cyan hero accent.

Inspired by the Perplexity aesthetic. Named via a 6-model consilium (5/6 converged).

## Palette

| Role | Hex |
|------|-----|
| Background | `#0D0F10` |
| Foreground | `#E2E4E6` |
| Cursor | `#20C5C0` |
| Selection bg | `#1A3535` |
| Selection fg | `#E2E4E6` |
| 0 black | `#1A1C1E` |
| 1 red | `#E06B6B` |
| 2 green | `#1CB8A8` |
| 3 yellow | `#D4A85A` |
| 4 blue | `#5B8DD9` |
| 5 magenta | `#9B7FD4` |
| 6 cyan (hero) | `#20C5C0` |
| 7 white | `#C8CACC` |
| 8 bright black | `#2A2C2E` |
| 9 bright red | `#EA7B7B` |
| 10 bright green | `#2CC8B8` |
| 11 bright yellow | `#E4B86A` |
| 12 bright blue | `#6B9DE9` |
| 13 bright magenta | `#AB8FE4` |
| 14 bright cyan | `#30D5D0` |
| 15 bright white | `#D8DADC` |

## Installation

### Ghostty

```sh
cp themes/abyssos ~/.config/ghostty/themes/abyssos
```

Then add to `~/.config/ghostty/config`:

```ini
theme = abyssos
```

Verify: `ghostty +list-themes | grep -i abyssos`

### Blink Shell

In Blink: **Settings → Appearance → New Theme**, paste the raw URL:

```
https://raw.githubusercontent.com/terry-li-hm/abyssos/main/themes/abyssos.js
```

## Python

```sh
pip install abyssos
```

```python
import abyssos
print(abyssos.PALETTE["cyan"])  # '#20C5C0'
```

## Rust

```toml
[dependencies]
abyssos = "0.1"
```

```rust
use abyssos::CYAN;
println!("{}", CYAN);  // "#20C5C0"
```

### tmux

```sh
# Add to ~/.tmux.conf:
source-file /path/to/abyssos/themes/abyssos.tmux.conf
```

Or copy the relevant lines from `themes/abyssos.tmux.conf`.

> Note: uses 256-colour numbers, not hex — hex breaks mouse clicks in some terminals.

## License

MIT
