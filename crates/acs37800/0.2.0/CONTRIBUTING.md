# Contributing

Thanks for your interest in improving the ACS37800 driver! This guide explains how to get a local environment ready, how to run the required checks, and what we expect from pull requests.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable toolchain.
- [`just`](https://github.com/casey/just#installation) for running the project recipes.
- `rustup` targets installed for cross-checks:
  ```shell
  rustup target add aarch64-unknown-linux-gnu thumbv8m.main-none-eabihf
  ```
- Run once: `just install-supported-toolchains` to install every rustc release from the MSRV through the latest stable (plus the required cross targets). This keeps your local toolchains in sync with CI.

## Project overview

- I²C support is implemented; SPI is in-progress.
- Feature flags:
  - `i2c` (required for most functionality)
  - `async` (enables async traits/tests)
  - `std`, `defmt` (optional platform integrations)
- Unit tests rely on `tokio` and `embedded-hal-mock`, which only build on host targets.

## Recommended workflow

1. Fork and clone the repository.
2. Create a feature branch (`git switch -c my-feature`).
3. Implement your change with appropriate tests/docs.
4. Run formatting and the full validation suite (see below).
5. Push and open a pull request against `main`.

## Formatting & linting

- Format Rust code with `cargo fmt` before committing.
- If you add new public APIs, update or generate docs where appropriate.

## Running tests

Host-side unit tests must pass for both sync and async configurations:

```shell
just test                 # latest stable toolchain
just test-msrv            # MSRV-only run (installs the toolchain if missing)
just test-all-toolchains  # runs tests on every supported toolchain (MSRV -> stable)
```

## Running checks

Cross-target checks ensure embedded builds remain healthy:

```shell
just check                 # latest stable toolchain
just check-msrv            # MSRV-only run (installs the toolchain if missing)
just check-all-toolchains  # runs checks on every supported toolchain (MSRV -> stable)
```

This recipe expands to the same matrix executed by CI:

- `cargo check` on AArch64 Linux (`std + i2c`)
- `cargo check` on Thumbv8-M for `i2c` and `async i2c`
- Example builds for Raspberry Pi (Linux) and RP2350 (Thumbv8-M, async)

> **Tip:** `just install-supported-toolchains` installs all required compilers and cross targets in one go, so `just check` or `just test` will work on a fresh machine without manual setup.

## Pull request checklist

- [ ] All code is formatted and warnings are resolved.
- [ ] `just test` passes locally.
- [ ] `just check` passes locally (or at least the portions touching your change).
- [ ] New features include tests and documentation.
- [ ] Commit messages and PR description explain the motivation and behavior change.

CI will run the same host tests plus the cross-platform checks; keeping them green locally speeds up reviews.

## Questions?

Feel free to open a discussion or issue if anything in this guide is unclear. We appreciate improvements to the documentation itself, too!
