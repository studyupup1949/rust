# accelerate-sparse

Safe Rust bindings for the sparse direct solvers in the [Accelerate framework](https://developer.apple.com/documentation/accelerate/sparse_solvers) included with macOS.

`accelerate-sparse` exposes the solver phases separately: analyse a sparsity pattern, factor values against that analysis, refactor in place when only the values change, and solve. This makes the symbolic analysis and allocated factor storage reusable in iterative algorithms.

> **Status:** This crate is pre-1.0 and its public API may change.

## Features

- Cholesky and three concrete `LDLᵀ` factorizations for symmetric matrices.
- QR and Cholesky of `AᵀA` for rectangular least-squares problems.
- Three concrete LU factorizations for general square matrices.
- `f32` and `f64` values.
- Separate symbolic and numeric phases with in-place numeric refactorization.
- Single, multiple, strided, and in-place right-hand sides.
- Scalar and block compressed-column storage, with coordinate (triplet) assembly.
- Factor inertia and operator access to individual factorization pieces.
- Validated sparse structures and recoverable numeric failures.

The safe crate is a thin layer over Accelerate. It does not introduce a matrix type or abstract over other solver libraries.

## Requirements

- Rust 1.85 or newer.
- macOS with an Apple SDK and the Accelerate framework.

The solver API exists only on macOS. On other targets both crates build as empty libraries, so the dependency declaration needs no condition, but uses of solver items need `#[cfg(target_os = "macos")]`. A non-macOS build does not exercise the bindings.

Two operations have additional availability requirements:

- LU factorization requires an SDK that provides it and macOS 15.5 or newer.
- `Factorization::inertia` requires an SDK that provides it and macOS 13.0 or newer.

Unsupported runtime paths return an error instead of dispatching to a trap in Accelerate.

## Installation

```toml
[dependencies]
accelerate-sparse = "0.1"
```

## Example

The following solves a symmetric positive-definite system stored as the lower triangle in compressed-column form:

```rust
use accelerate_sparse::{
    Attributes, FactorizationKind, SparseStructure, SymbolicFactorization,
    Triangle,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A = [[4, 1, 0],
    //      [1, 3, 1],
    //      [0, 1, 2]]
    let column_starts = [0_i64, 2, 4, 5];
    let row_indices = [0_i32, 1, 1, 2, 2];
    let structure = SparseStructure::from_csc(
        3,
        3,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )?;

    // The sparsity pattern is analysed once.
    let symbolic =
        SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;

    // Values follow the same column-major order as the row indices.
    let mut factorization =
        symbolic.factorize(&[4.0_f64, 1.0, 3.0, 1.0, 2.0])?;
    let x = factorization.solve_vec(&[1.0, 2.0, 3.0])?;

    assert!((x[0] - 2.0 / 9.0).abs() < 1e-12);
    assert!((x[1] - 1.0 / 9.0).abs() < 1e-12);
    assert!((x[2] - 13.0 / 9.0).abs() < 1e-12);

    // The pattern, analysis, and factor storage are reused for new values.
    factorization.refactor(&[8.0, 2.0, 6.0, 2.0, 4.0])?;
    let x = factorization.solve_vec(&[1.0, 2.0, 3.0])?;

    assert!((x[0] - 1.0 / 9.0).abs() < 1e-12);
    Ok(())
}
```

Further [worked examples](https://github.com/w1th0utnam3/accelerate-sparse/tree/main/accelerate-sparse/examples)
cover a Newton-style refactor loop, least-squares methods, and subfactor operators. Run one with
`cargo run --example newton_refactor`.

## Choosing a factorization

| Kind | Input matrix | Purpose |
| --- | --- | --- |
| `Cholesky` | Symmetric positive definite | `LLᵀ` factorization |
| `LdltUnpivoted` | Symmetric | `LDLᵀ` without pivot search |
| `LdltSbk` | Symmetric, indefinite allowed | `LDLᵀ` with supernode Bunch-Kaufman pivoting |
| `LdltTpp` | Symmetric, indefinite allowed | `LDLᵀ` with threshold partial pivoting; supports inertia |
| `Qr` | General `m × n` | Least squares from an `m`-element right-hand side |
| `CholeskyAtA` | General `m × n`, with `m ≥ n` | Least squares from the reduced `n`-element right-hand side `Aᵀb` |
| `LuUnpivoted` | General square | LU without pivoting; macOS 15.5 or newer |
| `LuSpp` | General square | LU with supernode pivoting; macOS 15.5 or newer |
| `LuTpp` | General square | LU with threshold partial pivoting; macOS 15.5 or newer |

`Qr` and `CholeskyAtA` may return `Ok` for a rank-deficient matrix without providing a rank estimate or diagnostic. The resulting solution may be non-unique. `CholeskyAtA` also squares the condition number, so callers that need to detect rank deficiency or poor conditioning must assess it separately.

`CholeskyAtA` takes the reduced right-hand side `Aᵀb`, which the caller forms — nothing in this crate does. No shape check can enforce that: for a square matrix, `b` and `Aᵀb` have the same length, so passing `b` is accepted and solves the wrong system.

Factorization failure is a returned runtime status, not a panic. The caller can retry with another
factorization kind. A failed refactorization also remains retryable; solving is refused until one
succeeds.

## Storage and operands

Sparse patterns use compressed-column storage with `i64` column starts and `i32` row indices, matching the widths stored by Accelerate. `SparseStructure::from_csc` borrows arrays with those widths, while `SparseStructure::convert_from_csc` narrows other integer representations. `SparseStructure::from_coordinates` assembles an unsorted coordinate (triplet) list instead, summing duplicate entries and folding symmetric entries onto their mirror in the declared triangle.

Values are column-major. For symmetric matrices, declare which triangle is stored with `Attributes::symmetric`. A block size above one changes each stored entry into a dense column-major square tile.

`solve_vec` handles one right-hand side. `DenseRef` and `DenseMut` support multiple right-hand
sides and an explicit column stride without requiring a copy. Their constructors return
`Result`, rejecting zero dimensions, insufficient storage, invalid strides, and
dimensions that do not fit Accelerate's integer ABI before a view reaches the framework.

The `effective_*`, `right_hand_side_rows`, `solution_rows`, and `in_place_operand_rows` methods
report scalar matrix and dense-operand dimensions.

## Errors and thread safety

Malformed sparse patterns return `StructureError`, and dense-view constructors return `InputError`.
Solver validation failures use `Error::Input(InputError)`, whose `input()` exposes the structured
reason. Numeric, framework, and factorization-state outcomes use `Error::Status`; `status()` exposes
the status and an optional diagnostic supplies context.

`Factorization<T>` is `Sync` but not `Send`: shared references may solve concurrently, while a factorization remains on the thread that owns it. `SymbolicFactorization` is neither `Send` nor `Sync` and remains on the thread that created it.

Accelerate may produce small run-to-run floating-point differences when it factors in parallel. Set `VECLIB_MAXIMUM_THREADS=1` when a comparison must attribute a numerical difference to a code change.

## Workspace

- `accelerate-sparse` provides the validated, safe API.
- `accelerate-sparse-sys` provides hand-written declarations for the repository's C shim. Most users should depend on the safe crate.

The safe crate re-exports the sys crate as `accelerate_sparse::sys` for callers that need an unwrapped entry point.

## Development

Run the complete pre-merge checks on Apple hardware:

```bash
cargo fmt --all --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
cargo doc --no-deps --all-features
cargo test --all-features --no-fail-fast
```

The empty-library path can be checked separately:

```bash
cargo build --target x86_64-unknown-linux-gnu
```

## Trademarks

`accelerate-sparse` is an independent project and is not affiliated with, sponsored by, or endorsed by Apple Inc.

Apple and macOS are trademarks of Apple Inc., registered in the U.S. and other countries and regions. The Accelerate framework is included with macOS and is not redistributed by this project.

## License

The `accelerate-sparse` and `accelerate-sparse-sys` crates are licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT License](LICENSE-MIT)

at your option.
