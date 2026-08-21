# Abaco — Math Engine for Rust

> Italian/Spanish: abacus

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/abaco)](https://crates.io/crates/abaco)
[![docs.rs](https://docs.rs/abaco/badge.svg)](https://docs.rs/abaco)

**Abaco** is a Rust math engine providing expression evaluation and unit conversion. Shared library crate for the [AGNOS](https://github.com/MacCracken/agnosticos) ecosystem.

**Pure Rust, zero `unsafe`** — minimal dependencies, 99%+ test coverage.

## Features

| Module | Description |
|--------|-------------|
| `core` | Value types (Integer, Float, Fraction, Complex, Text), Unit, UnitCategory (14 categories), Currency |
| `eval` | Tokenizer, recursive descent parser, 28+ math functions, variables, scientific notation, percentage shorthand |
| `units` | Unit registry with 100+ built-in units, HashMap-indexed O(1) lookups, SI + IEC data sizes |
| `dsp` | Audio DSP math primitives — dB conversion, MIDI↔frequency, envelope time constants, PolyBLEP, panning, crossfade |
| `ai` | Natural language math parsing, calculation history (feature-gated) |

## Quick Start

```toml
[dependencies]
abaco = "0.22"

# With natural language parsing:
# abaco = { version = "0.22", features = ["ai"] }
```

```rust
use abaco::{Evaluator, UnitRegistry};

// Evaluate expressions
let eval = Evaluator::new();
let result = eval.eval("2 + 3 * 4").unwrap();
assert_eq!(result.to_string(), "14");

let result = eval.eval("sqrt(144) + sin(pi / 2)").unwrap();
assert_eq!(result.to_string(), "13");

// Percentage shorthand
let result = eval.eval("200 * 15%").unwrap();
assert_eq!(result.to_string(), "30");

// Variables
let mut eval = Evaluator::new();
eval.set_variable("x", 5.0);
let result = eval.eval("x ^ 2 + 1").unwrap();
assert_eq!(result.to_string(), "26");

// Unit conversion
let registry = UnitRegistry::new();
let r = registry.convert(100.0, "celsius", "fahrenheit").unwrap();
assert!((r.to_value - 212.0).abs() < 0.1);

// SI and IEC data sizes
let r = registry.convert(1.0, "GB", "GiB").unwrap();
assert!((r.to_value - 0.931).abs() < 0.001);
```

## Feature Flags

| Feature | Default | Description | Extra deps |
|---------|---------|-------------|------------|
| `ai` | no | Natural language math parsing, calculation history | reqwest, tokio |
| `full` | — | All optional features | (same as ai) |

## Consumers

| Project | Uses |
|---------|------|
| [Abacus](https://github.com/MacCracken/abacus) | Desktop calculator GUI, CLI, REPL, MCP tools |
| [Impetus](https://github.com/MacCracken/impetus) | Unit conversions, expression evaluation for physics simulation |

## Supported Functions (28+)

**Trig:** sin, cos, tan, asin, acos, atan, atan2
**Hyperbolic:** sinh, cosh, tanh, asinh, acosh, atanh
**Logarithmic:** log (log10), ln, log2
**Rounding:** ceil, floor, round, trunc, fract
**Other:** sqrt, abs, exp, sign/sgn, deg, rad, min, max, pow

## Unit Categories (14)

Length, Mass, Temperature, Time, DataSize (SI + IEC), Speed, Area, Volume, Energy, Pressure, Angle, Frequency, Force, Power

100+ units with case-insensitive lookup, plural forms, and case-sensitive symbol distinction (mW vs MW).

## License

AGPL-3.0-only
