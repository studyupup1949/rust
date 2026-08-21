//! Correctness of the factor, refactor and solve cycle.
//!
//! Solutions are checked either against a value derived by hand or against the residual
//! `‖A x − b‖`, computed by a matrix-vector product written here. That product shares no code with
//! the solver, so it is an honest oracle rather than a restatement of the same assumption.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, DenseMut, DenseRef, FactorizationKind, SparseStructure, SymbolicFactorization,
    Triangle,
    error::Status,
    options::{NumericOptions, OrderMethod, SymbolicOptions},
};

// A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]], lower triangle, column-major. With b = [1, 2, 3] the
// solution is x = [2/9, 1/9, 13/9].
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];
const VALUES: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];
const RHS: [f64; 3] = [1.0, 2.0, 3.0];

fn expected() -> [f64; 3] {
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

fn assert_close(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() < 1e-9, "component {i}: got {g}, want {w}");
    }
}

/// Multiplies a symmetric matrix stored as one triangle by `x`, reflecting each off-diagonal.
fn symmetric_matvec(
    n: usize,
    column_starts: &[i64],
    row_indices: &[i32],
    values: &[f64],
    x: &[f64],
) -> Vec<f64> {
    let mut y = vec![0.0; n];
    for column in 0..n {
        for k in column_starts[column] as usize..column_starts[column + 1] as usize {
            let row = row_indices[k] as usize;
            y[row] += values[k] * x[column];
            if row != column {
                y[column] += values[k] * x[row];
            }
        }
    }
    y
}

fn residual(
    n: usize,
    column_starts: &[i64],
    row_indices: &[i32],
    values: &[f64],
    x: &[f64],
    b: &[f64],
) -> f64 {
    symmetric_matvec(n, column_starts, row_indices, values, x)
        .iter()
        .zip(b)
        .map(|(ax, b)| (ax - b) * (ax - b))
        .sum::<f64>()
        .sqrt()
}

/// A symmetric tridiagonal matrix, lower triangle. Strictly diagonally dominant when
/// `diagonal > 2 * |off|`, hence positive definite.
fn tridiagonal(n: usize, diagonal: f64, off: f64) -> (Vec<i64>, Vec<i32>, Vec<f64>) {
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

/// Cholesky reports a status for an indefinite matrix, and the `LDLᵀ` fallback succeeds.
#[test]
fn cholesky_fails_where_ldlt_succeeds() {
    // A = [[0, 1], [1, 0]], eigenvalues ±1. With b = [1, 1] the solution is x = [1, 1].
    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [0.0, 1.0, 0.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();

    // The pattern alone is fine; it is the numeric phase that rejects the matrix.
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let error = symbolic.factorize(&values).unwrap_err();
    assert_eq!(error.status(), Some(Status::FactorizationFailed));

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltSbk, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    assert_close(&factorization.solve_vec(&[1.0, 1.0]).unwrap(), &[1.0, 1.0]);
}

/// The unpivoted and TPP variants reach the same solution as the others on a positive-definite
/// system, where pivoting choice does not matter.
#[test]
fn the_other_ldlt_variants_solve_the_positive_definite_system() {
    for kind in [FactorizationKind::LdltUnpivoted, FactorizationKind::LdltTpp] {
        let symbolic = SymbolicFactorization::new(kind, &spd()).unwrap();
        let factorization = symbolic.factorize(&VALUES).unwrap();
        assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());
    }
}

/// Unpivoted `LDLᵀ` factors a symmetric indefinite matrix that Cholesky rejects, provided no pivot
/// is too small. `A = [[1, 2], [2, 1]]` has eigenvalues 3 and −1; with `b = [1, 1]` the solution is
/// `[1/3, 1/3]`. This distinguishes the two selectors beyond their shim constants.
///
/// `inertia.rs` distinguishes TPP from SBK because both agree on these small solve cases.
#[test]
fn unpivoted_factors_an_indefinite_matrix_cholesky_rejects() {
    // A = [[1, 2], [2, 1]], lower triangle.
    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [1.0, 2.0, 1.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    assert_eq!(
        symbolic.factorize(&values).unwrap_err().status(),
        Some(Status::FactorizationFailed)
    );

    let symbolic =
        SymbolicFactorization::new(FactorizationKind::LdltUnpivoted, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    assert_close(
        &factorization.solve_vec(&[1.0, 1.0]).unwrap(),
        &[1.0 / 3.0, 1.0 / 3.0],
    );
}

/// TPP handles a symmetric indefinite matrix as SBK does. Unpivoted cannot: a zero diagonal
/// leaves it no 1×1 pivot, which it reports as a numeric failure rather than trapping. This is
/// the distinction between the two new variants.
#[test]
fn tpp_handles_indefinite_where_unpivoted_fails() {
    // A = [[0, 1], [1, 0]], eigenvalues ±1. With b = [1, 1] the solution is x = [1, 1].
    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [0.0, 1.0, 0.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();

    let symbolic =
        SymbolicFactorization::new(FactorizationKind::LdltUnpivoted, &structure).unwrap();
    let error = symbolic.factorize(&values).unwrap_err();
    assert_eq!(error.status(), Some(Status::FactorizationFailed));

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    assert_close(&factorization.solve_vec(&[1.0, 1.0]).unwrap(), &[1.0, 1.0]);
}

#[test]
fn refactor_reuses_the_analysis_and_changes_the_answer() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let mut factorization = symbolic.factorize(&VALUES).unwrap();

    // Refactoring onto 2A must halve the solution. A refactor that silently did nothing would
    // leave it unchanged and pass a weaker check.
    let doubled: Vec<f64> = VALUES.iter().map(|v| 2.0 * v).collect();
    factorization.refactor(&doubled).unwrap();
    let halved: Vec<f64> = expected().iter().map(|e| e / 2.0).collect();
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &halved);
}

/// A failed refactor remains retryable: solving is refused, but another refactor is allowed.
#[test]
fn a_failed_refactor_is_retryable_and_refuses_to_solve() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let mut factorization = symbolic.factorize(&VALUES).unwrap();
    assert!(factorization.is_factored());

    // Negating the diagonal makes the matrix negative definite (Gershgorin puts every eigenvalue
    // in (-5, -1)), which Cholesky cannot factor.
    let indefinite = [-4.0, 1.0, -3.0, 1.0, -2.0];
    let error = factorization.refactor(&indefinite).unwrap_err();
    assert_eq!(error.status(), Some(Status::FactorizationFailed));
    assert!(!factorization.is_factored());

    let refused = factorization.solve_vec(&RHS).unwrap_err();
    assert_eq!(refused.status(), Some(Status::NotFactored));

    // The recovery: regularise and refactor again against the retained analysis.
    factorization.refactor(&VALUES).unwrap();
    assert!(factorization.is_factored());
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());
}

