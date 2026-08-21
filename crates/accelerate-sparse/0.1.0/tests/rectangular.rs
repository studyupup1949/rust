//! The rectangular factorizations: QR and Cholesky of the normal equations.
//!
//! Both solve the least-squares problem `min ‖A x − b‖`. QR takes the `m`-row observation vector
//! directly; CholeskyAtA takes it already reduced to `AᵀA`'s `n` rows. The small cases use a
//! hand-derived solution and the larger one a chosen `x*` with `b = A x*` in range; either way the
//! expected answer is known independently of the solver, not taken from the other path.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, DenseMut, DenseRef, FactorizationKind, SparseStructure, SymbolicFactorization,
    error::{InputError, OperandRole},
    options::{OrderMethod, SymbolicOptions},
};

// A = [[1, 0], [1, 1], [1, 2]] (m = 3, n = 2): a straight-line fit through (0,1), (1,2), (2,2).
// Stored column-major, both columns dense (the (0,1) zero is stored explicitly).
const COLUMN_STARTS: [i64; 3] = [0, 3, 6];
const ROW_INDICES: [i32; 6] = [0, 1, 2, 0, 1, 2];
const VALUES: [f64; 6] = [1.0, 1.0, 1.0, 0.0, 1.0, 2.0];

// b = [1, 2, 2]. Normal equations AᵀA = [[3, 3], [3, 5]], Aᵀb = [5, 6]; solving gives x = [7/6, 1/2].
const RHS_M: [f64; 3] = [1.0, 2.0, 2.0]; // the m-row observation vector, for QR
const RHS_N: [f64; 2] = [5.0, 6.0]; //       Aᵀb, the n-row vector for CholeskyAtA
const EXPECTED: [f64; 2] = [7.0 / 6.0, 1.0 / 2.0];

fn matrix() -> SparseStructure<'static> {
    SparseStructure::from_csc(3, 2, &COLUMN_STARTS, &ROW_INDICES, Attributes::ordinary())
        .expect("a general 3 by 2 pattern is well formed")
}

fn assert_close(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() < 1e-9, "component {i}: got {g}, want {w}");
    }
}

#[test]
fn qr_solves_a_least_squares_problem() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &matrix()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    // The right-hand side has m = 3 rows; the solution has n = 2.
    let x = factorization.solve_vec(&RHS_M).unwrap();
    assert_close(&x, &EXPECTED);
}

#[test]
fn cholesky_ata_solves_the_same_problem_from_the_normal_equations() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &matrix()).unwrap();
    assert_eq!(symbolic.effective_rows(), 3);
    assert_eq!(symbolic.effective_columns(), 2);
    assert_eq!(symbolic.right_hand_side_rows(), 2);
    assert_eq!(symbolic.solution_rows(), 2);
    assert_eq!(symbolic.in_place_operand_rows(), 3);

    let factorization = symbolic.factorize(&VALUES).unwrap();
    assert_eq!(factorization.effective_rows(), 3);
    assert_eq!(factorization.effective_columns(), 2);
    assert_eq!(factorization.right_hand_side_rows(), 2);
    assert_eq!(factorization.solution_rows(), 2);
    assert_eq!(factorization.in_place_operand_rows(), 3);

    // Here the right-hand side is already the n = 2 reduced vector Aᵀb.
    let x = factorization.solve_vec(&RHS_N).unwrap();
    assert_close(&x, &EXPECTED);
}

