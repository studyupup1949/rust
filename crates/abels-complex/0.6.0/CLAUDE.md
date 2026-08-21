# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`abels-complex` is a Rust library providing complex number implementations with both rectangular (`Complex`) and polar (`ComplexPolar`) representations. The library supports `f32` and `f64` types and is designed for `no_std` environments with optional feature flags.

## Core Architecture

### Dual Representation System

The library implements two distinct forms for complex numbers:

- **Rectangular form** (`Complex<FT>`): Stores `re` and `im` components
- **Polar form** (`ComplexPolar<FT>`): Stores `abs` and `arg` components

**Critical design principle**: No implicit conversions between forms. Operations that naturally return a different form do so explicitly (e.g., `Complex::exp()` returns `ComplexPolar`, `ComplexPolar::ln()` returns `Complex`).

### Type Structure

```
src/
├── lib.rs              # Root module, exports complex types
├── traits.rs           # Number trait that abstracts over f32/f64
├── complex/
│   ├── mod.rs          # Re-exports rectangular and polar
│   ├── rectangular.rs  # Generic Complex<FT> implementation
│   ├── polar.rs        # Generic ComplexPolar<FT> implementation
│   ├── f32/            # f32-specific implementations and trait impls
│   │   ├── rectangular.rs
│   │   └── polar.rs
│   └── f64/            # f64-specific implementations and trait impls
│       ├── rectangular.rs
│       └── polar.rs
└── dual/               # Dual numbers (currently commented out in lib.rs)
    ├── mod.rs
    ├── rectangular.rs
    └── polar.rs
```

### Number Trait

The `Number` trait in `src/traits.rs` abstracts over floating-point types (`f32` and `f64`). It combines multiple `num-traits` traits to provide a unified interface for mathematical operations. When implementing new functionality, use `Number` as the trait bound rather than individual traits.

## Building and Testing

### Build Commands
```bash
# Standard build with all default features
cargo build

# Build for no_std with libm
cargo build --no-default-features --features libm

# Build with specific features
cargo build --no-default-features --features "std,rand,approx"
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Test with different feature combinations
cargo test --no-default-features --features libm
cargo test --features "std,rand,approx,glam"
```

### Available Features
- `std` (default): Standard library support
- `libm` (mutually exclusive with `std`): Use libm for math functions in `no_std` environments
- `rand` (default): Random number generation support
- `approx` (default): Approximate float comparison traits
- `glam` (default): Integration with glam vector types

## Key Design Decisions

### No Implicit Normalization

`ComplexPolar` does not automatically normalize. This means:
- `abs` can be negative or zero
- `arg` can be outside the range `(-π, π]`

Call `.normalize()` explicitly when needed to ensure `abs ≥ 0` and `-π < arg ≤ π`.

### Return Type Philosophy

Functions return the form that is most efficient for the operation:
- `Complex::exp()` → `ComplexPolar` (naturally produces polar form)
- `Complex::powi()` → `ComplexPolar` (more efficient in polar)
- `ComplexPolar::ln()` → `Complex` (naturally produces rectangular form)

This avoids hidden conversion costs and makes performance characteristics visible.

### Type-Specific Implementations

The `f32/` and `f64/` subdirectories contain:
- Operator overloads for scalar-complex operations (e.g., `f64 + Complex64`)
- Display trait implementations
- Feature-gated integrations (glam conversions, rand distributions)
- Type-specific constants (e.g., `Complex64::NEG_ONE`)

## Development Notes

- The dual numbers module exists but is currently commented out in `src/lib.rs`
- All arithmetic operations are implemented via operator overloading
- The library uses `#[repr(C)]` for both `Complex` and `ComplexPolar` for FFI compatibility
- When adding new mathematical functions, consider which form (rectangular/polar) is most natural for the computation
