# action-proof

`action-proof` is a release preflight for GitHub Action repositories. It validates `action.yml` / `action.yaml`, checks common composite-action release mistakes, and emits a text, JSON, or Markdown receipt.

It exists because a normal Rust/Node/Python test suite can pass while GitHub still refuses to load an action manifest. `action-proof` catches that class of failure before a tag goes out.

## Install

```powershell
cargo install action-proof --locked
```

## Use

Run in an action repository:

```powershell
action-proof
```

Write a Markdown receipt:

```powershell
action-proof --format markdown --output action-proof.md
```

Check an explicit manifest:

```powershell
action-proof --manifest action.yml --repo-root .
```

Treat warnings as release-blocking:

```powershell
action-proof --strict
```

## GitHub Actions

```yaml
jobs:
  action-proof:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5
      - uses: wildmason/action-proof@v1
        with:
          manifest: action.yml
          format: markdown
          output: action-proof.md
          strict: "true"
```

## What It Checks

- Discovers exactly one `action.yml` or `action.yaml`.
- Parses the manifest as YAML and reports parser errors.
- Requires `name`, `description`, and `runs`.
- Rejects obsolete JavaScript runtimes `node12` and `node16`.
- Validates composite steps contain exactly one of `run` or `uses`.
- Requires every composite `run` step to declare `shell`.
- Warns on obvious download-and-execute shell patterns such as `curl ... | bash`.
- Warns on remote `uses:` references that are not pinned to a full 40-character SHA.
- Validates input/output names and input descriptions.
- Checks Marketplace branding, README, license, and presence of a released-action consumer smoke workflow.

## Output

Text is the default:

```text
action-proof 1.0.0 for action.yml
summary: 20 passed, 1 warned, 0 failed, 1 skipped
[PASS] pass    manifest.yaml
  manifest YAML parses
```

JSON and Markdown are available:

```powershell
action-proof --format json --output receipt.json
action-proof --format markdown --output receipt.md
```

## Exit Codes

`action-proof` exits `0` when there are no failed checks. Warnings do not fail the run unless `--strict` is passed.

## Limits

`action-proof` is a manifest and wrapper preflight. It does not execute the action, emulate GitHub Actions, verify all expression syntax, or prove that a third-party action is safe. Pair it with a real consumer workflow that uses the released action tag.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

