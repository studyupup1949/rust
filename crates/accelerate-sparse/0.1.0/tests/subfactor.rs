//! Extracting and applying the pieces of a factorization.
//!
//! Correctness is checked by composition: the pieces are put back together and the product compared
//! against a matrix-vector product written here, which shares no code with the solver. The
//! composition matters — Accelerate's Cholesky is `A = P L Lᵀ Pᵀ`, so `L Lᵀ` alone is not `A`, and a
//! test that assumed otherwise would fail for the right reason but tell the wrong story.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, DenseMut, DenseRef, FactorizationKind, SparseStructure, SubfactorKind,
    SymbolicFactorization, Triangle,
    error::{InputError, OperandRole, Status},
};

// A = [[4,1,0],[1,3,1],[0,1,2]], symmetric positive definite, lower triangle.
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];
const VALUES: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];
const RHS: [f64; 3] = [1.0, 2.0, 3.0];

// A = [[1,2,0],[2,1,2],[0,2,1]], symmetric indefinite, same pattern.
const INDEFINITE: [f64; 5] = [1.0, 2.0, 1.0, 2.0, 1.0];

fn spd() -> SparseStructure<'static> {
    SparseStructure::from_csc(
        3,
        3,
        &COLUMN_STARTS,
        &ROW_INDICES,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap()
}

/// `A x` for the symmetric matrix stored above, reflecting each off-diagonal entry.
fn matvec(values: &[f64], x: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0; 3];
    for column in 0..3 {
        for k in COLUMN_STARTS[column] as usize..COLUMN_STARTS[column + 1] as usize {
            let row = ROW_INDICES[k] as usize;
            y[row] += values[k] * x[column];
            if row != column {
                y[column] += values[k] * x[row];
            }
        }
    }
    y
}

fn assert_close(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() < 1e-9, "component {i}: got {g}, want {w}");
    }
}

/// `P L Lᵀ Pᵀ x` must equal `A x`. This is the whole point of the feature: the pieces really are
/// the factors of the matrix, and they compose.
#[test]
fn the_pieces_of_a_cholesky_compose_back_to_the_matrix() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();

    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let p = factorization.subfactor(SubfactorKind::P).unwrap();
    let lt = l.transpose().unwrap();
    let pt = p.transpose().unwrap();

    let x = [1.0f64, 2.0, 3.0];
    let step = pt.multiply_vec(&x).unwrap();
    let step = lt.multiply_vec(&step).unwrap();
    let step = l.multiply_vec(&step).unwrap();
    let got = p.multiply_vec(&step).unwrap();

    assert_close(&got, &matvec(&VALUES, &x));
}

/// Without the permutation the same pieces do not reproduce the matrix, so the test above is
/// checking the composition rather than restating whatever Accelerate returned.
#[test]
fn the_factor_alone_does_not_reproduce_the_matrix() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let lt = l.transpose().unwrap();

    // The composition check requires a genuine permutation that is not its own inverse. Assert
    // those prerequisites so a symmetric ordering cannot make the check vacuous.
    let p = factorization.subfactor(SubfactorKind::P).unwrap();
    let pt = p.transpose().unwrap();
    let e1 = [1.0f64, 0.0, 0.0];
    assert!(
        p.multiply_vec(&e1)
            .unwrap()
            .iter()
            .zip(&pt.multiply_vec(&e1).unwrap())
            .any(|(a, b)| (a - b).abs() > 1e-12),
        "the ordering is its own inverse, so the two associations cannot be told apart"
    );

    let x = [1.0f64, 2.0, 3.0];
    let got = l.multiply_vec(&lt.multiply_vec(&x).unwrap()).unwrap();
    let want = matvec(&VALUES, &x);
    assert!(
        got.iter().zip(&want).any(|(g, w)| (g - w).abs() > 1e-9),
        "L Lᵀ reproduced A, so the permutation is not part of this factorization after all"
    );
}

/// The half-solve applied twice is the full solve: solving with `PLPᵀ` and then with its transpose
/// must land on the same answer as the factorization's own solve.
#[test]
fn the_half_solve_applied_twice_is_the_full_solve() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let full = factorization.solve_vec(&RHS).unwrap();

    let half = factorization.subfactor(SubfactorKind::Plps).unwrap();
    let half_t = half.transpose().unwrap();
    let once = half.solve_vec(&RHS).unwrap();
    let twice = half_t.solve_vec(&once).unwrap();

    assert_close(&twice, &full);
}

