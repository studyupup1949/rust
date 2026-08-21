//! Pins which half of a symmetric matrix Accelerate reads, and that it ignores the other.
//!
//! These guard library behaviour, not this crate's code. Apple documents that the unselected half
//! is implied by reflection but never says what happens to entries actually *stored* there — and
//! the one place they do document it, the coordinate-format converters, states the opposite rule:
//! such entries are transposed and summed into their mirror. Under that rule every off-diagonal
//! here would double.
//!
//! Callers rely on the observed rule to hand over a full symmetric matrix without filtering it
//! first, so if a future Accelerate unified the two policies, these tests are what notices —
//! rather than a simulation quietly changing its trajectory.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
};

// A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]] stored *whole*, both triangles, column-major:
//   column 0: rows 0,1     column 1: rows 0,1,2     column 2: rows 1,2
// Index 2 is (row 0, column 1) and index 5 is (row 1, column 2): the two entries strictly above
// the diagonal, the half Accelerate is not told to read. Their mirrors are index 1 (row 1,
// column 0) and index 4 (row 2, column 1).
const FULL_COLUMN_STARTS: [i64; 4] = [0, 2, 5, 7];
const FULL_ROW_INDICES: [i32; 7] = [0, 1, 0, 1, 2, 1, 2];
const FULL_VALUES: [f64; 7] = [4.0, 1.0, 1.0, 3.0, 1.0, 1.0, 2.0];
const UPPER_SLOTS: [usize; 2] = [2, 5];
const LOWER_MIRRORS: [usize; 2] = [1, 4];

// The same matrix as its lower triangle only.
const TRI_COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const TRI_ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];
const TRI_VALUES: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];

const RHS: [f64; 3] = [1.0, 2.0, 3.0];

fn expected() -> [f64; 3] {
    [2.0 / 9.0, 1.0 / 9.0, 13.0 / 9.0]
}

fn assert_close(got: &[f64], want: &[f64], what: &str) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!(
            (g - w).abs() <= 1e-9 * g.abs().max(1.0),
            "{what}: component {i} is {g}, expected {w}"
        );
    }
}

/// Selects how values reach the factorization. Factorization and refactorization use different
/// Accelerate entry points, and both must follow the same triangle policy.
#[derive(Debug, Clone, Copy)]
enum Path {
    Factorize,
    /// Build from a baseline first, then move onto the values under test.
    Refactor,
}

