# aaai-cli

Command-line interface for **aaai** (audit for asset integrity).

```sh
# Snap a diff into an audit definition template
aaai snap --left ./before --right ./after --out audit.yaml

# Run the audit
aaai audit --left ./before --right ./after --config audit.yaml

# Show a colour-coded dashboard
aaai dashboard --left ./before --right ./after --config audit.yaml
```

## Commands

`audit` · `snap` · `report` · `check` · `lint` · `diff` · `merge`  
`history` · `export` · `dashboard` · `watch` · `init` · `config`  
`version` · `completions`

Exit codes: `0` PASSED · `1` FAILED · `2` PENDING · `3` ERROR · `4` CONFIG_ERROR

## Full Documentation

- [CLI Reference](https://github.com/nabbisen/aaai/blob/main/docs/src/cli.md)
- [Getting Started](https://github.com/nabbisen/aaai/blob/main/docs/src/getting-started.md)
- [CI/CD Integration](https://github.com/nabbisen/aaai/blob/main/docs/src/ci-integration.md)
