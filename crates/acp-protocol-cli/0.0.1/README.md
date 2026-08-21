# ACP CLI

> ⚠️ **Placeholder Release** - Full functionality coming soon.

Command-line interface for the [AI Context Protocol](https://github.com/acp-protocol/acp-spec).

## What is ACP?

The AI Context Protocol (ACP) is an open standard for embedding machine-readable context in codebases for AI-assisted development.

```javascript
/**
 * @acp-protocol:lock restricted
 * @acp-protocol:domain authentication
 * @acp-protocol:summary "Validates JWT tokens - security critical"
 */
function validateToken(token) {
  // AI tools will respect the constraints above
}
```

## Features (Coming Soon)

- **`acp init`** - Initialize ACP in your project
- **`acp index`** - Index your codebase to `.acp.cache.json`
- **`acp query`** - Query symbols, files, and domains
- **`acp constraints`** - View file constraints
- **`acp vars`** - Generate token-efficient variables

## Installation

```bash
cargo install acp-protocol-cli
```

## Links

- [GitHub Repository](https://github.com/acp-protocol/acp-spec)
- [Specification](https://github.com/acp-protocol/acp-spec/tree/main/spec)
- [Documentation](https://github.com/acp-protocol/acp-spec#readme)

## License

MIT License - Copyright (c) 2024 ACP Protocol Contributors