#[test]
fn a_factorization_outlives_its_analysis() {
    let mut factorization = {
        let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
        symbolic.factorize(&VALUES).unwrap()
    };
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());

    let doubled: Vec<f64> = VALUES.iter().map(|v| 2.0 * v).collect();
    factorization.refactor(&doubled).unwrap();
    let halved: Vec<f64> = expected().iter().map(|e| e / 2.0).collect();
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &halved);
}

#[test]
fn solve_in_place_agrees_with_the_out_of_place_form() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltSbk, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    let mut xb = RHS;
    factorization
        .solve_in_place(DenseMut::from_vector(&mut xb).unwrap())
        .unwrap();
    assert_close(&xb, &factorization.solve_vec(&RHS).unwrap());
}

/// Several right-hand sides at once, with a column stride wider than the matrix so the strided
/// path is exercised rather than the contiguous one.
#[test]
fn multiple_right_hand_sides_with_padding() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    // Two columns of three rows each, stored with a stride of five: the two trailing elements of
    // every column are padding Accelerate must skip.
    let stride = 5;
    let mut b = vec![0.0; stride * 2];
    b[0..3].copy_from_slice(&RHS);
    b[stride..stride + 3].copy_from_slice(&[2.0, 4.0, 6.0]);
    let mut x = vec![0.0; stride * 2];

    factorization
        .solve_into(
            DenseRef::from_column_major_slice_with_stride(&b, 3, 2, stride).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut x, 3, 2, stride).unwrap(),
        )
        .unwrap();

    assert_close(&x[0..3], &expected());
    // The second right-hand side is twice the first, so its solution is too.
    let doubled: Vec<f64> = expected().iter().map(|e| 2.0 * e).collect();
    assert_close(&x[stride..stride + 3], &doubled);
    assert_eq!(&x[3..stride], &[0.0, 0.0], "padding must not be written");
}

/// A size that reaches Accelerate's supernodal machinery, checked by residual rather than against
/// a precomputed solution.
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
    let b: Vec<f64> = (0..n).map(|i| (i as f64) - 3.5).collect();

    for kind in [FactorizationKind::Cholesky, FactorizationKind::LdltSbk] {
        let symbolic = SymbolicFactorization::new(kind, &structure).unwrap();
        let factorization = symbolic.factorize(&values).unwrap();
        let x = factorization.solve_vec(&b).unwrap();
        let r = residual(n, &column_starts, &row_indices, &values, &x, &b);
        assert!(r < 1e-9, "{kind:?}: residual {r} is too large");
    }
}

/// Every ordering must reach the same solution; they change the fill, not the answer.
#[test]
fn orderings_agree() {
    let n = 60;
    let (column_starts, row_indices, values) = tridiagonal(n, 4.0, -1.0);
    let structure = SparseStructure::from_csc(
        n,
        n,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();
    let b: Vec<f64> = (0..n).map(|i| (i % 5) as f64 - 2.0).collect();

    let mut solutions = Vec::new();
    for order in [OrderMethod::Default, OrderMethod::Amd, OrderMethod::Metis] {
        let symbolic = SymbolicFactorization::with_options(
            FactorizationKind::Cholesky,
            &structure,
            SymbolicOptions::new().order_method(order),
        )
        .unwrap();
        let factorization = symbolic.factorize(&values).unwrap();
        solutions.push(factorization.solve_vec(&b).unwrap());
    }

    for solution in &solutions[1..] {
        assert_close(solution, &solutions[0]);
    }
}

/// A custom option must round-trip through the numeric factorization — the non-null options path
/// through the shim, which the default solve does not take — and still produce a correct solve.
/// That the option struct starts from Accelerate's defaults rather than a zeroed one is checked
/// deterministically by the unit tests on `NumericOptions::to_raw`.
#[test]
fn numeric_options_are_accepted() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltSbk, &spd()).unwrap();
    let factorization = accelerate_sparse::Factorization::with_options(
        &symbolic,
        &VALUES,
        NumericOptions::new().pivot_tolerance(0.5),
    )
    .unwrap();
    assert_close(&factorization.solve_vec(&RHS).unwrap(), &expected());
}

#[test]
fn factor_size_is_reported() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    assert!(symbolic.factor_size::<f64>() > 0);
    assert_eq!(symbolic.value_count(), 5);
}
