//! Enumerations select the factorization and name the pieces it can supply.

use accelerate_sparse_sys as sys;

/// Selects the factorization Accelerate computes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FactorizationKind {
    /// Cholesky (`LLᵀ`). Requires a symmetric positive-definite matrix; the numeric phase fails
    /// otherwise.
    Cholesky,
    /// `LDLᵀ` that does no pivot search, taking each diagonal in turn as a scalar pivot. The numeric
    /// phase fails when a pivot is too small to divide by, which happens on many indefinite
    /// matrices. Use it only when the matrix is known not to need pivoting.
    LdltUnpivoted,
    /// `LDLᵀ` with supernode Bunch-Kaufman pivoting. Handles symmetric indefinite matrices.
    LdltSbk,
    /// `LDLᵀ` with threshold partial pivoting. Handles symmetric indefinite matrices.
    ///
    /// The only kind [`Factorization::inertia`](crate::Factorization::inertia) accepts.
    LdltTpp,
    /// QR of a general `m × n` matrix, solving the least-squares problem `min ‖A x − b‖`. The
    /// right-hand side has `m` rows and the solution `n`, so the matrix need not be square.
    ///
    /// **Observed:** For a wide matrix of full row rank, this returns the
    /// minimum-Euclidean-norm solution over all `n` entries.
    ///
    /// A successful factorization or solve does not establish that the matrix has full rank.
    /// Rank-deficient inputs have been observed to return `Ok` without a rank estimate or
    /// rank-deficiency status. Their solutions may be non-unique; assess rank separately when it
    /// matters to the caller.
    Qr,
    /// Cholesky of the normal equations `AᵀA = RᵀR`, which keeps `R` but discards the orthogonal
    /// factor. Solves the same least-squares problem as [`Qr`](Self::Qr) from the reduced `n`-row
    /// right-hand side.
    ///
    /// **The right-hand side is `Aᵀb`, not `b`.** The caller must form it. Only its length `n` is
    /// checked, so passing `b` for a square matrix is accepted and solves the wrong system.
    ///
    /// Requires at least as many rows as columns; otherwise `AᵀA` is singular and
    /// [`SymbolicFactorization::new`](crate::SymbolicFactorization) returns an input error. Use
    /// [`Qr`](Self::Qr) for a wide least-squares problem.
    ///
    /// The rank-deficiency caveat on [`Qr`](Self::Qr) applies. Forming `AᵀA` also squares the
    /// condition number, so assess conditioning separately when it matters to the caller.
    CholeskyAtA,
    /// LU of a general square matrix, no pivoting.
    ///
    /// The LU kinds factor an unsymmetric square matrix, so the whole matrix is read — use
    /// [`Attributes::ordinary`](crate::Attributes::ordinary), not a triangle.
    ///
    /// Needs macOS 15.5 to build and run. Availability errors come from
    /// [`SymbolicFactorization::new`](crate::SymbolicFactorization). An older OS reports
    /// [`Status::UnsupportedOs`](crate::error::Status::UnsupportedOs) rather than trapping; built
    /// against an SDK without LU, it reports
    /// [`Status::ParameterError`](crate::error::Status::ParameterError).
    LuUnpivoted,
    /// LU of a general square matrix, pivoting confined to each supernode rather than searching a
    /// whole column. Same OS requirement as [`LuUnpivoted`](Self::LuUnpivoted).
    LuSpp,
    /// LU of a general square matrix, threshold partial pivoting. Same OS requirement as
    /// [`LuUnpivoted`](Self::LuUnpivoted).
    LuTpp,
}

/// Identifies one piece of a factorization using Accelerate's names.
///
/// Which pieces a factorization can supply depends on its kind, and for
/// [`S`](Self::S) also on whether it was scaled; [`applies_to`](Self::applies_to) is the rule.
/// Extract one with [`Factorization::subfactor`](crate::Factorization::subfactor).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SubfactorKind {
    /// The reordering the analysis chose, as an operator. Every kind has one. An LU factorization
    /// reorders rows and columns separately and offers this as the row half, with [`Q`](Self::Q) as
    /// the column half.
    P,
    /// The diagonal scaling the numeric phase applied.
    ///
    /// **Observed:** Offered by the `LDLᵀ` kinds but not Cholesky, under every accepted scaling
    /// method.
    //
    // Should a future version of this crate expose the scaling method, re-establish the rule
    // above.
    S,
    /// The `L` of a Cholesky or `LDLᵀ` factorization.
    ///
    /// `L` alone does not reconstruct the matrix: Accelerate's Cholesky is `A = P L Lᵀ Pᵀ`, so the
    /// permutation belongs in the product.
    L,
    /// The `D` of an `LDLᵀ` factorization.
    D,
    /// Half of a symmetric solve, bundled as one operator.
    ///
    /// Solving with it and then with its transpose reproduces what the factorization's own solve
    /// does, so it is the way to reach the intermediate vector between the two triangular sweeps.
    /// Applied untransposed it carries the reordering and the lower factor together; transposed it
    /// carries the diagonal as well. Solve only; see
    /// [`supports_multiply`](Self::supports_multiply).
    Plps,
    /// The orthogonal factor of a QR factorization — and, for LU, the column reordering.
    ///
    /// Accelerate uses one selector for both, and so does this crate. Which one a handle holds
    /// follows from the kind it was taken from; they also differ in shape, reported by
    /// [`Subfactor::rows`](crate::Subfactor::rows).
    Q,
    /// The `R` of a QR or `CholeskyAtA` factorization.
    R,
    /// The half-solve of a QR or `CholeskyAtA` factorization.
    ///
    /// Solving with its transpose and then with this reproduces a `CholeskyAtA` factorization's own
    /// solve — the transpose first, which is the opposite order from [`Plps`](Self::Plps), because
    /// `AᵀA = Rᵀ R` puts `Rᵀ` against the right-hand side. The wrong order returns a plausible vector
    /// rather than an error. Solve only; see [`supports_multiply`](Self::supports_multiply).
    Rp,
    /// The row scaling of an LU factorization.
    Sr,
    /// The column scaling of an LU factorization.
    Sc,
}