/// A full-row-rank wide matrix has infinitely many exact solutions, and QR returns the one with
/// minimum Euclidean norm over all `n` entries.
///
/// Here every solution is `[7-t, t, 8-t]`. Its squared norm is minimised at `t = 5`, giving
/// `[2, 5, 3]`; equivalently, that vector is orthogonal to the null-space vector `[1, -1, 1]`.
/// This also guards the shape rejection for `CholeskyAtA` against ever being widened to QR.
#[test]
fn qr_returns_the_minimum_norm_solution_for_a_wide_matrix() {
    // A = [[1, 1, 0], [0, 1, 1]] — 2 rows, 3 columns, full row rank.
    let column_starts = [0i64, 1, 3, 4];
    let row_indices = [0i32, 0, 1, 1];
    let values = [1.0f64, 1.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(2, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &structure).unwrap();
    assert_eq!(symbolic.effective_rows(), 2);
    assert_eq!(symbolic.effective_columns(), 3);
    assert_eq!(symbolic.right_hand_side_rows(), 2);
    assert_eq!(symbolic.solution_rows(), 3);
    assert_eq!(symbolic.in_place_operand_rows(), 3);

    let factorization = symbolic.factorize(&values).unwrap();
    assert_eq!(factorization.effective_rows(), 2);
    assert_eq!(factorization.effective_columns(), 3);
    assert_eq!(factorization.right_hand_side_rows(), 2);
    assert_eq!(factorization.solution_rows(), 3);
    assert_eq!(factorization.in_place_operand_rows(), 3);

    // The right-hand side has m = 2 rows; the solution has all n = 3.
    let b = [7.0, 8.0];
    let x = factorization.solve_vec(&b).unwrap();
    assert_eq!(
        x.len(),
        3,
        "the solution must carry every column, not just min(m, n)"
    );

    // The result solves the system and agrees with the independently derived minimiser.
    let residual = matvec(2, &column_starts, &row_indices, &values, &x);
    assert_close(&residual, &b);
    assert_close(&x, &[2.0, 5.0, 3.0]);

    // Orthogonality to the null space characterises the minimum-norm solution.
    let null_dot = x[0] - x[1] + x[2];
    assert!(
        null_dot.abs() < 1e-9,
        "solution is not orthogonal to the null space: dot product {null_dot}"
    );
}

/// Stored dimensions and effective scalar dimensions agree when there is no transpose and the
/// block size is one; the operand sizes then follow the factorization kind.
#[test]
fn a_rectangular_factorization_reports_its_dimensions_and_operand_sizes() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &matrix()).unwrap();
    assert_eq!(symbolic.rows(), 3);
    assert_eq!(symbolic.columns(), 2);
    assert_eq!(symbolic.effective_rows(), 3);
    assert_eq!(symbolic.effective_columns(), 2);
    assert_eq!(symbolic.right_hand_side_rows(), 3);
    assert_eq!(symbolic.solution_rows(), 2);
    assert_eq!(symbolic.in_place_operand_rows(), 3);

    let factorization = symbolic.factorize(&VALUES).unwrap();
    assert_eq!(factorization.rows(), 3);
    assert_eq!(factorization.columns(), 2);
    assert_eq!(factorization.effective_rows(), 3);
    assert_eq!(factorization.effective_columns(), 2);
    assert_eq!(factorization.right_hand_side_rows(), 3);
    assert_eq!(factorization.solution_rows(), 2);
    assert_eq!(factorization.in_place_operand_rows(), 3);
    assert_eq!(factorization.kind(), FactorizationKind::Qr);
}

/// COLAMD is the ordering for the normal equations, so it applies to the rectangular kinds. It
/// reaches the same solution as the default ordering; it changes the fill, not the answer.
#[test]
fn the_rectangular_kinds_accept_the_colamd_ordering() {
    for kind in [FactorizationKind::Qr, FactorizationKind::CholeskyAtA] {
        let symbolic = SymbolicFactorization::with_options(
            kind,
            &matrix(),
            SymbolicOptions::new().order_method(OrderMethod::Colamd),
        )
        .unwrap();
        let factorization = symbolic.factorize(&VALUES).unwrap();
        let b: &[f64] = if matches!(kind, FactorizationKind::Qr) {
            &RHS_M
        } else {
            &RHS_N
        };
        assert_close(&factorization.solve_vec(b).unwrap(), &EXPECTED);
    }
}

/// In-place QR uses one buffer of `max(m, n)` rows: the observation vector on input, the solution
/// in the leading `n` rows on output.
#[test]
fn qr_solves_in_place_in_a_max_dimension_buffer() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &matrix()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    let mut buffer = RHS_M; // 3 = max(3, 2) rows
    factorization
        .solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer[..2], &EXPECTED);
}

/// A tall `CholeskyAtA` has an `n`-row logical input and output in an `m`-row physical carrier.
/// The trailing rows are not part of either vector, but Accelerate requires them to exist.
#[test]
fn cholesky_ata_solves_in_place_in_a_max_dimension_buffer() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &matrix()).unwrap();
    assert_eq!(symbolic.right_hand_side_rows(), 2);
    assert_eq!(symbolic.solution_rows(), 2);
    assert_eq!(symbolic.in_place_operand_rows(), 3);

    let factorization = symbolic.factorize(&VALUES).unwrap();
    let mut buffer = [RHS_N[0], RHS_N[1], 0.0];
    factorization
        .solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer[..2], &EXPECTED);
}

