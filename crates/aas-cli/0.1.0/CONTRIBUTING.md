# Contributing to AAS

## Code of Conduct

Be respectful, inclusive, and professional.

## Getting Started

1. Fork the repo
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make changes
5. Test: `cargo test --release`
6. Commit: `git commit -am "Clear description of changes"`
7. Push: `git push origin feature/your-feature`
8. Open a PR

## Development Setup

```bash
# Clone
git clone https://github.com/yourusername/aas.git
cd aas

# Build
cargo build

# Test
cargo test

# Run with tracing
RUST_LOG=debug cargo run --release -- run

# Format
cargo fmt

# Lint
cargo clippy --all-targets --all-features
```

## Code Style

- **Formatting**: `cargo fmt` (enforced in CI)
- **Linting**: `cargo clippy` (fix warnings)
- **Comments**: Minimal. Only explain *why*, not what. Code should be self-documenting.
- **Naming**: Clear and descriptive. No abbreviations unless universally understood (HTTP, LLM, RSI).
- **Error handling**: Use `?` operator, propagate errors, don't swallow them.
- **Logging**: Use `tracing` crate. Log at appropriate levels (debug, info, warn, error).

## Git Commit Messages

```
[AREA] Short imperative sentence

Longer explanation if needed. Explain *why*, not what.

Related: #123
```

**Examples**:
- `[learning] Cache successful solutions automatically`
- `[cli] Add 'aas connect' command for integrations`
- `[rsi] Adjust confidence thresholds based on success rate`
- `[docs] Update README with learning examples`

## Areas (Use in Commit Messages)

- `[learning]` — cache, issue signatures, memory
- `[execution]` — running commands, verification, rollback
- `[rsi]` — self-improvement, threshold adjustment
- `[llm]` — LLM providers, routing, integrations
- `[agent]` — agent trait, run_cycle, detect/analyze/plan/execute
- `[cli]` — command-line interface, daemon, commands
- `[config]` — configuration files, settings
- `[docs]` — documentation, README, guides
- `[test]` — tests and test infrastructure
- `[ci]` — CI/CD, GitHub Actions

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_signature() {
        // Test the small unit
    }
}
```

### Integration Tests

```bash
# Create tests/integration/my_test.rs
#[tokio::test]
async fn test_learning_cache() {
    // Test across multiple components
}
```

### Running Tests

```bash
cargo test                    # All tests
cargo test --release          # Release mode (faster)
cargo test -- --nocapture     # Show println! output
cargo test --package aas -- --test-threads=1  # Single-threaded
```

### Benchmarks

```bash
cargo bench
```

## What to Work On

### Good First Issues

- Adding tests for existing functionality
- Improving error messages
- Documentation improvements
- Small bug fixes

### Medium Issues

- New agent domain (Health, Metrics, etc.)
- New CLI command
- Config improvements

### Large Issues

- New learning mechanism (failure pattern learning)
- New integration (Slack, Discord)
- Cross-agent discovery
- Meta-improvement (agents improving agents)

## Learning & Caching Logic

If you're working on learning:

1. **Cache lookup** happens in `run_cycle` before `analyze()`
2. **Cache storage** happens after `verify()` succeeds
3. **Issue signature** must be deterministic (same issue → same signature)
4. **Cache is in-memory** (HashMap, not DB) for speed
5. **Test**: Run twice with same issue, verify logs show "reusing learned solution"

## Integration Development

### Adding a New LLM Provider

1. Create `src/llm/providers/my_provider.rs`
2. Implement `LLMProvider` trait
3. Add to `LLMRouter` in `src/llm/router.rs`
4. Add config in `src/config/settings.rs`
5. Wire into `main.rs` initialization
6. Test with `aas run --dry-run`

### Adding a New Integration

1. Create `src/integrations/my_service.rs`
2. Implement connection/API logic
3. Add `aas connect my-service` command in `src/cli/integrations.rs`
4. Add config in `src/config/settings.rs`
5. Document in README.md

## Documentation

### README

Update if:
- New CLI command
- New agent
- New integration
- Changed configuration

### Architecture Docs

- `docs/architecture.md` — System design
- `docs/learning.md` — Learning system deep dive
- `docs/integrations.md` — Adding new providers

### Code Comments

Use sparingly. Only explain *why*, not what:

```rust
// Good: explains the decision
// Cache only after verify() to avoid storing broken solutions
ctx.learned_solutions.insert(sig, action);

// Bad: just repeats code
// Insert into learned_solutions
ctx.learned_solutions.insert(sig, action);
```

## PR Review Process

1. **Automated**: CI runs tests, formatting, linting
2. **Code Review**: At least one approval required
3. **Merge**: Squash and merge (keep history clean)

### What Reviewers Look For

- Does it solve the stated problem?
- Is it the simplest solution?
- Are there any edge cases?
- Is it tested?
- Does it fit the architecture?
- Is it documented?
- Are there any performance concerns?

## Performance Considerations

AAS runs agents in loops. Performance matters:

- **Cache lookups**: Should be `O(1)` (HashMap)
- **Issue detection**: Should be fast (no deep FS traverses)
- **Verification**: Should be thorough but not slow (re-detect, not deep analysis)
- **Learning**: Should be instant (just HashMap insert)

Profile with:
```bash
cargo bench
RUST_LOG=aas=debug aas run  # See cycle duration
```

## Debugging

### With Logging

```bash
RUST_LOG=debug aas run                # Everything
RUST_LOG=aas::swarm=trace aas run    # Just swarm
RUST_LOG=aas::memory=debug aas run   # Just memory
```

### With REPL

Debug DB state:
```bash
sqlite3 ~/.local/share/aas/aas.db
> SELECT * FROM cycle_performance ORDER BY timestamp DESC LIMIT 10;
```

### With Dry Run

Test without executing:
```bash
aas run --dry-run
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit: `[release] v0.2.0`
4. Tag: `git tag v0.2.0`
5. Push: `git push origin main --tags`
6. GitHub Actions builds and publishes

## Questions?

- Check existing issues
- Ask in PR comments
- Open a discussion

---

Thank you for contributing! 🚀
