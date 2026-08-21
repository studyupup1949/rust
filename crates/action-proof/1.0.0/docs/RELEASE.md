# Release Playbook

## Local Gates

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
cargo publish --dry-run --locked
```

## Self-Proof

```powershell
cargo run --locked -- --manifest action.yml --repo-root . --strict
```

If `--strict` fails only because a deliberate floating reference is present in this repository's own action wrapper, either pin that action to a SHA or document why the warning is accepted for the release.

## Tag And Publish

```powershell
git tag -a vX.Y.Z -m "action-proof vX.Y.Z"
git push origin vX.Y.Z
cargo publish --locked
git push origin main
```

## Post-Publish

```powershell
cargo install action-proof --version X.Y.Z --locked --force
action-proof --version
```

Create a GitHub Release and run the released-action consumer smoke workflow from `main`.

