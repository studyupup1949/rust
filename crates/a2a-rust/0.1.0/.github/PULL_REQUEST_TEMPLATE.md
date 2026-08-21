## Summary

<!-- Brief description of what this PR does -->

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation (changes to documentation only)
- [ ] Refactor (code change that neither fixes a bug nor adds a feature)
- [ ] Test (adding or updating tests)

## Related Issues

<!-- Link to related issues: Fixes #123, Closes #456 -->

## Changes Made

<!-- Detailed list of changes -->

- 
- 

## Protocol Compliance

<!-- If this touches types or serialization, confirm protocol compliance -->

- [ ] Changes align with A2A v1.0 spec (tag `v1.0.0`)
- [ ] Serde serialization matches proto3 JSON mapping
- [ ] N/A — no protocol-related changes

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Serialization round-trip tests verified
- [ ] Manual testing performed

<details>
<summary>Test output</summary>

```
cargo test output here
```

</details>

## Checklist

- [ ] My code follows the project's code style (see AGENTS.md)
- [ ] I have run `cargo fmt --all`
- [ ] I have run `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] I have run `cargo test --all-features`
- [ ] I have added rustdoc comments for new public items
- [ ] My commit messages follow [conventional commits](https://www.conventionalcommits.org/)
