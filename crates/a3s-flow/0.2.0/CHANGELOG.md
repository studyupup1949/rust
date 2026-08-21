# Changelog

All notable changes to a3s-flow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-09

### Changed

- **BREAKING:** Extracted MCP node to external crate `a3s-flow-mcp` ([#8c7fe09](https://github.com/A3S-Lab/Flow/commit/8c7fe09))
  - MCP is no longer included in `NodeRegistry::with_defaults()`
  - Users must explicitly register MCP via `registry.register(Arc::new(McpNode))`
  - Reduces core engine size by 710 lines
  - Follows "Minimal Core + External Extensions" architecture pattern

### Removed

- `nodes::mcp` module (moved to `a3s-flow-mcp` crate)
- MCP-related dependencies from core (reqwest SSE features, futures-util for MCP)

### Added

- Architecture documentation: `DESIGN_MCP_AS_TOOL.md`
- Clear separation between core nodes and external tool nodes

### Migration Guide

**Before (v0.1.0):**
```rust
use a3s_flow::NodeRegistry;

let registry = NodeRegistry::with_defaults();
// MCP was automatically included
```

**After (v0.2.0):**
```rust
use a3s_flow::NodeRegistry;
use a3s_flow_mcp::McpNode;
use std::sync::Arc;

let mut registry = NodeRegistry::with_defaults();
registry.register(Arc::new(McpNode));  // Explicitly register MCP
```

## [0.1.0] - 2026-03-08

### Added

- Initial release of a3s-flow workflow engine
- 16 built-in Dify-compatible nodes:
  - `noop`, `start`, `end` — flow control
  - `http-request` — HTTP client
  - `if-else` — conditional routing
  - `template-transform` — Jinja2 templates
  - `variable-aggregator` — fan-in merge
  - `code` — sandboxed Rhai scripts
  - `iteration` — array loops
  - `sub-flow` — nested workflows
  - `llm` — OpenAI-compatible chat
  - `question-classifier` — LLM routing
  - `assign` — variable assignment
  - `parameter-extractor` — LLM parameter extraction
  - `loop` — while loops
  - `list-operator` — array operations
  - `mcp` — Model Context Protocol integration
- `FlowEngine` lifecycle API: start, pause, resume, terminate
- JSON DAG definition format
- Wave-based concurrent execution
- Extensible `Node` trait for custom nodes
- `NodeRegistry` for node type management
- Comprehensive test coverage

[0.2.0]: https://github.com/A3S-Lab/Flow/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/A3S-Lab/Flow/releases/tag/v0.1.0