/// Giving Accelerate only the `n` logical rows for an in-place `CholeskyAtA` solve is observed to
/// report success without writing anything. The safe API requires the `m`-row carrier before that
/// silent no-op is reachable.
#[test]
fn cholesky_ata_rejects_an_in_place_buffer_without_the_carrier_rows() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &matrix()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let mut buffer = RHS_N;
    assert_eq!(
        factorization
            .solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
            .unwrap_err()
            .input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::InPlace,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(buffer, RHS_N);
}

/// A QR right-hand side sized to the columns rather than the rows is rejected, not silently
/// mis-solved.
#[test]
fn qr_rejects_a_right_hand_side_sized_to_the_columns() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &matrix()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    let b = [1.0, 2.0]; // n = 2 rows, but QR wants m = 3
    let mut x = [0.0; 2];
    let error = factorization
        .solve_into(
            DenseRef::from_column_major_slice_with_stride(&b, 2, 1, 2).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut x, 2, 1, 2).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::RightHandSide,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(x, [0.0; 2]);
}

/// A QR solution sized to the rows rather than the columns is rejected.
#[test]
fn qr_rejects_a_solution_sized_to_the_rows() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &matrix()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    let b = RHS_M;
    let mut x = [0.0; 3]; // m = 3 rows, but the solution has n = 2
    let error = factorization
        .solve_into(
            DenseRef::from_column_major_slice_with_stride(&b, 3, 1, 3).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut x, 3, 1, 3).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::Solution,
            expected: 2,
            actual: 3,
        })
    );
    assert_eq!(x, [0.0; 3]);
}

// --- larger systems, checked against a known solution -----------------------------------------

/// `y = A x` for a general matrix in CSC, size `m`. Shares no code with the solver.
fn matvec(m: usize, cs: &[i64], ri: &[i32], v: &[f64], x: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0; m];
    for (col, x_j) in x.iter().enumerate() {
        for k in cs[col] as usize..cs[col + 1] as usize {
            y[ri[k] as usize] += v[k] * x_j;
        }
    }
    y
}

/// `Aᵀ b`, size `n` (the number of columns).
fn transpose_matvec(n: usize, cs: &[i64], ri: &[i32], v: &[f64], b: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0; n];
    for (col, y_j) in y.iter_mut().enumerate() {
        for k in cs[col] as usize..cs[col + 1] as usize {
            *y_j += v[k] * b[ri[k] as usize];
        }
    }
    y
}

/// A tall `m × n` matrix (`m > 2n`) of full column rank. The diagonal block in the first `n` rows
/// guarantees the rank; the two lower entries per column overlap with the next column's, so `AᵀA`
/// is tridiagonal rather than diagonal and the factorization does real elimination and fill —
/// otherwise the "supernodal" case would be a trivially separable one.
fn tall_full_rank(m: usize, n: usize) -> (Vec<i64>, Vec<i32>, Vec<f64>) {
    assert!(
        m > 2 * n,
        "the lower band must fit below the identity block"
    );
    let mut cs = vec![0i64];
    let mut ri = Vec::new();
    let mut v = Vec::new();
    for j in 0..n {
        ri.push(j as i32); // diagonal block: full column rank
        v.push(2.0);
        ri.push((n + j) as i32);
        v.push(1.0);
        ri.push((n + j + 1) as i32); // shared with column j+1, so AᵀA gains an off-diagonal
        v.push(0.5);
        cs.push(ri.len() as i64);
    }
    (cs, ri, v)
}

/// A consistent overdetermined system at supernodal size has an exact least-squares solution: if
/// `b = A x*` lies in the column space, both kinds recover `x*` with zero residual.
#[test]
fn least_squares_recovers_a_known_solution_at_supernodal_size() {
    let (m, n) = (300usize, 100usize);
    let (cs, ri, v) = tall_full_rank(m, n);
    let x_star: Vec<f64> = (0..n).map(|i| 1.0 + i as f64 * 0.1).collect();
    let b = matvec(m, &cs, &ri, &v, &x_star); // in range, so the LS solution is exactly x*

    let structure = SparseStructure::from_csc(m, n, &cs, &ri, Attributes::ordinary()).unwrap();

    // QR takes the m-row observation vector.
    let qr = SymbolicFactorization::new(FactorizationKind::Qr, &structure)
        .unwrap()
        .factorize(&v)
        .unwrap();
    assert_close(&qr.solve_vec(&b).unwrap(), &x_star);

    // CholeskyAtA takes the n-row reduced vector Aᵀb.
    let atb = transpose_matvec(n, &cs, &ri, &v, &b);
    let ata = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure)
        .unwrap()
        .factorize(&v)
        .unwrap();
    assert_close(&ata.solve_vec(&atb).unwrap(), &x_star);
}

