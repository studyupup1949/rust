# abi2human

[![Publish to crates.io](https://github.com/devdotbo/abi2human-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/devdotbo/abi2human-rs/actions/workflows/publish.yml)

A zero-dependency Rust implementation for converting Ethereum ABI to human-readable format.

## Features

- 🚀 **Zero Dependencies**: Pure Rust implementation with no external dependencies
- 📝 **Human-Readable Output**: Convert complex ABI JSON to readable function signatures
- 🎯 **Multiple Output Formats**: JSON array, raw text, or compact JSON
- 📁 **Batch Processing**: Convert single files or entire directories
- 🧪 **Well Tested**: Comprehensive test suite included
- ⚡ **Fast**: Optimized Rust performance

## Installation

### From crates.io (Recommended)

```bash
cargo install abi2human
```

### Build from Source

```bash
cargo build --release
```

The binary will be available at `./target/release/abi2human`

## Usage

### Quick ABI Inspection

```bash
# Output to stdout in JSON format
abi2human contract.json -o

# Raw text format (one function per line)
abi2human contract.json -or

# Compact JSON (no pretty printing)
abi2human contract.json -o --no-pretty
```

### File Conversion

```bash
# Convert and save to a new file
abi2human input.json output.json

# Convert with custom suffix
abi2human input.json -s ".readable"
```

### Batch Directory Processing

```bash
# Convert all JSON files in a directory
abi2human ./abis/ -d ./readable/

# Filter with pattern
abi2human ./abis/ -d ./readable/ -p "*.abi.json"
```

### Command Line Options

```
OPTIONS:
  -o, --stdout     Output to stdout
  -r, --raw        Output raw text format instead of JSON
  -h, --help       Show help message
  -v, --version    Show version
  -q, --quiet      Suppress non-output messages
  -d, --dir        Process directory
  -p, --pattern    Glob pattern for filtering files
  -s, --suffix     Custom suffix for output files (default: ".readable")
  --no-pretty      Disable pretty-printing
```

## Examples

### ERC20 Token ABI

Input:
```json
[
  {
    "type": "function",
    "name": "transfer",
    "inputs": [
      {"name": "to", "type": "address"},
      {"name": "amount", "type": "uint256"}
    ],
    "outputs": [{"type": "bool"}],
    "stateMutability": "nonpayable"
  }
]
```

Output:
```
function transfer(address to, uint256 amount) returns (bool)
```

### Event Example

Input:
```json
{
  "type": "event",
  "name": "Transfer",
  "inputs": [
    {"name": "from", "type": "address", "indexed": true},
    {"name": "to", "type": "address", "indexed": true},
    {"name": "value", "type": "uint256", "indexed": false}
  ]
}
```

Output:
```
event Transfer(address indexed from, address indexed to, uint256 value)
```

## Supported ABI Types

- ✅ Functions (view, pure, payable, nonpayable)
- ✅ Events (with indexed parameters)
- ✅ Constructors
- ✅ Fallback functions
- ✅ Receive functions

## Development

### Running Tests

```bash
cargo test
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

## Architecture

The project is organized into several modules:

- `abi.rs` - ABI data structures and formatting
- `json_parser.rs` - Custom JSON parser implementation
- `converter.rs` - Core conversion logic
- `file_ops.rs` - File and directory operations
- `main.rs` - CLI entry point and argument parsing
- `tests.rs` - Unit tests

## Why Zero Dependencies?

This implementation uses no external crates, providing:

- **Security**: No supply chain vulnerabilities from dependencies
- **Simplicity**: Easy to audit and understand
- **Portability**: Works anywhere Rust compiles
- **Stability**: No breaking changes from dependency updates

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.