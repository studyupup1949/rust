# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2026-04-03

### ⚡ Upgraded to ADK-Rust 0.5.0

Major upgrade from adk-rust 0.3.2 to 0.5.0, bringing significant new capabilities.

### Added

- **14 LLM Providers** — Gemini, OpenAI, Anthropic, DeepSeek, Groq, Ollama, Fireworks, Together, Mistral, Perplexity, Cerebras, SambaNova, Amazon Bedrock, Azure AI
- **Shared Model Factory** (`src/model_factory.rs`) — Centralized model creation for all providers, eliminating code duplication across agents
- **Tool Resilience** — All agents now use:
  - `tool_timeout(5 min)` to prevent hung tool calls
  - `default_retry_budget(2 retries)` for automatic transient failure recovery
  - `circuit_breaker_threshold(5)` to disable tools after consecutive failures
- **Context Compaction** — Loop agent uses `LlmEventSummarizer` to automatically summarize older conversation events, keeping context bounded during long sessions
- **Per-Agent Generation Config** — PRD and Architect agents now pass `temperature` and `max_output_tokens` directly to the LLM builder
- **DeepSeek auto-selection** — Automatically uses `reasoner` or `chat` client based on model name
- **Anthropic thinking mode** — Automatically enables `with_thinking()` when `thinking_enabled` is set in config
- **`schemars` dependency** — For future tool parameter schema generation

### Changed

- **`GOOGLE_API_KEY`** is now the primary env var for Gemini (with `GEMINI_API_KEY` fallback)
- **`SUPPORTED_PROVIDERS`** expanded from 4 to 14 providers
- **OpenTelemetry** upgraded from 0.21 to 0.31 (metrics API: `.init()` → `.build()`)
- **`AdkError::Tool()`** → `AdkError::tool()` (adk-core 0.5.0 API change)
- **`RunnerConfig`** — Added new fields: `context_cache_config`, `cache_capable`, `request_context`, `cancellation_token`
- **`Runner.run()`** — Now takes typed `UserId`/`SessionId` instead of `String`
- **`Content::new("user").with_text()`** builder pattern replaces manual struct construction
- **Default models updated** — PRD: `gemini-3.1-pro-preview`, Architect: `gemini-3-pro-preview`, Loop: `gemini-2.5-flash`

### Removed

- Duplicated `create_model_from_config` functions (was copy-pasted in 4 files)

## [0.3.2] - 2026-02-21

### Added

- Initial public release
- Three-agent pipeline: PRD → Architect → Loop
- Interactive chat mode with session persistence
- Multi-language support: Rust, Python, TypeScript, Go, Java
- OpenTelemetry integration
- Support for Gemini, OpenAI, Anthropic, Ollama providers