/// The transpose attribute is honoured end to end: factoring the stored transpose `Aᵀ` with the
/// flag set describes the original `A`, so it must solve to what factoring `A` directly does. That
/// non-transposed solve is the oracle, so this pins the dimension swap without a separate
/// least-squares derivation.
#[test]
fn transpose_solves_the_same_system_as_the_untransposed_form() {
    // A = [[1, 0], [1, 1], [1, 2]] (3 x 2) and its explicit transpose Aᵀ (2 x 3).
    let a_cs = COLUMN_STARTS;
    let a_ri = ROW_INDICES;
    let a_v = VALUES;
    // Aᵀ stored column-major (columns are A's rows): col0=[1,0], col1=[1,1], col2=[1,2].
    let at_cs = [0i64, 2, 4, 6];
    let at_ri = [0i32, 1, 0, 1, 0, 1];
    let at_v = [1.0, 0.0, 1.0, 1.0, 1.0, 2.0];

    let direct_symbolic = SymbolicFactorization::new(
        FactorizationKind::Qr,
        &SparseStructure::from_csc(3, 2, &a_cs, &a_ri, Attributes::ordinary()).unwrap(),
    )
    .unwrap();
    let direct_factorization = direct_symbolic.factorize(&a_v).unwrap();
    let direct = direct_factorization.solve_vec(&RHS_M).unwrap();

    let transposed_symbolic = SymbolicFactorization::new(
        FactorizationKind::Qr,
        &SparseStructure::from_csc(
            2,
            3,
            &at_cs,
            &at_ri,
            Attributes::ordinary().with_transpose(true),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(transposed_symbolic.rows(), 2);
    assert_eq!(transposed_symbolic.columns(), 3);
    assert_eq!(transposed_symbolic.effective_rows(), 3);
    assert_eq!(transposed_symbolic.effective_columns(), 2);
    assert_eq!(transposed_symbolic.right_hand_side_rows(), 3);
    assert_eq!(transposed_symbolic.solution_rows(), 2);
    assert_eq!(transposed_symbolic.in_place_operand_rows(), 3);

    let transposed_factorization = transposed_symbolic.factorize(&at_v).unwrap();
    assert_eq!(transposed_factorization.rows(), 2);
    assert_eq!(transposed_factorization.columns(), 3);
    assert_eq!(transposed_factorization.effective_rows(), 3);
    assert_eq!(transposed_factorization.effective_columns(), 2);
    assert_eq!(transposed_factorization.right_hand_side_rows(), 3);
    assert_eq!(transposed_factorization.solution_rows(), 2);
    assert_eq!(transposed_factorization.in_place_operand_rows(), 3);
    let transposed = transposed_factorization.solve_vec(&RHS_M).unwrap();

    assert_close(&transposed, &direct);
}

/// Observed: neither kind reports this rank deficiency as an error. Both solutions satisfy the
/// system, but no particular vector is part of the contract because the solution is non-unique.
#[test]
fn rank_deficiency_is_not_reported() {
    // A = [[1, 1], [1, 1], [1, 1]] (3 x 2), rank 1.
    let cs = [0i64, 3, 6];
    let ri = [0i32, 1, 2, 0, 1, 2];
    let v: [f64; 6] = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let structure = SparseStructure::from_csc(3, 2, &cs, &ri, Attributes::ordinary()).unwrap();

    let qr = SymbolicFactorization::new(FactorizationKind::Qr, &structure)
        .unwrap()
        .factorize(&v)
        .unwrap();
    let qr_solution = qr.solve_vec(&[1.0, 1.0, 1.0]).unwrap();

    let ata = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure)
        .unwrap()
        .factorize(&v)
        .unwrap();
    let ata_solution = ata.solve_vec(&[3.0, 3.0]).unwrap();

    for solution in [&qr_solution, &ata_solution] {
        assert!(solution.iter().all(|entry| entry.is_finite()));
        let fitted_value: f64 = solution.iter().sum();
        assert!(
            (fitted_value - 1.0).abs() < 1e-9,
            "solution {solution:?} does not satisfy the rank-deficient system"
        );
    }
}
