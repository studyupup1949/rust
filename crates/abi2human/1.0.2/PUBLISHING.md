# Publishing Guide for abi2human

## Prerequisites

1. **Crates.io Account**: Create an account at https://crates.io/
2. **API Token**: Generate a token at https://crates.io/me
3. **GitHub Repository**: Update the repository URLs in `Cargo.toml`

## Setup GitHub Secrets

Add your crates.io API token as a GitHub secret:
1. Go to your repository's Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `CARGO_REGISTRY_TOKEN`
4. Value: Your crates.io API token

## Local Publishing (Manual)

### 1. Test Build
```bash
cargo build --release
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

### 2. Dry Run
```bash
cargo publish --dry-run
```

### 3. Publish to crates.io
```bash
cargo publish
```

## Automated Publishing (GitHub Actions)

### 1. Update Version
Edit `Cargo.toml`:
```toml
version = "1.0.1"  # Increment version
```

### 2. Update CHANGELOG
Add your changes to `CHANGELOG.md`

### 3. Commit and Tag
```bash
git add .
git commit -m "chore: release v1.0.1"
git tag -a v1.0.1 -m "Release version 1.0.1"
git push origin main
git push origin v1.0.1
```

The GitHub Actions will automatically:
- Run tests
- Check formatting and linting
- Publish to crates.io
- Create GitHub release with binaries for:
  - Linux (x86_64)
  - macOS (x86_64 and ARM64)
  - Windows (x86_64)

## Version Management

Follow [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes

## Pre-release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] README.md is up to date
- [ ] CHANGELOG.md includes new changes
- [ ] Version bumped in Cargo.toml
- [ ] Repository URLs in Cargo.toml are correct
- [ ] Author information is correct

## Post-release

After successful publication:
1. Verify on https://crates.io/crates/abi2human
2. Test installation: `cargo install abi2human`
3. Verify binaries in GitHub release