/// `Q` of a QR factorization is the thin `m × n` factor, not the square one, and it is orthogonal:
/// `Qᵀ (Q x) = x`. The shape is the part worth pinning — a square `Q` would need different operands.
#[test]
fn q_of_a_qr_factorization_is_thin_and_orthogonal() {
    // 4x3 overdetermined, stored whole.
    let column_starts = [0i64, 4, 7, 9];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(4, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    let q = factorization.subfactor(SubfactorKind::Q).unwrap();
    assert_eq!(
        (q.rows(), q.columns()),
        (4, 3),
        "Q should be the thin factor"
    );
    let qt = q.transpose().unwrap();
    assert_eq!((qt.rows(), qt.columns()), (3, 4));

    let x = [1.0f64, -2.0, 0.5];
    let qx = q.multiply_vec(&x).unwrap();
    assert_eq!(qx.len(), 4);
    let back = qt.multiply_vec(&qx).unwrap();
    assert_close(&back, &x);

    // Only `Qᵀ Q` is the identity for a thin factor; `Q Qᵀ` is a rank-3 projector on four
    // dimensions. Without this, the orthogonality check above would pass for a square Q too.
    let e0 = [1.0f64, 0.0, 0.0, 0.0];
    let projected = q.multiply_vec(&qt.multiply_vec(&e0).unwrap()).unwrap();
    assert!(
        projected.iter().zip(&e0).any(|(p, e)| (p - e).abs() > 1e-9),
        "Q Qᵀ acted as the identity, so Q is not the thin factor: {projected:?}"
    );

    // R is square, of the column count.
    let r = factorization.subfactor(SubfactorKind::R).unwrap();
    assert_eq!((r.rows(), r.columns()), (3, 3));
}

/// Every pairing of a kind with a piece either works or is refused, and the table says which.
///
/// Covers the rectangular and LU parents as well as the symmetric ones, since that is where the
/// table's interesting entries are: `Q` means the orthogonal factor for QR and the column
/// permutation for LU, `R` belongs to QR and `CholeskyAtA` alone, and the two LU scalings to LU.
#[test]
fn only_the_applicable_pieces_can_be_extracted() {
    let pieces = [
        SubfactorKind::P,
        SubfactorKind::S,
        SubfactorKind::L,
        SubfactorKind::D,
        SubfactorKind::Plps,
        SubfactorKind::Q,
        SubfactorKind::R,
        SubfactorKind::Rp,
        SubfactorKind::Sr,
        SubfactorKind::Sc,
    ];

    // Symmetric parents. Cholesky needs a definite matrix; the LDLᵀ kinds are given the indefinite
    // one so that unpivoted does not fail on a small pivot.
    for kind in [
        FactorizationKind::Cholesky,
        FactorizationKind::LdltUnpivoted,
        FactorizationKind::LdltSbk,
        FactorizationKind::LdltTpp,
    ] {
        let values = if kind == FactorizationKind::Cholesky {
            VALUES
        } else {
            INDEFINITE
        };
        let symbolic = SymbolicFactorization::new(kind, &spd()).unwrap();
        let factorization = symbolic.factorize(&values).unwrap();
        check_pairings(&factorization, kind, &pieces);
    }

    // Rectangular parents, 4x3 overdetermined.
    let column_starts = [0i64, 4, 7, 9];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let rect =
        SparseStructure::from_csc(4, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    for kind in [FactorizationKind::Qr, FactorizationKind::CholeskyAtA] {
        let symbolic = SymbolicFactorization::new(kind, &rect).unwrap();
        let factorization = symbolic.factorize(&values).unwrap();
        check_pairings(&factorization, kind, &pieces);
    }

    // Unsymmetric square parents. A = [[2,1,0],[0,3,1],[1,0,4]], stored whole.
    let lu_starts = [0i64, 2, 4, 6];
    let lu_rows = [0i32, 2, 0, 1, 1, 2];
    let lu_values = [2.0f64, 1.0, 1.0, 3.0, 1.0, 4.0];
    let square =
        SparseStructure::from_csc(3, 3, &lu_starts, &lu_rows, Attributes::ordinary()).unwrap();
    for kind in [
        FactorizationKind::LuUnpivoted,
        FactorizationKind::LuSpp,
        FactorizationKind::LuTpp,
    ] {
        let symbolic = SymbolicFactorization::new(kind, &square).unwrap();
        let factorization = symbolic.factorize(&lu_values).unwrap();
        check_pairings(&factorization, kind, &pieces);
    }
}

fn check_pairings<T: accelerate_sparse::Scalar>(
    factorization: &accelerate_sparse::Factorization<T>,
    kind: FactorizationKind,
    pieces: &[SubfactorKind],
) {
    for &piece in pieces {
        let result = factorization.subfactor(piece);
        assert_eq!(
            result.is_ok(),
            piece.applies_to(kind),
            "{kind:?} + {piece:?}: the table and Accelerate disagree"
        );
        match result {
            Ok(subfactor) => assert_eq!(subfactor.kind(), piece),
            Err(error) => assert_eq!(
                error.input(),
                Some(&InputError::SubfactorUnavailable {
                    subfactor: piece,
                    factorization: kind,
                })
            ),
        }
    }
}

/// Neither half-solve piece can be multiplied by, and the two guards earn their place differently.
/// Letting `Plps` reach Accelerate aborts the process; letting `Rp` reach it returns success with the
/// destination untouched, a silent wrong answer rather than a crash. Both are refused here.
#[test]
fn the_half_solve_pieces_refuse_multiplication() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let plps = factorization.subfactor(SubfactorKind::Plps).unwrap();

    // Rp belongs to the rectangular kinds, so it needs a parent of its own.
    let rect_starts = [0i64, 4, 7, 9];
    let rect_rows = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let rect_values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let rect =
        SparseStructure::from_csc(4, 3, &rect_starts, &rect_rows, Attributes::ordinary()).unwrap();
    let rect_symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &rect).unwrap();
    let rect_factorization = rect_symbolic.factorize(&rect_values).unwrap();
    let rp = rect_factorization.subfactor(SubfactorKind::Rp).unwrap();

    for piece in [&plps, &rp] {
        assert!(!piece.kind().supports_multiply(), "{:?}", piece.kind());

        let mut out = [0.0f64; 3];
        let error = piece
            .multiply_into(
                DenseRef::from_vector(&RHS).unwrap(),
                DenseMut::from_vector(&mut out).unwrap(),
            )
            .unwrap_err();
        assert_eq!(
            error.input(),
            Some(&InputError::MultiplyUnsupported {
                subfactor: piece.kind(),
            }),
            "{:?}",
            piece.kind()
        );
        assert_eq!(out, [0.0; 3]);

        let mut buffer = RHS;
        let error = piece
            .multiply_in_place(DenseMut::from_vector(&mut buffer).unwrap())
            .unwrap_err();
        assert_eq!(
            error.input(),
            Some(&InputError::MultiplyUnsupported {
                subfactor: piece.kind(),
            }),
            "{:?}",
            piece.kind()
        );
        assert_eq!(buffer, RHS);

        // Solving with it is the supported direction.
        piece.solve_vec(&RHS).unwrap();
    }
}

