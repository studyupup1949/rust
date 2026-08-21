# add-osc-8-hyperlink

A Rust port of [sentriz/add-osc-8-hyperlink](https://github.com/sentriz/add-osc-8-hyperlink).

Add clickable OSC 8 hyperlinks to file paths in terminal output while preserving ANSI color codes.

## Features

- Converts file paths to clickable `file://` hyperlinks
- Preserves ANSI color codes (works with colored output from `git status`, `rg`, etc.)
- Supports absolute paths, relative paths, and `~/` home directory expansion
- Works with any terminal that supports OSC 8 hyperlinks (Ghostty, iTerm2, WezTerm, etc.)

## Installation

From crates.io:
```bash
cargo install add-osc-8-hyperlink
```

From GitHub:
```bash
cargo install --git https://github.com/konradko/add-osc-8-hyperlink-rs
```

Or build from source:

```bash
cargo build --release
cp target/release/add-osc-8-hyperlink ~/.local/bin/
```

## Usage

Pipe any command output through the tool:

```bash
# Git status with colors and clickable file paths
git -c color.status=always status | add-osc-8-hyperlink

# Ripgrep with clickable results
rg --color=always pattern | add-osc-8-hyperlink

# Any command with file paths
ls -la | add-osc-8-hyperlink
```

### Shell Integration

Add to your `.bashrc` or `.bash_profile`:

```bash
function g { git -c color.status=always status "$@" | add-osc-8-hyperlink; }
```

Or for fish shell:

```fish
function git
    if isatty stdout; and contains -- $argv[1] diff status log
        command git -c color.status=always -c color.ui=always $argv | add-osc-8-hyperlink
        return
    end
    command git $argv
end
```

## How it Works

The tool scans each line for file paths matching:
- Absolute paths starting with common prefixes (`/tmp`, `/home`, `/usr`, etc.)
- Relative paths matching entries in the current directory
- Home directory paths starting with `~/`

Paths are wrapped in OSC 8 escape sequences:
```
\e]8;;file://hostname/path\a<visible text>\e]8;;\a
```

ANSI color codes (`\e[31m`, etc.) are explicitly excluded from path matching, so colored output passes through unchanged.

## Performance

Benchmarked against the [Go implementation](https://github.com/sentriz/add-osc-8-hyperlink):

| Input Size | Rust | Go | Speedup |
|------------|------|-----|---------|
| 5,000 lines | 15.7 ms | 37.6 ms | 2.4x faster |
| 50,000 lines | 103.3 ms | 306.1 ms | 3.0x faster |

Binary size: 1.6 MB (Rust) vs 2.8 MB (Go)

## License

MIT