impl SubfactorKind {
    /// Whether `kind` can supply this piece.
    ///
    /// **Observed:** The [Sparse Solvers documentation][1] lists [`S`](Self::S) for Cholesky, but
    /// Cholesky does not supply it.
    ///
    /// [1]: https://developer.apple.com/documentation/accelerate/sparse_solvers
    pub fn applies_to(self, kind: FactorizationKind) -> bool {
        use FactorizationKind as F;
        let ldlt = matches!(kind, F::LdltUnpivoted | F::LdltSbk | F::LdltTpp);
        match self {
            Self::P => true,
            Self::S => ldlt,
            Self::L => ldlt || kind == F::Cholesky,
            Self::D => ldlt,
            Self::Plps => ldlt || kind == F::Cholesky,
            Self::Q => kind == F::Qr || kind.is_lu(),
            Self::R | Self::Rp => matches!(kind, F::Qr | F::CholeskyAtA),
            Self::Sr | Self::Sc => kind.is_lu(),
        }
    }

    /// Whether this piece can be multiplied by, as opposed to only solved with.
    ///
    /// The two half-solve pieces cannot. Multiplying by [`Plps`](Self::Plps) aborts the process
    /// inside Accelerate; the check keeps that call unreachable.
    pub fn supports_multiply(self) -> bool {
        !matches!(self, Self::Plps | Self::Rp)
    }

    /// The subfactor's shape in scalars, given its parent's kind and scalar dimensions.
    ///
    /// **Observed:** Accelerate sizes pieces from the larger and smaller parent dimensions. Every
    /// piece is square with the smaller dimension, except QR's `Q`, which is
    /// `larger × smaller`.
    ///
    /// The parent's transpose attribute does not affect the shape; the subfactor tracks its own.
    pub(crate) fn scalar_shape(
        self,
        kind: FactorizationKind,
        rows: usize,
        columns: usize,
    ) -> (usize, usize) {
        let larger = rows.max(columns);
        let smaller = rows.min(columns);
        if self == Self::Q && kind == FactorizationKind::Qr {
            (larger, smaller)
        } else {
            (smaller, smaller)
        }
    }

    pub(crate) fn to_raw(self) -> u8 {
        match self {
            Self::P => sys::ACCSP_SUBFACTOR_P,
            Self::S => sys::ACCSP_SUBFACTOR_S,
            Self::L => sys::ACCSP_SUBFACTOR_L,
            Self::D => sys::ACCSP_SUBFACTOR_D,
            Self::Plps => sys::ACCSP_SUBFACTOR_PLPS,
            Self::Q => sys::ACCSP_SUBFACTOR_Q,
            Self::R => sys::ACCSP_SUBFACTOR_R,
            Self::Rp => sys::ACCSP_SUBFACTOR_RP,
            Self::Sr => sys::ACCSP_SUBFACTOR_SR,
            Self::Sc => sys::ACCSP_SUBFACTOR_SC,
        }
    }
}

impl FactorizationKind {
    /// Whether this factorization requires the matrix to be square.
    ///
    /// The symmetric and LU kinds do; the rectangular [`Qr`](Self::Qr) and
    /// [`CholeskyAtA`](Self::CholeskyAtA) do not. `CholeskyAtA` still needs at least as many rows
    /// as columns, which [`requires_rows_ge_columns`](Self::requires_rows_ge_columns) reports.
    pub fn requires_square(self) -> bool {
        match self {
            Self::Cholesky
            | Self::LdltUnpivoted
            | Self::LdltSbk
            | Self::LdltTpp
            | Self::LuUnpivoted
            | Self::LuSpp
            | Self::LuTpp => true,
            Self::Qr | Self::CholeskyAtA => false,
        }
    }