/// A piece of an `LDLᵀ` factorization in single precision, so the per-element-type dispatch is
/// exercised on both sides.
#[test]
fn subfactors_work_in_single_precision() {
    let values: Vec<f32> = INDEFINITE.iter().map(|v| *v as f32).collect();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &spd()).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    let d = factorization.subfactor(SubfactorKind::D).unwrap();
    assert_eq!((d.rows(), d.columns()), (3, 3));
    let y = d.multiply_vec(&[1.0f32, 0.0, 0.0]).unwrap();
    assert_eq!(y.len(), 3);
    // Applying D and then solving with it is the identity for any invertible D, which is the most
    // that can be claimed here: D is block diagonal rather than diagonal, so the components beyond
    // the first are where a 2x2 pivot shows, and they are checked rather than discarded.
    let back = d.solve_vec(&y).unwrap();
    for (component, expected) in back.iter().zip([1.0f32, 0.0, 0.0]) {
        assert!(
            (component - expected).abs() < 1e-5,
            "D then D⁻¹ should be the identity, got {back:?}"
        );
    }
}

/// The block form counts a subfactor's shape in scalars, not blocks — unlike the factorization it
/// came from, whose dimensions are in blocks.
#[test]
fn a_block_subfactor_is_measured_in_scalars() {
    // Two 2x2 diagonal blocks: 2 blocks, 4 scalars.
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let values = [4.0f64, 1.0, 1.0, 3.0, 5.0, 1.0, 1.0, 4.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap()
    .with_block_size(2)
    .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    assert_eq!(
        factorization.columns(),
        2,
        "the factorization counts blocks"
    );

    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    assert_eq!(
        (l.rows(), l.columns()),
        (4, 4),
        "the subfactor counts scalars"
    );
    let y = l.multiply_vec(&[1.0, 0.0, 0.0, 0.0]).unwrap();
    assert_eq!(y.len(), 4);
}

/// Workspace is reported rather than demanded. `Q` of a QR factorization needs substantially more
/// than the other pieces, which is the case that would justify exposing a caller-supplied buffer.
#[test]
fn workspace_is_reported_and_scales_with_the_right_hand_sides() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();

    let one = l.workspace_required(1);
    let four = l.workspace_required(4);
    assert!(one > 0, "an application needs scratch space");
    assert!(
        four > one,
        "scratch space grows with the right-hand side count"
    );
}

