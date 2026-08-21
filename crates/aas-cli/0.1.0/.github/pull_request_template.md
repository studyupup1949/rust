## Summary

<!-- One sentence: what does this PR accomplish? -->

## Changes

<!-- Bullet list of what changed -->
- 
- 

## Why

<!-- Why is this change needed? What problem does it solve? -->

## Testing

<!-- How was this tested? -->
- [ ] Unit tests added/updated
- [ ] Integration test added
- [ ] Manual testing: _____________

## Type

- [ ] Bug fix
- [ ] New feature
- [ ] Performance improvement
- [ ] Refactor
- [ ] Documentation

## Impact

- [ ] Agent behavior (learning, execution, etc.)
- [ ] LLM integration (new provider, routing change)
- [ ] Memory/storage (new patterns, learning data)
- [ ] CLI (new command, config change)
- [ ] Configuration (new settings)

## Breaking Changes

<!-- If this breaks existing config or behavior, describe migration path -->

## Checklist

- [ ] Tests pass: `cargo test --release`
- [ ] Builds clean: `cargo build --release`
- [ ] Updated README if needed
- [ ] Added CHANGELOG entry if user-facing
- [ ] Code follows project style (see CONTRIBUTING.md)

## Related Issues

Closes #

---

**Notes for reviewers**: 
- If this touches learning cache, verify cache hit behavior with `aas logs follow`
- If this touches agent execution, test with `aas run --dry-run` first
- If this adds an integration, include setup instructions
