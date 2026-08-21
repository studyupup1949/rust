# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Test
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Lint & Format
```bash
# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Run clippy for linting
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

### Run
```bash
# Run debug version
cargo run -- [args]

# Run release version
cargo run --release -- [args]

# Or use the compiled binary
./target/release/abi2human [args]
```

## Architecture

This is a zero-dependency Rust CLI tool that converts Ethereum ABI JSON to human-readable format. The codebase is modular with clear separation of concerns:

### Core Modules

- **`abi.rs`**: Defines ABI data structures (`AbiItem`, `AbiInput`, `AbiOutput`) and implements their Display traits for human-readable formatting
- **`json_parser.rs`**: Custom JSON parser that handles ABI parsing without external dependencies
- **`converter.rs`**: Orchestrates the conversion process - parses ABI content and formats output
- **`file_ops.rs`**: Handles all file I/O operations including single file, directory, and stdin/stdout processing
- **`main.rs`**: CLI entry point with argument parsing and command routing
- **`tests.rs`**: Comprehensive test suite

### Key Design Patterns

1. **Zero Dependencies**: All JSON parsing and formatting is implemented from scratch to avoid supply chain risks
2. **Stream Processing**: The tool can process stdin to stdout for pipeline integration
3. **Batch Operations**: Supports converting entire directories with pattern matching
4. **Multiple Output Formats**: JSON array, raw text, or compact JSON

### Data Flow

1. Input (file/directory/stdin) → 
2. JSON Parser (converts to ABI structures) → 
3. Converter (formats to human-readable) → 
4. Output (file/stdout with chosen format)

The tool is designed to be fast, secure, and easily auditable with no external dependencies.