#[test]
fn a_wrong_operand_shape_is_rejected_before_reaching_accelerate() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let short = [1.0f64, 2.0];
    assert_eq!(
        l.multiply_vec(&short).unwrap_err().input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::Multiplicand,
            expected: 3,
            actual: 2,
        })
    );
}

// ---------------------------------------------------------------------------------------------
// Properties that hold whatever Accelerate's internal identities are
//
// The composition tests above need to know how the pieces multiply back together. These do not: a
// permutation is a permutation and a block-diagonal factor is block diagonal whatever order it
// appears in, and each property is checked by applying the piece to the basis vectors.
// ---------------------------------------------------------------------------------------------

/// Every column of a permutation has exactly one entry, equal to one.
#[test]
fn the_permutation_piece_really_is_a_permutation() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    assert_is_permutation(&factorization.subfactor(SubfactorKind::P).unwrap(), "P");
}

/// Applies `piece` to each basis vector and checks the images are the columns of a permutation:
/// every entry zero or one, exactly one one per column, and every row reached once.
fn assert_is_permutation(piece: &accelerate_sparse::Subfactor<'_, f64>, name: &str) {
    assert!(
        piece.columns() > 0 && piece.rows() > 0,
        "{name} reported an empty shape, which would make every check below vacuous"
    );
    let mut hit = vec![0usize; piece.rows()];
    for column in 0..piece.columns() {
        let mut e = vec![0.0f64; piece.columns()];
        e[column] = 1.0;
        let image = piece.multiply_vec(&e).unwrap();

        let ones: Vec<usize> = image
            .iter()
            .enumerate()
            .filter(|(_, v)| (**v - 1.0).abs() < 1e-12)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(
            ones.len(),
            1,
            "column {column} of {name} is not a unit column: {image:?}"
        );
        assert!(
            image
                .iter()
                .all(|v| v.abs() < 1e-12 || (v - 1.0).abs() < 1e-12),
            "column {column} of {name} has an entry that is neither zero nor one: {image:?}"
        );
        hit[ones[0]] += 1;
    }
    assert!(
        hit.iter().all(|count| *count == 1),
        "{name} is not a bijection: {hit:?}"
    );
}

