# Contributing to AATP

Thanks for your interest. AATP holds a deliberately high bar; contributions are welcome as
long as they keep it.

## Ground rules

AATP is, and stays:

- **Zero-dependency** — no runtime *and* no dev dependencies. The tests and the
  fuzz/property harness use only the standard library and a hand-written PRNG.
- **`#![no_std]`** — the library must build for a bare-metal target.
- **`#![forbid(unsafe_code)]`** — no `unsafe`, anywhere.

A change that adds a dependency, breaks `no_std`, or introduces `unsafe` will not be merged.

## Development

```sh
cargo test --all          # unit + integration + doc tests
cargo fmt --all --check    # formatting (CI-enforced)
cargo clippy --all-targets -- -D warnings   # lints (CI-enforced)
cargo build --lib --target thumbv7em-none-eabihf   # no_std proof
```

Longer fuzz soak:

```sh
AATP_FUZZ_ITERS=1000000 cargo test --release --test fuzz
```

## The testing bar

New behaviour must be **tested to the same standard as the rest of the crate**:

- Every logical branch is covered — the project targets **0 survivors** under behavioural
  mutation analysis. If you add a comparison, a boundary, or a branch, add the test that
  pins it.
- Wire-critical constants (tables, keys, dictionary entries) are pinned by known-answer,
  snapshot, or digest tests, because mutation analysis does not reach literals.
- Parsers must fail closed on hostile input — never panic, over-read, or over-allocate.
  Add or extend a `tests/fuzz.rs` property if you touch a decoder.

## Pull requests

1. Keep PRs focused and describe what and why.
2. Ensure all CI jobs are green (fmt, clippy, the OS × MSRV test matrix, `no_std`).
3. Update `CHANGELOG.md` under `[Unreleased]`.
4. Update `AUDIT.md` / `.audit/` if you change the verification story.

## License

By contributing, you agree that your contributions are licensed under the
[MIT License](LICENSE).
