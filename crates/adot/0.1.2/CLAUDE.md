# adot

A minimal dotfile manager written in Rust. Drop-in replacement for dotdrop.

You're my coding assistant, remember these preferences:

## Project Structure

- `src/` for source code, `tests/` for tests (never tests in `src/`!)
- Everything controlled through the `Makefile` — prefer `make` commands over raw cargo/shell commands
- Example references:
  - `~/workspaces/nvrr` — Makefile for brew formula and crate releases
  - `~/workspaces/white-dragon` — cross-compilation setup (macOS/Linux/Windows via zigbuild)
- Reference dotdrop config: `~/dotfiles/config.yaml`

## Rust Preferences

- Use `cargo add` to add packages (no version unless needed), NEVER patch Cargo.toml directly
- Early return always over nested ifs: check -> return if not true, check -> return if not true, do thing
- Prefer type hints, always include error handling
- Prefer small files over big 500-line monsters
- No `unwrap()` in non-test code — propagate errors properly

## Dependency Policy

- **As few dependencies as possible** — suggest dependencies before adding them
- I will either write them myself or fork them so they're pulled from my fork for full control
- Every dependency must be justified and approved before adding
- clap is from `git@github.com:Dimfred/clap.git` (forked)
- yaml-rust2 is from `git@github.com:Dimfred/yaml-rust2.git` (forked)
- Always use `cargo install --locked` — pins Cargo.lock versions, prevents silent resolution of malicious/broken patch versions of transitive deps

## Config Format

- YAML config, compatible with dotdrop's `config.yaml` format
- Top-level sections: `config` (ignored, defaults only), `dotfiles`, `profiles`, `variables`, `dynvariables`
- Deployment modes: copy, symlink (absolute, link_children), templates (simple regex replacement)
- Profiles support `include` for inheritance, per-profile `variables` and `dynvariables`
- Respect XDG directories (`$XDG_CONFIG_HOME`, etc.)

## Linting

- Before committing: run `make lint-fix`, then `make lint` if issues remain and fix them manually
- Never run unsafe fix commands

## Superpowers

- NEVER auto-invoke any `superpowers:*` skills on your own initiative
- Only invoke a superpowers skill when the user explicitly requests it

## Committing

- When committing don't diff everything, just commit with a message of what we did in the conversation