fn solve(
    column_starts: &[i64],
    row_indices: &[i32],
    values: &[f64],
    baseline: &[f64],
    kind: FactorizationKind,
    path: Path,
    rhs: &[f64],
) -> Vec<f64> {
    let n = column_starts.len() - 1;
    let structure = SparseStructure::from_csc(
        n,
        n,
        column_starts,
        row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .expect("the pattern is well formed");
    let symbolic = SymbolicFactorization::new(kind, &structure).unwrap();

    let factorization = match path {
        Path::Factorize => symbolic.factorize(values).unwrap(),
        Path::Refactor => {
            // Built from the unperturbed matrix so Cholesky starts from something it can factor,
            // then moved onto the values under test the way an iteration would.
            let mut factorization = symbolic.factorize(baseline).unwrap();
            factorization.refactor(values).unwrap();
            factorization
        }
    };
    factorization.solve_vec(rhs).unwrap()
}

fn kinds_and_paths() -> impl Iterator<Item = (FactorizationKind, Path)> {
    [FactorizationKind::Cholesky, FactorizationKind::LdltSbk]
        .into_iter()
        .flat_map(|kind| {
            [Path::Factorize, Path::Refactor]
                .into_iter()
                .map(move |p| (kind, p))
        })
}

/// The core tripwire. Three cases together identify *which* half is read, rather than merely
/// showing that some half is: perturbing the ignored half must change nothing, perturbing its
/// mirror must change the answer, and a value large enough to dominate the matrix must still be
/// ignored — a tolerance-sized perturbation could hide inside rounding, `1e7` cannot.
#[test]
fn the_upper_triangle_is_ignored_and_the_lower_is_read() {
    for (kind, path) in kinds_and_paths() {
        let what = format!("{kind:?}/{path:?}");

        // Storing the whole matrix must agree with storing one triangle.
        let baseline = solve(
            &TRI_COLUMN_STARTS,
            &TRI_ROW_INDICES,
            &TRI_VALUES,
            &TRI_VALUES,
            kind,
            path,
            &RHS,
        );
        assert_close(&baseline, &expected(), &format!("{what}: one triangle"));
        assert_close(
            &solve(
                &FULL_COLUMN_STARTS,
                &FULL_ROW_INDICES,
                &FULL_VALUES,
                &FULL_VALUES,
                kind,
                path,
                &RHS,
            ),
            &expected(),
            &format!("{what}: full storage"),
        );

        // Perturb only above the diagonal: ignored, so the answer must not move.
        let mut upper_perturbed = FULL_VALUES;
        for &k in &UPPER_SLOTS {
            upper_perturbed[k] = 1.5;
        }
        assert_close(
            &solve(
                &FULL_COLUMN_STARTS,
                &FULL_ROW_INDICES,
                &upper_perturbed,
                &FULL_VALUES,
                kind,
                path,
                &RHS,
            ),
            &expected(),
            &format!("{what}: perturbed above the diagonal"),
        );

        // The same perturbation below the diagonal *is* read, so the answer must move. Without
        // this the checks above would also pass for a solve that ignored its values entirely,
        // including a refactor that silently did nothing.
        let mut lower_perturbed = FULL_VALUES;
        for &k in &LOWER_MIRRORS {
            lower_perturbed[k] = 1.5;
        }
        let moved = solve(
            &FULL_COLUMN_STARTS,
            &FULL_ROW_INDICES,
            &lower_perturbed,
            &FULL_VALUES,
            kind,
            path,
            &RHS,
        );
        let shift = moved
            .iter()
            .zip(expected())
            .map(|(g, w)| (g - w).abs())
            .fold(0.0_f64, f64::max);
        assert!(
            shift > 1e-6,
            "{what}: perturbing the lower triangle must change the solution, else these checks \
             cannot tell a read half from an ignored one; largest shift {shift}"
        );

        // A value that would dominate the matrix if it were read at all.
        let mut upper_garbage = FULL_VALUES;
        for &k in &UPPER_SLOTS {
            upper_garbage[k] = 1.0e7;
        }
        assert_close(
            &solve(
                &FULL_COLUMN_STARTS,
                &FULL_ROW_INDICES,
                &upper_garbage,
                &FULL_VALUES,
                kind,
                path,
                &RHS,
            ),
            &expected(),
            &format!("{what}: garbage above the diagonal"),
        );
    }
}

/// The documented shortcut: a symmetric matrix's row-major arrays are already valid column-major
/// arrays, so a caller holding CSR can pass it as CSC unchanged.
///
/// The arrays are built here by rows, independently of the column-major constants above, and
/// asserted to be identical to them — which is *why* the shortcut works, and makes this a test of
/// the claim rather than a restatement of it.
#[test]
fn a_full_symmetric_matrix_in_row_major_order_is_already_column_major() {
    // A = [[4, 1, 0], [1, 3, 1], [0, 1, 2]] walked by rows.
    let rows: [&[(i32, f64)]; 3] = [
        &[(0, 4.0), (1, 1.0)],
        &[(0, 1.0), (1, 3.0), (2, 1.0)],
        &[(1, 1.0), (2, 2.0)],
    ];
    let mut row_starts = vec![0i64];
    let mut column_indices = Vec::new();
    let mut values = Vec::new();
    for row in rows {
        for &(column, value) in row {
            column_indices.push(column);
            values.push(value);
        }
        row_starts.push(column_indices.len() as i64);
    }

    assert_eq!(
        row_starts, FULL_COLUMN_STARTS,
        "row starts are column starts"
    );
    assert_eq!(
        column_indices, FULL_ROW_INDICES,
        "column indices are row indices"
    );
    assert_eq!(values, FULL_VALUES);

    // Handed over as CSC without transposing or filtering anything.
    let x = solve(
        &row_starts,
        &column_indices,
        &values,
        &values,
        FactorizationKind::Cholesky,
        Path::Factorize,
        &RHS,
    );
    assert_close(&x, &expected(), "row-major arrays read as column-major");
}

/// A 2-D five-point Laplacian on a `k` by `k` grid: `n = k*k`, diagonally dominant hence positive
/// definite. Returned as full storage and as the lower triangle, plus the positions of the
/// ignored (row < column) and read off-diagonal (row > column) slots.
#[allow(clippy::type_complexity)]
fn laplacian(
    k: usize,
) -> (
    (Vec<i64>, Vec<i32>, Vec<f64>),
    (Vec<i64>, Vec<i32>, Vec<f64>),
    Vec<usize>,
    Vec<usize>,
) {
    let n = k * k;
    let (mut full_cs, mut full_ri, mut full_v) = (vec![0i64], Vec::new(), Vec::new());
    let (mut tri_cs, mut tri_ri, mut tri_v) = (vec![0i64], Vec::new(), Vec::new());
    let (mut ignored, mut read_off_diagonal) = (Vec::new(), Vec::new());

    for column in 0..n {
        let (r, c) = (column / k, column % k);
        let mut entries: Vec<(usize, f64)> = Vec::with_capacity(5);
        if r > 0 {
            entries.push((column - k, -1.0));
        }
        if c > 0 {
            entries.push((column - 1, -1.0));
        }
        entries.push((column, 4.0));
        if c + 1 < k {
            entries.push((column + 1, -1.0));
        }
        if r + 1 < k {
            entries.push((column + k, -1.0));
        }

        for &(row, value) in &entries {
            if row < column {
                ignored.push(full_ri.len());
            } else {
                if row > column {
                    read_off_diagonal.push(full_ri.len());
                }
                tri_ri.push(row as i32);
                tri_v.push(value);
            }
            full_ri.push(row as i32);
            full_v.push(value);
        }
        full_cs.push(full_ri.len() as i64);
        tri_cs.push(tri_ri.len() as i64);
    }

    (
        (full_cs, full_ri, full_v),
        (tri_cs, tri_ri, tri_v),
        ignored,
        read_off_diagonal,
    )
}

/// The same policy at a size that actually reaches Accelerate's supernodal machinery.
///
/// A 3 by 3 matrix is below any plausible supernodal or blocking threshold, so a policy change
/// confined to the supernodal value scatter would sail past the small test while silently halving
/// every off-diagonal in a real Hessian. This one is `n = 900` with a five-point stencil, close in
/// shape to what a consumer actually hands over.
///
/// Checked to a tolerance rather than bitwise on purpose: Accelerate's factorization is
/// multithreaded and not reproducible run to run unless `VECLIB_MAXIMUM_THREADS` is pinned, and a
/// test must not depend on the environment doing that.
#[test]
fn the_triangle_policy_holds_at_supernodal_size() {
    let k = 30;
    let n = k * k;
    let ((full_cs, full_ri, full_v), (tri_cs, tri_ri, tri_v), ignored, read_off_diagonal) =
        laplacian(k);
    assert_eq!(
        ignored.len(),
        2 * k * (k - 1),
        "one ignored slot per off-diagonal pair"
    );

    let rhs: Vec<f64> = (0..n).map(|i| ((i % 7) as f64) - 3.0).collect();

    // Everything Accelerate was not told to read, set to a value that would dominate the matrix.
    let mut garbage = full_v.clone();
    for &k in &ignored {
        garbage[k] = 1.0e9;
    }
    // The mirrored half, which it does read.
    let mut perturbed_below = full_v.clone();
    for &k in &read_off_diagonal {
        perturbed_below[k] = -0.25;
    }

    for (kind, path) in kinds_and_paths() {
        let what = format!("{kind:?}/{path:?}");
        let baseline = solve(&tri_cs, &tri_ri, &tri_v, &tri_v, kind, path, &rhs);

        assert_close(
            &solve(&full_cs, &full_ri, &full_v, &full_v, kind, path, &rhs),
            &baseline,
            &format!("{what}: full storage against one triangle"),
        );
        assert_close(
            &solve(&full_cs, &full_ri, &garbage, &full_v, kind, path, &rhs),
            &baseline,
            &format!("{what}: garbage above the diagonal"),
        );

        let moved = solve(
            &full_cs,
            &full_ri,
            &perturbed_below,
            &full_v,
            kind,
            path,
            &rhs,
        );
        let shift = moved
            .iter()
            .zip(&baseline)
            .map(|(g, w)| (g - w).abs())
            .fold(0.0_f64, f64::max);
        assert!(
            shift > 1e-6,
            "{what}: perturbing below the diagonal must change the solution; largest shift {shift}"
        );
    }
}