/// For an LU factorization, `Q` is the column reordering rather than an orthogonal factor. It is
/// extracted by the pairing tests but applied nowhere else, so its shape and its meaning rest on
/// nothing — this applies it and checks it is a permutation, as `P` is checked above.
///
/// The matrix is an arrow, chosen because it forces the analysis to reorder: for the small dense
/// examples used elsewhere in this file both pieces come back as the identity, which every check
/// below would pass against a no-op. Each is therefore asserted to be something other than the
/// identity first.
#[test]
fn the_lu_reordering_pieces_are_permutations() {
    const N: usize = 6;
    let mut column_starts = vec![0i64];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for column in 0..N {
        if column == 0 {
            for row in 0..N {
                row_indices.push(row as i32);
                values.push(if row == 0 { 10.0 } else { 1.0 });
            }
        } else {
            row_indices.push(0);
            values.push(1.0);
            row_indices.push(column as i32);
            values.push(5.0);
        }
        column_starts.push(row_indices.len() as i64);
    }

    let structure =
        SparseStructure::from_csc(N, N, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LuTpp, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    for piece in [SubfactorKind::P, SubfactorKind::Q] {
        let sub = factorization.subfactor(piece).unwrap();
        assert_eq!(
            (sub.rows(), sub.columns()),
            (N, N),
            "{piece:?} of an LU factorization should be square"
        );
        assert!(
            !is_identity(&sub),
            "{piece:?} is the identity, so nothing below distinguishes it from a no-op"
        );
        assert_is_permutation(&sub, &format!("{piece:?}"));
    }
}

/// Whether `piece` maps every basis vector to itself.
fn is_identity(piece: &accelerate_sparse::Subfactor<'_, f64>) -> bool {
    (0..piece.columns()).all(|column| {
        let mut e = vec![0.0f64; piece.columns()];
        e[column] = 1.0;
        piece
            .multiply_vec(&e)
            .unwrap()
            .iter()
            .enumerate()
            .all(|(row, value)| {
                if row == column {
                    (value - 1.0).abs() < 1e-12
                } else {
                    value.abs() < 1e-12
                }
            })
    })
}

/// The `D` of an `LDLᵀ` factorization is symmetric and block diagonal with blocks of at most two.
///
/// Not diagonal: threshold partial pivoting takes 2×2 pivots, and it takes one for the values used
/// here — and, measured separately, for positive-definite ones too — so a caller must not assume `D`
/// is a scaling. The values factored below are the indefinite ones. The property checked is
/// the structural one — nothing beyond the first off-diagonal, symmetric, and no two adjacent
/// off-diagonal entries both non-zero, which would mean a block of three.
#[test]
fn the_d_piece_is_symmetric_block_diagonal() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &spd()).unwrap();
    let factorization = symbolic.factorize(&INDEFINITE).unwrap();
    let d = factorization.subfactor(SubfactorKind::D).unwrap();

    let n = d.columns();
    assert_eq!(
        (d.rows(), n),
        (3, 3),
        "an unexpected shape would make every check below vacuous"
    );
    let mut entries = vec![vec![0.0f64; n]; n];
    for column in 0..n {
        let mut e = vec![0.0f64; n];
        e[column] = 1.0;
        let image = d.multiply_vec(&e).unwrap();
        for (row, value) in image.iter().enumerate() {
            entries[row][column] = *value;
            assert!(
                row.abs_diff(column) <= 1 || value.abs() < 1e-12,
                "D has an entry at ({row}, {column}) outside a 2x2 block: {value}"
            );
        }
    }

    for (row, row_entries) in entries.iter().enumerate() {
        for (column, value) in row_entries.iter().enumerate() {
            assert!(
                (value - entries[column][row]).abs() < 1e-12,
                "D is not symmetric at ({row}, {column})"
            );
        }
    }

    let mut coupled = false;
    for row in 1..n {
        let joins_previous = entries[row][row - 1].abs() > 1e-12;
        let joins_next = row + 1 < n && entries[row + 1][row].abs() > 1e-12;
        assert!(
            !(joins_previous && joins_next),
            "row {row} of D is coupled on both sides, which would be a block of three"
        );
        coupled |= joins_previous;
    }

    // The point of the test: a caller must not treat D as a scaling. Were it diagonal here, every
    // check above would hold and the claim would be untested.
    assert!(
        coupled,
        "D is diagonal for these values, so this test no longer shows what it says it does"
    );
}

/// LU's two scaling pieces are diagonal and round-trip.
///
/// This pins their shape, availability and diagonality and nothing more. Accelerate returns the
/// identity for both under the default options — *observed*, and consistent with Apple documenting
/// no scaling as LU's default — so the round-trip below would hold even if the operations did
/// nothing. Making them non-trivial needs `scalingMethod`, which this crate does not expose; if it
/// ever does, strengthen this to assert they are not the identity.
#[test]
fn the_lu_scaling_pieces_are_diagonal_and_round_trip() {
    let starts = [0i64, 2, 4, 6];
    let rows = [0i32, 2, 0, 1, 1, 2];
    let values = [2.0f64, 1.0, 1.0, 3.0, 1.0, 4.0];
    let structure =
        SparseStructure::from_csc(3, 3, &starts, &rows, Attributes::ordinary()).unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LuTpp, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    for piece in [SubfactorKind::Sr, SubfactorKind::Sc] {
        let scaling = factorization.subfactor(piece).unwrap();
        assert_eq!((scaling.rows(), scaling.columns()), (3, 3));

        for column in 0..3 {
            let mut e = vec![0.0f64; 3];
            e[column] = 1.0;
            let image = scaling.multiply_vec(&e).unwrap();
            for (row, value) in image.iter().enumerate() {
                if row != column {
                    assert!(
                        value.abs() < 1e-12,
                        "{piece:?} has an off-diagonal entry at ({row}, {column}): {value}"
                    );
                }
            }
        }

        // Applying the scaling and then solving with it returns the original vector.
        let x = [1.0f64, -2.0, 3.5];
        let scaled = scaling.multiply_vec(&x).unwrap();
        let back = scaling.solve_vec(&scaled).unwrap();
        assert_close(&back, &x);
    }
}

