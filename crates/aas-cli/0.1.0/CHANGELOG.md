# Changelog

All notable changes to AAS are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- MVP Learning System: Agents cache successful solutions and reuse them
  - `learned_solutions` HashMap in AgentContext
  - Cache hit before LLM analysis (0 LLM calls for repeat issues)
  - Cache storage after verified success
  - Observable in logs: "learned solution for X", "reusing learned solution for X"
- Daemon mode for background execution
  - `aas daemon start` — start background service
  - `aas daemon stop` — stop daemon
  - `aas daemon status` — check if running
  - `aas daemon logs` — view daemon output
- Integration connector CLI
  - `aas connect claude-code` — enable Claude Code for file edits
  - `aas connect openclaw` — enable OpenClaw for external tasks
  - `aas disconnect <name>` — disable integration
  - `aas integrations` — list all available integrations
- GitHub repository scaffolding
  - Pull request template
  - Issue templates (bug report, feature request)
  - CONTRIBUTING.md guide
  - .gitignore
- Duration support for `aas run`
  - `aas run --duration 5m` — run for 5 minutes then exit
  - `aas run --duration 1h` — run for 1 hour
- Dry-run mode (planned)
  - `aas run --dry-run` — detect issues but don't execute

### Changed
- Simplified src/memory/learning.rs
  - Removed boilerplate LearnedSolution struct
  - Removed success_count/failure_count tracking
  - Single issue_signature() function (20 lines, down from 90)
- Updated src/swarm/agent.rs
  - Added learned_solutions HashMap to AgentContext
  - Cache check before LLM pipeline
  - Caching on verified success
  - ~40 new lines of learning logic

### Removed
- src/execution/staging.rs (unnecessary StagingEnvironment)
  - Theater if verification works correctly
  - Simplification per Ponytail principle

## [0.1.0] - 2026-06-21

### Initial Release
- Multi-domain agent framework (Repository, Logs, Health, Metrics)
- LLM integration (Claude API, Hermes, Fallback)
- SQLite memory store
- Pattern engine for issue detection
- Prediction engine for trend analysis
- RSI engine for self-improvement
- Event-driven coordination
- CLI interface
- Interactive dashboard (TUI)
