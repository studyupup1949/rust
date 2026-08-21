# Rules

`action-proof` emits check IDs that are intended to stay stable within a major version.

## Manifest Discovery

- `manifest.discover`: an explicit manifest exists, or exactly one of `action.yml` / `action.yaml` exists under `--repo-root`.
- `manifest.read`: the manifest file can be read.
- `manifest.yaml`: the manifest parses as YAML.
- `manifest.root`: the manifest root is a mapping.

## Metadata

- `metadata.fields`: top-level metadata fields are known GitHub Action metadata fields.
- `metadata.name`: `name` exists and is non-empty.
- `metadata.description`: `description` exists and is non-empty.
- `inputs.names`: input names use GitHub-compatible characters.
- `inputs.descriptions`: every input has a non-empty description.
- `inputs.required`: every `required` flag is a boolean.
- `outputs.names`: output names use GitHub-compatible characters.
- `outputs.descriptions`: every output has a non-empty description. This is a warning, not a failure.
- `branding`: Marketplace branding includes icon and color.

## Runs

- `runs.using`: accepts `composite`, `docker`, `node20`, and `node24`. `node12` and `node16` fail.
- `runs.steps`: composite actions have a non-empty `runs.steps` list.
- `runs.steps.shape`: every composite step is a mapping with `run` or `uses`.
- `runs.steps.exclusive`: no composite step has both `run` and `uses`.
- `runs.steps.shell`: every composite `run` step declares `shell`.
- `runs.steps.shell_risk`: warns on obvious download-and-execute patterns.
- `runs.steps.uses_pinning`: warns on remote `uses:` references that are not full-SHA pinned.
- `runs.main`: JavaScript actions define `main`.
- `runs.image`: Docker actions define `image`.

## Repository Readiness

- `repo.readme`: README exists.
- `repo.license`: a license file exists.
- `repo.consumer_smoke`: a workflow appears to consume a released action tag.

## Strict Mode

`--strict` converts warnings into release-blocking failures at the summary/exit-code level. The individual check status remains `warn` in the receipt so the original severity is not lost.

