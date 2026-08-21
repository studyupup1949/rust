# ACP - AI Context Protocol

> ⚠️ **Placeholder Release** - Full functionality coming soon.

Core library for the [AI Context Protocol](https://github.com/acp-protocol/acp-spec).

## What is ACP?

The AI Context Protocol (ACP) is an open standard for embedding machine-readable context in codebases for AI-assisted development.

## Features (Coming Soon)

- **Parsing** - Extract ACP annotations from source code
- **Indexing** - Build structured cache of codebase metadata
- **Querying** - Search and filter indexed symbols and files
- **Constraints** - Evaluate and enforce code constraints
- **Variables** - Expand token-efficient variable references

## Usage

```rust,ignore
use acp::{Cache, Config};

let config = Config::load(".acp.config.json")?;
let cache = Cache::build(&config)?;

// Query symbols
let auth_symbols = cache.query_domain("authentication")?;
```

## Related Crates

- [`acp-cli`](https://crates.io/crates/acp-cli) - Command-line interface

## Links

- [GitHub Repository](https://github.com/acp-protocol/acp-spec)
- [Specification](https://github.com/acp-protocol/acp-spec/tree/main/spec)

## License

MIT License - Copyright (c) 2024 ACP Protocol Contributors