/// The normal-equations half-solve applied twice is that factorization's full solve, as the
/// symmetric half-solve is for Cholesky.
#[test]
fn the_normal_equations_half_solve_applied_twice_is_the_full_solve() {
    let column_starts = [0i64, 4, 7, 9];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(4, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    // CholeskyAtA takes a right-hand side already reduced to n rows, so its own solve is the
    // normal-equations solve that the half-solve pieces must reproduce.
    let rhs = [1.0f64, 2.0, 3.0];
    let full = factorization.solve_vec(&rhs).unwrap();

    // The transpose comes first: `AᵀA = Rᵀ R` puts `Rᵀ` against the right-hand side. Both orders
    // are checked because the wrong one returns a plausible-looking vector.
    let half = factorization.subfactor(SubfactorKind::Rp).unwrap();
    let half_t = half.transpose().unwrap();
    let once = half_t.solve_vec(&rhs).unwrap();
    let twice = half.solve_vec(&once).unwrap();

    assert_close(&twice, &full);
}

/// The in-place forms agree with the out-of-place ones, for both operations.
#[test]
fn the_in_place_forms_agree_with_the_out_of_place_ones() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();

    let solved = l.solve_vec(&RHS).unwrap();
    let mut buffer = RHS;
    l.solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer, &solved);

    let multiplied = l.multiply_vec(&RHS).unwrap();
    let mut buffer = RHS;
    l.multiply_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer, &multiplied);
}

/// Several right-hand sides at once, through a strided view into one buffer, must match solving
/// each on its own.
#[test]
fn multiple_right_hand_sides_match_solving_each_alone() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();

    let first = [1.0f64, 2.0, 3.0];
    let second = [-1.0f64, 0.5, 4.0];
    let alone = [l.solve_vec(&first).unwrap(), l.solve_vec(&second).unwrap()];

    // Two columns in one buffer, with a stride wider than the rows so the padding is skipped.
    let mut storage = vec![0.0f64; 8];
    storage[0..3].copy_from_slice(&first);
    storage[4..7].copy_from_slice(&second);
    let b = DenseRef::from_column_major_slice_with_stride(&storage, 3, 2, 4).unwrap();
    let mut out = vec![0.0f64; 8];
    l.solve_into(
        b,
        DenseMut::from_column_major_slice_with_stride(&mut out, 3, 2, 4).unwrap(),
    )
    .unwrap();

    assert_close(&out[0..3], &alone[0]);
    assert_close(&out[4..7], &alone[1]);
}

/// `Q` of a QR factorization needs more scratch space than the triangular pieces, which is the
/// observation behind reporting the figure at all.
#[test]
fn the_orthogonal_factor_needs_the_most_workspace() {
    let column_starts = [0i64, 4, 7, 9];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(4, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    let q = factorization.subfactor(SubfactorKind::Q).unwrap();
    let r = factorization.subfactor(SubfactorKind::R).unwrap();
    assert!(
        q.workspace_required(1) > r.workspace_required(1),
        "Q needs {} bytes and R needs {}",
        q.workspace_required(1),
        r.workspace_required(1)
    );
}

/// Exercises concurrent applications through a shared subfactor.
#[test]
fn a_subfactor_can_be_applied_from_many_threads() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let expected = l.multiply_vec(&RHS).unwrap();

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..250 {
                    assert_close(&l.multiply_vec(&RHS).unwrap(), &expected);
                }
            });
        }
    });
}

#[test]
fn a_short_solution_operand_is_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let mut out = [0.0f64; 2];
    let error = l
        .solve_into(
            DenseRef::from_vector(&RHS).unwrap(),
            DenseMut::from_vector(&mut out).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::Solution,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(out, [0.0; 2]);
}

#[test]
fn solve_operands_disagreeing_on_the_column_count_are_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let b = vec![0.0f64; 6];
    let mut x = vec![0.0f64; 3];
    let error = l
        .solve_into(
            DenseRef::from_column_major_slice_with_stride(&b, 3, 2, 3).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut x, 3, 1, 3).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandColumns {
            first: OperandRole::RightHandSide,
            first_columns: 2,
            second: OperandRole::Solution,
            second_columns: 1,
        })
    );
    assert_eq!(x, vec![0.0; 3]);
}

#[test]
fn a_short_in_place_operand_is_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let mut buffer = [0.0f64; 2];
    assert_eq!(
        l.solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
            .unwrap_err()
            .input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::InPlace,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(buffer, [0.0; 2]);
}

#[test]
fn a_short_right_hand_side_is_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let short = [1.0f64, 2.0];
    let mut out = [0.0f64; 3];
    let error = l
        .solve_into(
            DenseRef::from_vector(&short).unwrap(),
            DenseMut::from_vector(&mut out).unwrap(),
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
    assert_eq!(out, [0.0; 3]);
}

