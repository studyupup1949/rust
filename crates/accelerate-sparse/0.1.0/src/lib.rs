//! Safe Rust bindings for the sparse solvers in the Accelerate framework included with macOS.
//!
//! The API exposes Accelerate's phases directly: analyse a sparsity pattern, factor values against
//! that analysis, re-form the factorization when only the values change, and solve. The analysis
//! and factor storage can be reused across iterations, including in Newton solvers.
//!
//! The main entry points follow those phases:
//!
//! - [`SparseStructure`] validates a compressed-column sparsity pattern.
//! - [`SymbolicFactorization`] analyses that pattern for a chosen [`FactorizationKind`].
//! - [`Factorization`] factors values, refactors them in place, and solves systems.
//! - [`DenseRef`] and [`DenseMut`] describe strided column-major operands for multiple right-hand
//!   sides.
//!
//! ```
//! use accelerate_sparse::{
//!     Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]], lower triangle, column-major.
//! let column_starts = [0i64, 2, 4, 5];
//! let row_indices = [0i32, 1, 1, 2, 2];
//! let structure = SparseStructure::from_csc(
//!     3,
//!     3,
//!     &column_starts,
//!     &row_indices,
//!     Attributes::symmetric(Triangle::Lower),
//! )?;
//!
//! // The pattern is analysed once and reused for every set of values that shares it.
//! let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;
//!
//! let mut factorization = symbolic.factorize(&[4.0, 1.0, 3.0, 1.0, 2.0])?;
//! let x = factorization.solve_vec(&[1.0, 2.0, 3.0])?;
//!
//! // Later iterations change only the values, reusing both the analysis and the factor memory.
//! factorization.refactor(&[8.0, 2.0, 6.0, 2.0, 4.0])?;
//! let x = factorization.solve_vec(&[1.0, 2.0, 3.0])?;
//! # let _ = x;
//! # Ok(())
//! # }
//! ```
//!
//! # Choosing a factorization
//!
//! | Kind | Matrix | Notes |
//! | --- | --- | --- |
//! | [`FactorizationKind::Cholesky`] | symmetric positive definite | fails in the numeric phase otherwise |
//! | [`FactorizationKind::LdltUnpivoted`] | symmetric, no pivoting | fails when a pivot is too small, including on many indefinite matrices |
//! | [`FactorizationKind::LdltSbk`] | symmetric, indefinite allowed | fails only if singular |
//! | [`FactorizationKind::LdltTpp`] | symmetric, indefinite allowed | fails only if singular; the only kind [`Factorization::inertia`] accepts |
//! | [`FactorizationKind::Qr`] | general, any `m × n` | least squares; may not report rank deficiency |
//! | [`FactorizationKind::CholeskyAtA`] | general `m × n`, with `m ≥ n` | least squares via `AᵀA`; may not report rank deficiency and loses accuracy as conditioning worsens |
//! | [`FactorizationKind::LuUnpivoted`] / [`LuSpp`] / [`LuTpp`] | general square, unsymmetric | needs macOS 15.5; older OS reports [`Status::UnsupportedOs`] |
//!
//! [`LuSpp`]: FactorizationKind::LuSpp
//! [`LuTpp`]: FactorizationKind::LuTpp
//!
//! [`Factorization::inertia`] counts the factored matrix's eigenvalues by sign, for
//! [`FactorizationKind::LdltTpp`] alone. It needs macOS 13.0 to run, and an SDK providing the
//! underlying function to build; either one missing reports [`Status::UnsupportedOs`], and
//! [`sys::HAS_INERTIA`] reports the SDK half without making a call.
//!
//! # Working with the factors
//!
//! [`Factorization::subfactor`] hands back one piece of a factorization — its `L` or `Q`, a
//! permutation, a scaling — as a [`Subfactor`]. A subfactor is an operator, not a matrix: it can
//! be solved with and multiplied by, but not read entry by entry.
//! [`SubfactorKind::applies_to`] says which kinds supply which piece.
//!
//! The pieces include the fill-reducing permutation because `L` alone does not reconstruct the
//! matrix: Accelerate's Cholesky is `A = P L Lᵀ Pᵀ`, and the other kinds are permuted the same
//! way. Two of the pieces are half-solves that reproduce a full solve when applied twice, in the
//! order the [`SubfactorKind`] variants document — which differs between the symmetric and the
//! normal-equations forms.
//!
//! A subfactor borrows its factorization and applies its live state, so the factorization cannot be
//! refactored while a piece is alive.
//!
//! # Element types
//!
//! [`f64`] and [`f32`], through the sealed [`Scalar`] trait. A
//! [`SymbolicFactorization`] serves either, because the analysis depends on the pattern and not on
//! the values; the element type is fixed when values are first factored against it, and the same
//! analysis can carry factorizations of both at once. The default pivot and zero tolerances differ
//! by element type and come from the framework.
//!
//! # Storage
//!
//! Sparse input is compressed-column (CSC): each value belongs to the row index at the same
//! position within its column's range. Values remain in exactly the same order as their indices.
//! Accelerate stores `i64` column starts and `i32` row indices, widths no Rust sparse library
//! matches. [`SparseStructure::from_csc`] borrows arrays with those widths,
//! [`SparseStructure::convert_from_csc`] may allocate to narrow other index types, and
//! [`SparseStructure::from_coordinates`] assembles an unsorted coordinate (triplet) list, summing
//! duplicates and folding symmetric entries into the declared triangle. Values pass unchanged to
//! [`SymbolicFactorization::factorize`] without an adapter allocation.
//!
//! ```
//! use accelerate_sparse::{
//!     Attributes, DenseMut, DenseRef, FactorizationKind, SparseStructure, SymbolicFactorization,
//!     Triangle,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Raw CSC storage from another matrix type. These values are the lower triangle of
//! // [[4, 1, 0], [1, 3, 1], [0, 1, 2]].
//! let column_starts = [0usize, 2, 4, 5];
//! let row_indices = [0usize, 1, 1, 2, 2];
//! let values = [4.0, 1.0, 3.0, 1.0, 2.0];
//!
//! let structure = SparseStructure::convert_from_csc(
//!     3,
//!     3,
//!     &column_starts,
//!     &row_indices,
//!     Attributes::symmetric(Triangle::Lower),
//! )?;
//! assert_eq!(structure.column_starts(), &[0, 2, 4, 5]);
//! assert_eq!(structure.row_indices(), &[0, 1, 1, 2, 2]);
//!
//! let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;
//! let factorization = symbolic.factorize(&values)?;
//!
//! // Compact dense operands are column-major: each group of three is one right-hand side.
//! let right_hand_sides = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
//! let b = DenseRef::from_column_major_slice(&right_hand_sides, 3, 2)?;
//! let mut solutions = [0.0; 6];
//! let mut x = DenseMut::from_column_major_slice(&mut solutions, 3, 2)?;
//! assert_eq!(b.column_stride(), 3);
//! assert_eq!(b.backing_slice(), &right_hand_sides);
//! assert_eq!(x.backing_slice_mut().len(), 6);
//! factorization.solve_into(b, x)?;
//! # Ok(())
//! # }
//! ```
//!
//! Compact and regularly strided dense operands must be column-major; use
//! [`DenseRef::from_column_major_slice_with_stride`] and
//! [`DenseMut::from_column_major_slice_with_stride`] for the latter. Row-major or arbitrarily
//! strided dense operands require a copy into column-major storage.
//!
//! A [`SparseStructure::with_block_size`] above one switches to the block form supported by
//! Accelerate: the matrix becomes a grid of dense `block_size` by `block_size` tiles, the pattern
//! counts tiles rather than scalars, and each stored tile contributes
//! `block_size * block_size` column-major values. Dense right-hand sides and solutions stay in
//! scalars. [`SparseStructure::with_block_size`] documents the symmetric-triangle rules for blocks.
//! [`Factorization::effective_rows`], [`Factorization::effective_columns`],
//! [`Factorization::right_hand_side_rows`], [`Factorization::solution_rows`], and
//! [`Factorization::in_place_operand_rows`] report the resulting scalar matrix and dense-operand
//! dimensions.
//!
//! For a symmetric matrix only one triangle is read; [`Attributes`] documents the scalar rules and
//! [`SparseStructure::with_block_size`] the block rules. **Both are observed and not documented by
//! Apple for this path.**
//!
//! # Errors
//!
//! Public input validation is fallible. [`StructureError`] reports malformed sparse storage before
//! it reaches Accelerate. [`DenseRef`] and [`DenseMut`] constructors return [`InputError`] directly.
//! Solver operations wrap other caller-supplied incompatibilities in [`Error::Input`], including
//! factorization shape and ordering, values length, operand shape, and unavailable subfactor
//! operations.
//!
//! [`Error::Status`] carries a [`Status`] from Accelerate or factorization state. Failed refactors
//! remain retryable. Framework diagnostics, when available, are exposed by [`Error::detail`].
//! [`Factorization::inertia`] can likewise return Accelerate's [`Status::ParameterError`] for a
//! factorization kind that does not support the query.
//!
//! # Thread safety
//!
//! [`Factorization`] is [`Sync`] but not [`Send`]: shared references can solve concurrently,
//! reference-count changes are serialized, and the owner stays on its creating thread.
//! [`SymbolicFactorization`] is neither, because the factorization it produces shares its Accelerate
//! reference count without borrowing it. Each type documents its ownership constraint.
//!
//! Parallel factorization can produce small run-to-run floating-point differences. Set
//! `VECLIB_MAXIMUM_THREADS=1` for reproducible comparisons. Separately, an unconfirmed report
//! describes rare hangs inside Accelerate's symmetric factorization on some hardware, which the
//! same setting avoids. Both effects are Accelerate's internal threading, unrelated to the `Send`
//! and `Sync` bounds above.
//!
//! See the [Sparse Solvers documentation][1] published by Apple for the underlying library.
//!
//! [1]: https://developer.apple.com/documentation/accelerate/sparse_solvers
//!
//! # Trademarks
//!
//! `accelerate-sparse` is an independent project and is not affiliated with, sponsored by, or
//! endorsed by Apple Inc.
//!
//! Apple and macOS are trademarks of Apple Inc., registered in the U.S. and other countries and
//! regions. The Accelerate framework is included with macOS and is not redistributed by this
//! project.
//!
//! # Platform
//!
//! macOS only in substance. On other targets this compiles to an empty library, allowing it to
//! remain in cross-platform dependency trees. The solver items exist only on macOS, so
//! cross-platform code must gate uses:
//!
//! ```
//! #[cfg(target_os = "macos")]
//! fn solve_with_accelerate() {
//!     use accelerate_sparse::SparseStructure;
//!     // ...
//! }
//! ```
//!
//! [`Status`]: crate::error::Status
//! [`Status::UnsupportedOs`]: crate::error::Status::UnsupportedOs
//! [`Status::ParameterError`]: crate::error::Status::ParameterError
//! [`StructureError`]: crate::error::StructureError
//! [`InputError`]: crate::error::InputError
//! [`Error::Input`]: crate::error::Error::Input
//! [`Error::Status`]: crate::error::Error::Status
//! [`Error::detail`]: crate::error::Error::detail

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Exposes the raw FFI layer beneath this crate.
///
/// Re-exported so unwrapped entry points use the same shim version as the safe API.
pub use accelerate_sparse_sys as sys;

#[cfg(any(target_os = "macos", docsrs))]
mod dense;
#[cfg(any(target_os = "macos", docsrs))]
mod diagnostics;
#[cfg(any(target_os = "macos", docsrs))]
pub mod error;
#[cfg(any(target_os = "macos", docsrs))]
mod factorization;
#[cfg(any(target_os = "macos", docsrs))]
mod kind;
#[cfg(any(target_os = "macos", docsrs))]
pub mod options;
#[cfg(any(target_os = "macos", docsrs))]
mod scalar;
#[cfg(any(target_os = "macos", docsrs))]
mod structure;

#[cfg(any(target_os = "macos", docsrs))]
pub use crate::{
    dense::{DenseMut, DenseRef},
    factorization::{Factorization, Inertia, Subfactor, SymbolicFactorization},
    kind::{FactorizationKind, SubfactorKind},
    scalar::Scalar,
    structure::{Attributes, MatrixKind, SparseStructure, Triangle},
};
