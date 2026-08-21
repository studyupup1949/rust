# AATP Audit & Integrity Certification

- **Date:** 2026-07-11
- **Status:** Audit-Certified / Production Ready
- **Commit:** `9d203924f1e5e6a8103dd46767ade985b1521b30`

## Executive Summary

AATP has been put through an *Adversarial Deep-Audit* by specialised agents, backed by a
100% behavioural mutation analysis across **all** modules (codec, message, state, dict,
shadow, lib). Every finding was reproduced, fixed, and re-verified. The underlying
evidence lives in [`.audit/`](.audit/) and is reproducible.

## Integrity Matrix

The following findings were identified and closed during the audit:

| Finding | Severity | Fix (empirically verified) |
|:---|:---|:---|
| Comparative speed claim (unbacked) | High | Softened to an argument from structure; the crate ships **no** cross-format benchmark, only a self-throughput one in the dev workspace. |
| Placeholder repository URL | Med | Replaced with the live repository (`repository = "https://github.com/MerlijnW70/aatp"`); no longer a dead link. |
| Macro E0392 (all-scalar struct) | Med | Added a dedicated no-lifetime `aatp_message!` arm. |
| Macro hollow coverage (u16 arm) | Low-Med | Added a `u16` field to the macro integration tests. |
| Dictionary pinning typo risk | Low-Med | Added an entry-for-entry snapshot test pinning the vocabulary. |
| Build-artifact cruft | Low | `cargo clean` + `.gitignore` rules (`/target`, `/Cargo.lock`). |
| CRC vs. authentication scope | Low | Explicit "detects corruption, not a malicious sender" caveat in the README and module docs. |

## What "100% Mutation Analysis" means here (scope, honestly)

"100%" means **zero survivors under the mutation operator set** used to audit the code:
comparison / boolean / arithmetic operator swaps, condition forcing, and unary-negation
removal. Per module: codec 8/8 · message 25/25 · state 17/17 · dict 35/35 · shadow 16/16
· lib 46/46 — **147/147, 0 survivors**
([`.audit/mutation-analysis/coverage-report.txt`](.audit/mutation-analysis/coverage-report.txt)).

It is a strong signal, **not** a proof of total correctness. The operator set does not
mutate integer/byte literals, so the wire-critical constant tables (CRC32C table,
SplitMix64 keystream, dictionary) are pinned separately by known-answer, snapshot, and
fold-digest tests — and every class of single-bit corruption was reproduced and confirmed
to fail the build
([`.audit/verification-proofs/fuzz-and-corruption.txt`](.audit/verification-proofs/fuzz-and-corruption.txt)).

## Verification Statement

This release (commit `9d20392`) is certified to meet the following reproducible requirements:

- **`no_std`** — verified by a bare-metal build on `thumbv7em-none-eabihf`.
- **`#![forbid(unsafe_code)]`** — enforced by the compiler as a hard error; no `unsafe` in the crate.
- **100% behavioural mutation coverage** — 0 survivors across all modules (147/147).
- **Data integrity** — every class of single-bit corruption fails the build.
- **Zero dependencies** — no runtime *and* no dev dependencies.
- **CI** — fmt · clippy · test (3 OS × {stable, MSRV 1.85}) · no_std, all jobs green.

Evidence: [`.audit/`](.audit/). This document is evidence of the integrity of the current
release — it claims demonstrably-tested correctness, not "provably bug-free."
