# Guide to Contributing to actm

Development for this project happens on the
[~thatonelutenist/actm@lists.sr.ht](https://lists.sr.ht/~thatonelutenist/actm) mailing list, please
direct all patch sets and other development related email to it:
<~thatonelutenist/actm@lists.sr.ht>.

## Communicating With The Developers

At the moment, our primary means of communication is our mailing list, please feel free to send us
an email at <~thatonelutenist/actm@lists.sr.ht>, and subscribe to our announcements at
[~thatonelutenist/actm@lists.sr.ht](https://lists.sr.ht/~thatonelutenist/actm).

Also see our project hub at https://sr.ht/~thatonelutenist/actm.

## Building and Tooling

### Nix

For convience of development, a [nix](https://nixos.org/guides/ad-hoc-developer-environments.html)
[flake](https://terranix.org/documentation/getting-startet-with-nix-flakes/) is provided. This flake
is also used for CI, and the `ci.sh` script in the root directory will run the same steps as the CI
would, inside the same nix environment.

This nix flake additionally provides some useful tooling, such as rust-analyzer and various cargo
utilities, as well as [direnv](https://direnv.net/) support for running your shell and/or editor
inside of the development environment defined by the flake.

Usage of the nix development environment is currently not needed for our development workflow,
however, as CI is run inside the nix environment, your patch set will be rejected if it does not
function properly inside of the nix environment.

The nix flake provides a (optional) binary cache, due to a UI quirk of `direnv` you will want to run
`nix build` or some other command that prompts to accept the cache before running `direnv allow`

### Rust

actm does not currently have a set MSRV, and currently targets the latest stable compiler. The exact
version of the toolchain in use is defined by the nix flake, but when developing without it, please
use the latest stable release of the toolchain, at least for `rustc` and `clippy`.

### cargo-audit

This project makes use of [cargo-audit](https://github.com/rustsec/rustsec/tree/HEAD/cargo-audit)
for scanning for known vulnerabilities in dependencies.

Your patch set will be rejected if it adds any new cargo-audit hits, however, your patch set is not
required to fix existing hits for approval.

### cargo-nextest

This project's official nix development environment provides [cargo-nextest](https://nexte.st/).
While not required, as it should give identical results to `cargo test`, it is substantially faster
in our use case, and it is highly recommended to make use of it.

It must be remembered that cargo-nextest does not currently support documentation tests, and thus
you will additionally need to run:

```bash
cargo test --doc
```

or use the `ci.sh` script to run documentation tests.

## Submitting Patches and Etiquette

### Git Commit Message Formatting

This project makes use of [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) to
procedurally generate the change log.

Your patch set will be rejected if it does not follow the conventional commit guidelines, or if it
does not include _both_ the `!` in the type field _and_ the the `BREAKING CHANGE` line in the event
your patch set makes a [semver](https://semver.org/) breaking change.

Please make the commit title as clear and descriptive as possible, as the description field of the
title line will be what shows up in the change log.

### Commit Structure

Each commit in your patch set should be as close to a complete logical unit of change as possible.
If you add a new required method to a trait, for instance, do not include intermediate commits in
your patch set, the trait and all its implementations should both be included in the same commit.

Every commit in a patch set should compile, and, ideally, pass ci.

### No Merge Commits

We keep the history for our default branch, `trunk`, linear, so patch sets containing a merge commit
will be rejected.

We encourage you to become familiar with [git rebase](https://git-rebase.io/) before contributing to
the project.

### Submitting Patches

This project does collaboration via email, please [git send-email](https://git-send-email.io/) your
patch sets to our development mailing list at \<~thatonelutenist/actm>.

## Programming guidelines and style notes

### Testing

Please make your best effort to write comprehensive tests for any new features you add, as well as
regression tests for any bugs you fix. This project makes extensive use of
[`proptest`](https://altsysrq.github.io/rustdoc/proptest/latest/proptest/), and it is recommended to
use it where appropriate.

[`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) is provided as part of the nix
development environment, and you are encouraged to try to keep your patch sets from decreasing
coverage as much as possible. We are well aware that `cargo-llvm-cov` has bugs and code coverage is
a dodgy metric to start with, but please exercise your best judgment.

### Imports

This project likes to keep its imports at the crate granularity level, which looks a bit like this:

```rust
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
use tracing::{debug, error, instrument, trace, warn};
```

It is recommended that you run `cargo fmt -- --config "imports_granularity=Crate"` to enforce this
formatting constraint.

### Panicking

As a general rule, there should never be a panic in the production version of the code, unless a
very bad and impossible to handle safely bug is detected. Even uncorrectable errors should be
processed through `Result`s instead of panicking wherever practical.

The usage of `assert!()` in the production version of the code is not allowed, signaling an error
through a `Result` should be used instead whenever practical, and an explicit panic in the rare case
where it is truly too onerous to implement error handling, both of which should have well thought
out errors that result in useful error messages.