#[test]
fn a_short_product_operand_is_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let mut out = [0.0f64; 2];
    let error = l
        .multiply_into(
            DenseRef::from_vector(&RHS).unwrap(),
            DenseMut::from_vector(&mut out).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::Product,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(out, [0.0; 2]);
}

#[test]
fn a_short_in_place_multiply_operand_is_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let mut buffer = [0.0f64; 2];
    assert_eq!(
        l.multiply_in_place(DenseMut::from_vector(&mut buffer).unwrap())
            .unwrap_err()
            .input(),
        Some(&InputError::OperandRows {
            operand: OperandRole::InPlace,
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(buffer, [0.0; 2]);
}

#[test]
fn multiply_operands_disagreeing_on_the_column_count_are_rejected() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let l = factorization.subfactor(SubfactorKind::L).unwrap();
    let x = vec![0.0f64; 6];
    let mut y = vec![0.0f64; 3];
    let error = l
        .multiply_into(
            DenseRef::from_column_major_slice_with_stride(&x, 3, 2, 3).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut y, 3, 1, 3).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandColumns {
            first: OperandRole::Multiplicand,
            first_columns: 2,
            second: OperandRole::Product,
            second_columns: 1,
        })
    );
    assert_eq!(y, vec![0.0; 3]);
}

/// A parent with more columns than rows. The pieces are sized from the *smaller* dimension, so a
/// wide parent is not the transpose of a tall one, and getting it wrong is not an error but a
/// plausible-looking answer: Accelerate rejects the operand and leaves the buffer as it found it.
///
/// `A = [[1,1,0],[0,1,1]]` is 2x3 of full row rank. Every piece is 2x2 except `Q`, which is the thin
/// `3x2`.
#[test]
fn a_wide_parent_sizes_its_pieces_from_the_smaller_dimension() {
    let column_starts = [0i64, 1, 3, 4];
    let row_indices = [0i32, 0, 1, 1];
    let values = [1.0f64, 1.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(2, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();

    for (piece, expected) in [
        (SubfactorKind::P, (2, 2)),
        (SubfactorKind::Q, (3, 2)),
        (SubfactorKind::R, (2, 2)),
        (SubfactorKind::Rp, (2, 2)),
    ] {
        let sub = factorization.subfactor(piece).unwrap();
        assert_eq!(
            (sub.rows(), sub.columns()),
            expected,
            "{piece:?} of a 2x3 QR has the wrong shape"
        );
    }

    // The shapes are right, so an application succeeds and writes a result. Under the previous
    // model every one of these returned Ok with the buffer untouched.
    let q = factorization.subfactor(SubfactorKind::Q).unwrap();
    let qx = q.multiply_vec(&[1.0, -2.0]).unwrap();
    assert_eq!(qx.len(), 3);
    assert!(
        qx.iter().any(|v| v.abs() > 1e-12),
        "applying Q wrote nothing: {qx:?}"
    );
    let back = q.transpose().unwrap().multiply_vec(&qx).unwrap();
    assert_close(&back, &[1.0, -2.0]);
}

// A wide `CholeskyAtA` parent once probed here to show its pieces are also square of the smaller
// dimension. That matrix is now rejected at analysis — its normal equations are singular, so the
// factorization has no solution to return — and cannot be built through the safe API. The
// wide-parent piece sizing it checked (`P`, `R`, `Rp` at the smaller dimension) is pinned by the QR
// case above, whose wide matrix stays valid; the rejection itself is covered in `tests/validation`.

/// A parent declared transposed reports the same shapes as the same matrix stored the other way
/// round.
///
/// **Observed:** The shape rule uses the larger and smaller dimensions and is invariant to the
/// parent's transpose.
#[test]
fn a_transposed_parent_reports_the_same_shapes() {
    let tall_starts = [0i64, 4, 7, 9];
    let tall_rows = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let tall_values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];

    let plain =
        SparseStructure::from_csc(4, 3, &tall_starts, &tall_rows, Attributes::ordinary()).unwrap();
    let transposed = SparseStructure::from_csc(
        4,
        3,
        &tall_starts,
        &tall_rows,
        Attributes::ordinary().with_transpose(true),
    )
    .unwrap();

    for piece in [SubfactorKind::P, SubfactorKind::Q, SubfactorKind::R] {
        let mut shapes = Vec::new();
        for structure in [&plain, &transposed] {
            let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, structure).unwrap();
            let factorization = symbolic.factorize(&tall_values).unwrap();
            let sub = factorization.subfactor(piece).unwrap();
            shapes.push((sub.rows(), sub.columns()));
        }
        assert_eq!(
            shapes[0], shapes[1],
            "{piece:?} changed shape when the parent was declared transposed"
        );
    }
}

