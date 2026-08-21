# Changelog

All notable changes to Abaco will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.22.4] - 2026-03-22

### Added

- `dsp` module — pure numeric DSP math primitives for audio engines
  - Decibel conversions: `amplitude_to_db`, `db_to_amplitude` (f32 and f64 variants), `db_gain_factor`
  - MIDI: `midi_to_freq`, `freq_to_midi`, constants `A4_FREQUENCY`, `A4_MIDI_NOTE`, `SEMITONES_PER_OCTAVE`
  - Envelope: `time_constant` (one-pole smoothing coefficient from ms + sample rate)
  - Waveform: `poly_blep` (anti-aliasing correction), `angular_frequency` (biquad filter design)
  - Panning: `constant_power_pan` (sin/cos law), `equal_power_crossfade`
  - Utility: `sanitize_sample` (NaN/Inf → 0.0)
- 24 tests for dsp module
- 21 DSP criterion benchmarks (scalar + batch-4096)
- ROADMAP.md

### Performance

- dB conversions use `ln`/`exp` with precomputed constants instead of `log10`/`powf` — 42-62% faster
- MIDI-to-frequency uses `exp2` instead of `powf(2.0, x)`
- Pan/crossfade use single `sin_cos()` call instead of separate `sin()` + `cos()`

### Changed

- Benchmark script outputs both CSV history and 3-point tracking Markdown table
- 50 criterion benchmarks total (was 29), 242 tests

## [0.22.3] - 2026-03-22

### Performance

- Tokenizer rewritten to byte-level iteration: 43-62% faster expression evaluation
- Unit lookup indexed with HashMaps for O(1) symbol/name resolution: 94-98% faster lookups
- Registry creation pre-allocates HashMap capacity for 100+ units
- CalculationHistory switched from Vec to VecDeque for O(1) front eviction
- Function dispatch consolidated: arity check and dispatch in single match

### Added

- IEC binary data size units: KiB, MiB, GiB, TiB, PiB (powers of 1024)
- SI decimal data sizes corrected: kB, MB, GB, TB, PB now use powers of 1000
- Cross-conversion between SI and IEC (e.g. 1 GB = 0.931 GiB)
- 29 criterion benchmarks, 218 tests (all features), 99.4% line coverage
- CONTRIBUTING.md, CODE_OF_CONDUCT.md, SECURITY.md
- codecov.yml with 90% project target
- Example: examples/basic.rs
- CI: deny, MSRV, coverage, doc, benchmark, multi-platform test jobs
- Release workflow with crates.io publish and version verification

### Changed

- License aligned to AGPL-3.0-only across Cargo.toml, LICENSE, README
- Cargo.toml: added documentation, exclude fields
- deny.toml: added version fields, Unicode-DFS-2016
- Makefile: added coverage, test-all, doc with -D warnings
- CI: 8-job pipeline (was 4), multi-platform testing
- Release: library publish workflow (was binary packaging)
- .gitignore: comprehensive (was 6 lines)

### Fixed

- Bench-history script: handles criterion's wrapped benchmark name format

## [0.1.0] - 2026-03-22

### Changed — Flatten to shared math crate

- Refactored from multi-crate workspace to single flat library crate
- Extracted GUI and binary to [abacus](https://github.com/MacCracken/abacus)
- Feature-gated AI module behind `ai` feature flag
- Added rustls-tls to reqwest
- Removed binary deps (clap, anyhow, tracing-subscriber) — library only

### Modules

- `core` — Value types (Integer, Float, Fraction, Complex, Text), Unit, UnitCategory (14 categories), Currency
- `eval` — Tokenizer, recursive descent parser, evaluator with 28+ functions, variables, scientific notation, percentage shorthand
- `units` — Unit registry with 95+ built-in units across 14 categories, conversion engine
- `ai` — Natural language math parsing, calculation history (feature-gated)

[0.22.4]: https://github.com/MacCracken/abaco/compare/0.22.3...0.22.4
[0.22.3]: https://github.com/MacCracken/abaco/compare/0.1.0...0.22.3
[0.1.0]: https://github.com/MacCracken/abaco/releases/tag/0.1.0