    /// Whether this factorization requires at least as many rows as columns.
    ///
    /// True only for [`CholeskyAtA`](Self::CholeskyAtA), whose normal equations `AᵀA` are singular
    /// for a wider matrix. [`Qr`](Self::Qr) has no such shape requirement. The square kinds satisfy
    /// the condition trivially and report `false` here —
    /// [`requires_square`](Self::requires_square) is the stronger check that covers them.
    pub fn requires_rows_ge_columns(self) -> bool {
        matches!(self, Self::CholeskyAtA)
    }

    /// Whether this is one of the LU kinds, whose symbolic phase the safe layer must serialize.
    pub(crate) fn is_lu(self) -> bool {
        matches!(self, Self::LuUnpivoted | Self::LuSpp | Self::LuTpp)
    }

    pub(crate) fn to_raw(self) -> core::ffi::c_int {
        match self {
            Self::Cholesky => sys::ACCSP_KIND_CHOLESKY,
            Self::LdltUnpivoted => sys::ACCSP_KIND_LDLT_UNPIVOTED,
            Self::LdltSbk => sys::ACCSP_KIND_LDLT_SBK,
            Self::LdltTpp => sys::ACCSP_KIND_LDLT_TPP,
            Self::Qr => sys::ACCSP_KIND_QR,
            Self::CholeskyAtA => sys::ACCSP_KIND_CHOLESKY_ATA,
            Self::LuUnpivoted => sys::ACCSP_KIND_LU_UNPIVOTED,
            Self::LuSpp => sys::ACCSP_KIND_LU_SPP,
            Self::LuTpp => sys::ACCSP_KIND_LU_TPP,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins every kind to its shim selector. The `_Static_assert`s in the shim fix the selector
    /// values to Accelerate's, but nothing else checks that each enum arm maps to the right one —
    /// and the behavioural tests cannot, since kinds that share a solve result (the LU variants,
    /// SBK vs TPP) are indistinguishable there. A swapped arm would otherwise be silent.
    #[test]
    fn every_kind_maps_to_its_selector() {
        use FactorizationKind::*;
        for (kind, raw) in [
            (Cholesky, sys::ACCSP_KIND_CHOLESKY),
            (LdltUnpivoted, sys::ACCSP_KIND_LDLT_UNPIVOTED),
            (LdltSbk, sys::ACCSP_KIND_LDLT_SBK),
            (LdltTpp, sys::ACCSP_KIND_LDLT_TPP),
            (Qr, sys::ACCSP_KIND_QR),
            (CholeskyAtA, sys::ACCSP_KIND_CHOLESKY_ATA),
            (LuUnpivoted, sys::ACCSP_KIND_LU_UNPIVOTED),
            (LuSpp, sys::ACCSP_KIND_LU_SPP),
            (LuTpp, sys::ACCSP_KIND_LU_TPP),
        ] {
            assert_eq!(kind.to_raw(), raw, "{kind:?} maps to the wrong selector");
        }
    }

    /// Pins subfactor selectors because pairing tests cannot detect swaps between pieces with the
    /// same availability.
    #[test]
    fn every_subfactor_maps_to_its_selector() {
        use SubfactorKind::*;
        for (piece, raw) in [
            (P, sys::ACCSP_SUBFACTOR_P),
            (S, sys::ACCSP_SUBFACTOR_S),
            (L, sys::ACCSP_SUBFACTOR_L),
            (D, sys::ACCSP_SUBFACTOR_D),
            (Plps, sys::ACCSP_SUBFACTOR_PLPS),
            (Q, sys::ACCSP_SUBFACTOR_Q),
            (R, sys::ACCSP_SUBFACTOR_R),
            (Rp, sys::ACCSP_SUBFACTOR_RP),
            (Sr, sys::ACCSP_SUBFACTOR_SR),
            (Sc, sys::ACCSP_SUBFACTOR_SC),
        ] {
            assert_eq!(piece.to_raw(), raw, "{piece:?} maps to the wrong selector");
        }
    }

    /// Pins which kinds carry the rows-at-least-columns precondition. Only `CholeskyAtA` does;
    /// pinning every arm guards against widening it to `Qr`, which accepts a wide matrix, or to a
    /// square kind, whose stronger `requires_square` already covers the shape.
    #[test]
    fn only_cholesky_ata_requires_rows_ge_columns() {
        use FactorizationKind::*;
        for kind in [
            Cholesky,
            LdltUnpivoted,
            LdltSbk,
            LdltTpp,
            Qr,
            LuUnpivoted,
            LuSpp,
            LuTpp,
        ] {
            assert!(
                !kind.requires_rows_ge_columns(),
                "{kind:?} should not carry the rows-at-least-columns precondition"
            );
        }
        assert!(CholeskyAtA.requires_rows_ge_columns());
    }
}