/// A factorization whose last attempt failed refuses to hand out pieces, and that refusal is what
/// keeps a process abort out of reach: Accelerate traps if asked for a piece of a failed
/// factorization, and the error callback does not intercept it.
///
/// Reached through Cholesky, since the parent must be a kind that can fail while still supplying
/// pieces on success.
#[test]
fn a_failed_factorization_refuses_to_supply_pieces() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let mut factorization = symbolic.factorize(&VALUES).unwrap();
    factorization.subfactor(SubfactorKind::L).unwrap();

    // Negating the diagonal makes the matrix negative definite (Gershgorin puts every eigenvalue
    // in (-5, -1)), which Cholesky cannot factor.
    let indefinite = [-4.0, 1.0, -3.0, 1.0, -2.0];
    factorization.refactor(&indefinite).unwrap_err();
    assert!(!factorization.is_factored());

    for piece in [SubfactorKind::L, SubfactorKind::P, SubfactorKind::Plps] {
        let error = factorization.subfactor(piece).unwrap_err();
        assert_eq!(
            error.status(),
            Some(Status::NotFactored),
            "{piece:?} should be refused while the factorization is unfactored"
        );
    }

    // Recovering the factorization makes the pieces available again.
    factorization.refactor(&VALUES).unwrap();
    factorization.subfactor(SubfactorKind::L).unwrap();
}

/// Extracting pieces from one shared factorization on many threads at once.
///
/// This is the case the reference-count lock exists for: extraction retains the factorization and
/// dropping the piece releases it, both reachable through a shared reference, and Accelerate does not
/// document that count as atomic.
///
/// It is a smoke test, not a detector. Removing the lock does not make it fail — the race, if there
/// is one, is inside Accelerate where no sanitizer this crate can run will see it. What this catches
/// is the ordinary kind of breakage: a deadlock, a double release, a handle that stops working.
#[test]
fn pieces_can_be_extracted_from_one_factorization_on_many_threads() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&VALUES).unwrap();
    let expected = factorization
        .subfactor(SubfactorKind::L)
        .unwrap()
        .multiply_vec(&RHS)
        .unwrap();

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..500 {
                    let l = factorization.subfactor(SubfactorKind::L).unwrap();
                    let lt = l.transpose().unwrap();
                    assert_close(&l.multiply_vec(&RHS).unwrap(), &expected);
                    assert_eq!(lt.rows(), 3);
                    // Both handles are released here, on this thread.
                }
            });
        }
    });

    // The factorization is still usable after four thousand retain/release cycles against it.
    assert_close(
        &factorization
            .subfactor(SubfactorKind::L)
            .unwrap()
            .multiply_vec(&RHS)
            .unwrap(),
        &expected,
    );
}

/// The in-place forms on a piece that is not square, where `max(rows, columns)` differs from both.
///
/// Every other in-place test uses a square piece, so the operand rule is satisfied by the row count
/// and the column count alike and an implementation using either would pass. `Q` of a 4x3 QR is the
/// one piece where they differ: solving consumes four rows and produces three, multiplying consumes
/// three and produces four, and one buffer of four serves both.
#[test]
fn the_in_place_forms_handle_a_non_square_piece() {
    let column_starts = [0i64, 4, 7, 9];
    let row_indices = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    let structure =
        SparseStructure::from_csc(4, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Qr, &structure).unwrap();
    let factorization = symbolic.factorize(&values).unwrap();
    let q = factorization.subfactor(SubfactorKind::Q).unwrap();
    assert_eq!((q.rows(), q.columns()), (4, 3));

    // Solving: four rows in, three out, in a buffer of four.
    let b = [1.0f64, 2.0, 3.0, 4.0];
    let expected = q.solve_vec(&b).unwrap();
    assert_eq!(expected.len(), 3);
    let mut buffer = b;
    q.solve_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer[..3], &expected);

    // Multiplying: three rows in, four out, in the same buffer size.
    let x = [1.0f64, -2.0, 0.5];
    let expected = q.multiply_vec(&x).unwrap();
    assert_eq!(expected.len(), 4);
    let mut buffer = [x[0], x[1], x[2], 0.0];
    q.multiply_in_place(DenseMut::from_vector(&mut buffer).unwrap())
        .unwrap();
    assert_close(&buffer, &expected);
}
