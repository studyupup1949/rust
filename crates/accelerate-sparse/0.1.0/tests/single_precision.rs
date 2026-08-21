//! The `f32` path, against the same systems the `f64` tests use.
//!
//! These exist to check the element type is carried end to end — the analysis is shared between
//! the two, while the numeric factorization, the dense operands and Accelerate's default
//! tolerances are all per type, so a wrong entry point would show up as a wrong answer rather than
//! as a compile error.
//!
//! Tolerances here are stated relative to `f32::EPSILON` (about 1.2e-7) rather than to the `1e-9`
//! the double-precision tests use.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, DenseMut, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
    error::Status,
};

// A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]], lower triangle, column-major. With b = [1, 2, 3] the
// solution is x = [2/9, 1/9, 13/9].
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];
const VALUES: [f32; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];
const RHS: [f32; 3] = [1.0, 2.0, 3.0];

fn expected() -> [f32; 3] {
    [2.0 / 9.0, 1.0 / 9.0, 13.0 / 9.0]
}

fn spd() -> SparseStructure<'static> {
    SparseStructure::from_csc(
        3,
        3,
        &COLUMN_STARTS,
        &ROW_INDICES,
        Attributes::symmetric(Triangle::Lower),
    )
    .expect("the pattern is well formed")
}

fn assert_close(got: &[f32], want: &[f32]) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() < 1e-6, "component {i}: got {g}, want {w}");
    }
}

/// Multiplies a symmetric matrix stored as one triangle by `x`, reflecting each off-diagonal.
/// Accumulates in `f64` so the oracle is not limited by the precision it is checking.
fn symmetric_matvec(
    n: usize,
    column_starts: &[i64],
    row_indices: &[i32],
    values: &[f32],
    x: &[f32],
) -> Vec<f64> {
    let mut y = vec![0.0; n];
    for column in 0..n {
        for k in column_starts[column] as usize..column_starts[column + 1] as usize {
            let row = row_indices[k] as usize;
            y[row] += f64::from(values[k]) * f64::from(x[column]);
            if row != column {
                y[column] += f64::from(values[k]) * f64::from(x[row]);
            }
        }
    }
    y
}

/// `‖A x − b‖ / ‖b‖`, both norms taken in `f64`.
fn relative_residual(
    n: usize,
    column_starts: &[i64],
    row_indices: &[i32],
    values: &[f32],
    x: &[f32],
    b: &[f32],
) -> f64 {
    let ax = symmetric_matvec(n, column_starts, row_indices, values, x);
    let norm = |v: &[f64]| v.iter().map(|e| e * e).sum::<f64>().sqrt();
    let residual: Vec<f64> = ax.iter().zip(b).map(|(ax, b)| ax - f64::from(*b)).collect();
    norm(&residual) / norm(&b.iter().map(|e| f64::from(*e)).collect::<Vec<_>>())
}

/// A symmetric tridiagonal matrix, lower triangle. Strictly diagonally dominant when
/// `diagonal > 2 * |off|`, hence positive definite.
fn tridiagonal(n: usize, diagonal: f32, off: f32) -> (Vec<i64>, Vec<i32>, Vec<f32>) {
    let mut column_starts = vec![0i64];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for column in 0..n {
        row_indices.push(column as i32);
        values.push(diagonal);
        if column + 1 < n {
            row_indices.push(column as i32 + 1);
            values.push(off);
        }
        column_starts.push(row_indices.len() as i64);
    }
    (column_starts, row_indices, values)
}

#[test]
fn cholesky_solves_a_positive_definite_system() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());
}

#[test]
fn ldlt_solves_the_same_system() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltSbk, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());
}

/// The indefinite case, as in double precision: Cholesky reports a status and the fallback works.
#[test]
fn cholesky_fails_where_ldlt_succeeds() {
    // A = [[0, 1], [1, 0]], eigenvalues ±1. With b = [1, 1] the solution is x = [1, 1].
    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [0.0f32, 1.0, 0.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let error = symbolic.factorize(&values).unwrap_err();
    assert_eq!(error.status(), Some(Status::FactorizationFailed));

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltSbk, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    assert_close(&factorization.solve_vec(&[1.0, 1.0]).unwrap(), &[1.0, 1.0]);
}

/// Refactor and the in-place solve, which reach different shim entry points than the pair above.
#[test]
fn refactor_and_solve_in_place() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let mut factorization = symbolic.factorize(&VALUES).unwrap();

    let doubled: Vec<f32> = VALUES.iter().map(|v| 2.0 * v).collect();
    factorization.refactor(&doubled).unwrap();

    let mut xb = RHS;
    factorization
        .solve_in_place(DenseMut::from_vector(&mut xb).unwrap())
        .unwrap();

    // Doubling A halves the solution.
    let halved: Vec<f32> = expected().iter().map(|e| e / 2.0).collect();
    assert_close(&xb, &halved);
}

/// One analysis serves both element types: the pattern is all it depends on.
#[test]
fn one_analysis_serves_both_element_types() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();

    let single = symbolic.factorize(&VALUES).unwrap();
    let double = symbolic.factorize(&[4.0f64, 1.0, 3.0, 1.0, 2.0]).unwrap();

    let single = single.solve_vec(&RHS).unwrap();
    let double = double.solve_vec(&[1.0f64, 2.0, 3.0]).unwrap();

    let promoted: Vec<f64> = single.iter().map(|e| f64::from(*e)).collect();
    for (i, (s, d)) in promoted.iter().zip(&double).enumerate() {
        assert!(
            (s - d).abs() < 1e-6,
            "component {i}: single precision {s}, double precision {d}"
        );
    }
}

/// A size that reaches Accelerate's supernodal machinery. The observed relative residual is about
/// 1.3e-7, one `f32::EPSILON`; the bound is two orders of magnitude above that, so rounding alone
/// does not reach it.
#[test]
fn residual_is_small_on_a_larger_system() {
    let n = 400;
    let (column_starts, row_indices, values) = tridiagonal(n, 4.0, -1.0);
    let structure = SparseStructure::from_csc(
        n,
        n,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();
    let b: Vec<f32> = (0..n).map(|i| (i as f32) - 3.5).collect();

    for kind in [FactorizationKind::Cholesky, FactorizationKind::LdltSbk] {
        let symbolic = SymbolicFactorization::new(kind, &structure).unwrap();
        let factorization = symbolic.factorize(&values).unwrap();
        let x = factorization.solve_vec(&b).unwrap();
        let r = relative_residual(n, &column_starts, &row_indices, &values, &x, &b);
        assert!(r < 1e-5, "{kind:?}: relative residual {r} is too large");
    }
}

/// The analysis reports a size per element type; neither is derived from the other here.
#[test]
fn factor_size_is_reported_for_both_element_types() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    assert!(symbolic.factor_size::<f32>() > 0);
    assert!(symbolic.factor_size::<f64>() > 0);
}
