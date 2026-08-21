# aaai-core

Core engine library for **aaai** (audit for asset integrity).

This crate provides:
- **Folder diff** — parallel directory comparison (rayon-based)
- **Audit engine** — match diffs against an audit definition with content strategies
- **Report generation** — Markdown, JSON, HTML, SARIF v2.1.0
- **Secret masking** — regex-based redaction of API keys, passwords, tokens
- **History store** — append-only audit run log (`~/.aaai/history.jsonl`)
- **Project config** — `.aaai.yaml` auto-discovery and defaults

For the full documentation, examples, and CLI/GUI guides see the
[project README](https://github.com/nabbisen/aaai#readme) and
the [documentation](https://github.com/nabbisen/aaai/tree/main/docs/src).
