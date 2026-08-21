//! Counting eigenvalues by sign off an LDLᵀ-TPP factorization.
//!
//! The small cases have eigenvalues in closed form, so the expected inertia is arithmetic rather
//! than another factorization's opinion. The larger ones reach Accelerate's supernodal path,
//! where that is no longer possible, and get their expected answer from a construction whose
//! inertia is known before any factorization happens: Sylvester's law of inertia, the saddle-point
//! result, and Gershgorin's discs, cross-checked by a Sturm sequence.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, Factorization, FactorizationKind, Inertia, SparseStructure, SymbolicFactorization,
    Triangle, error::Status, options::NumericOptions,
};

/// Whether this build can answer the query at all.
///
/// False when the SDK did not provide `SparseGetInertia`, or when `ACCSP_DISABLE_INERTIA`
/// suppressed it. This crate has no build script, so the capability arrives from the sys crate as
/// a constant rather than as a `#[cfg]`, and the tests below check it at run time. The
/// unsupported configuration is not left untested: it has a case of its own at the end, and
/// `the_capability_is_present_unless_it_was_switched_off` keeps a build that lost the capability by
/// accident from passing as one that never had it.
const SUPPORTED: bool = accelerate_sparse::sys::HAS_INERTIA;

/// Every other test here returns early when the capability is absent, which means a probe that
/// silently fails would leave the whole file passing while asserting nothing about the feature.
///
/// The build script reports "unavailable" for every error it meets — an unwritable probe file, a
/// compiler that will not start — so that failure mode is reachable without anyone noticing. Only
/// two things legitimately turn the capability off: an SDK too old to declare the function, which
/// this host is not, or the environment variable. Anything else is a regression in the gate.
#[test]
fn the_capability_is_present_unless_it_was_switched_off() {
    assert!(
        SUPPORTED || std::env::var_os("ACCSP_DISABLE_INERTIA").is_some(),
        "the build reported no inertia support, and ACCSP_DISABLE_INERTIA is not set: the SDK \
         probe in the sys crate's build script has regressed"
    );
}

// Pattern shared by every matrix below: symmetric tridiagonal, 3x3, lower triangle.
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];

// A = [[4,1,0],[1,3,1],[0,1,2]], positive definite.
const DEFINITE: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];

// A = [[1,2,0],[2,1,2],[0,2,1]], a symmetric tridiagonal Toeplitz matrix whose eigenvalues are
// 1 + 4cos(kπ/4) for k = 1, 2, 3 — approximately 3.83, 1 and -1.83.
const INDEFINITE: [f64; 5] = [1.0, 2.0, 1.0, 2.0, 1.0];

fn structure() -> SparseStructure<'static> {
    SparseStructure::from_csc(
        3,
        3,
        &COLUMN_STARTS,
        &ROW_INDICES,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap()
}

fn inertia_of(
    kind: FactorizationKind,
    values: &[f64],
) -> Result<Inertia, accelerate_sparse::error::Error> {
    let symbolic = SymbolicFactorization::new(kind, &structure()).unwrap();
    symbolic.factorize(values).unwrap().inertia()
}

#[test]
fn a_positive_definite_matrix_has_only_positive_eigenvalues() {
    if !SUPPORTED {
        return;
    }

    let inertia = inertia_of(FactorizationKind::LdltTpp, &DEFINITE).unwrap();
    assert_eq!(
        inertia,
        Inertia {
            positive: 3,
            zero: 0,
            negative: 0
        }
    );
}

#[test]
fn an_indefinite_matrix_counts_both_signs() {
    if !SUPPORTED {
        return;
    }

    let inertia = inertia_of(FactorizationKind::LdltTpp, &INDEFINITE).unwrap();
    assert_eq!(
        inertia,
        Inertia {
            positive: 2,
            zero: 0,
            negative: 1
        }
    );
    assert_eq!(
        inertia.positive + inertia.zero + inertia.negative,
        3,
        "every eigenvalue must be accounted for"
    );
}

/// Negating a positive-definite matrix negates every eigenvalue, so the counts must swap. This
/// separates a real query from a constant: the pattern, the kind and the dimension are unchanged.
#[test]
fn negating_the_matrix_swaps_the_counts() {
    if !SUPPORTED {
        return;
    }

    let negated: Vec<f64> = DEFINITE.iter().map(|v| -v).collect();
    let inertia = inertia_of(FactorizationKind::LdltTpp, &negated).unwrap();
    assert_eq!(
        inertia,
        Inertia {
            positive: 0,
            zero: 0,
            negative: 3
        }
    );
}

/// A singular matrix has a zero eigenvalue, and an exactly representable one is reported as a
/// zero pivot rather than as a small positive or negative one. `[[1,1],[1,1]]` has eigenvalues 2
/// and 0.
#[test]
fn an_exactly_singular_matrix_reports_a_zero_pivot() {
    if !SUPPORTED {
        return;
    }

    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [1.0f64, 1.0, 1.0];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure).unwrap();
    let inertia = symbolic.factorize(&values).unwrap().inertia().unwrap();
    assert_eq!(
        inertia,
        Inertia {
            positive: 1,
            zero: 1,
            negative: 0
        }
    );
}

/// The counts follow the values, not the analysis: one factorization refactored from a definite
/// matrix to an indefinite one reports each in turn.
#[test]
fn inertia_follows_a_refactor() {
    if !SUPPORTED {
        return;
    }

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure()).unwrap();
    let mut factorization = symbolic.factorize(&DEFINITE).unwrap();
    assert_eq!(factorization.inertia().unwrap().negative, 0);

    factorization.refactor(&INDEFINITE).unwrap();
    assert_eq!(factorization.inertia().unwrap().negative, 1);
}

#[test]
fn inertia_is_available_in_single_precision() {
    if !SUPPORTED {
        return;
    }

    let values: Vec<f32> = INDEFINITE.iter().map(|v| *v as f32).collect();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure()).unwrap();
    let inertia = symbolic.factorize(&values).unwrap().inertia().unwrap();
    assert_eq!(
        inertia,
        Inertia {
            positive: 2,
            zero: 0,
            negative: 1
        }
    );
}

/// Every other kind is refused with a parameter error rather than a wrong answer. Accelerate
/// supplies the explanation through the error callback, making this one of the few paths where a
/// diagnostic is attached.
#[test]
fn every_other_kind_is_refused() {
    if !SUPPORTED {
        return;
    }

    for kind in [
        FactorizationKind::Cholesky,
        FactorizationKind::LdltUnpivoted,
        FactorizationKind::LdltSbk,
    ] {
        let error = inertia_of(kind, &DEFINITE).unwrap_err();
        assert_eq!(
            error.status(),
            Some(Status::ParameterError),
            "{kind:?} should refuse an inertia query"
        );
        assert!(
            error.detail().is_some(),
            "{kind:?} should carry Accelerate's explanation"
        );
    }
}

/// An unfactored handle is reported as such before the kind is considered, so a caller is told
/// what actually went wrong rather than being sent after the wrong question.
///
/// The two conditions can only be separated on a kind that both fails and refuses inertia, hence
/// Cholesky here. TPP itself has not been observed to fail at all: refactoring one onto a matrix
/// of NaNs, of infinities, or of zeros leaves it factored and reporting success, so the
/// unfactored path cannot be reached through the kind that accepts this query.
#[test]
fn an_unfactored_handle_is_reported_before_the_wrong_kind() {
    if !SUPPORTED {
        return;
    }

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure()).unwrap();
    let mut factorization = symbolic.factorize(&DEFINITE).unwrap();

    // Cholesky refuses an inertia query, but only once it holds a factorization at all.
    assert_eq!(
        factorization.inertia().unwrap_err().status(),
        Some(Status::ParameterError)
    );

    // Negating the diagonal makes the matrix negative definite (Gershgorin puts every eigenvalue
    // in (-5, -1)), which Cholesky cannot factor.
    let indefinite = [-4.0, 1.0, -3.0, 1.0, -2.0];
    factorization.refactor(&indefinite).unwrap_err();
    assert!(!factorization.is_factored());

    assert_eq!(
        factorization.inertia().unwrap_err().status(),
        Some(Status::NotFactored)
    );
}

// ---------------------------------------------------------------------------------------------
// Oracles at supernodal size
//
// The cases above are all a few rows across, which is well inside the size where Accelerate takes
// its small-matrix path, so none of them exercise what a real problem would. These are sized past
// the point where that changes — the COLAMD hang recorded elsewhere in this crate puts it around
// sixty — and their expected answers can no longer be eigenvalues worked out by hand. Each matrix
// is built instead so that its inertia is known before it is factored.
// ---------------------------------------------------------------------------------------------

/// A deterministic generator, seeded so any failure reproduces. The values fill a pattern and
/// carry no meaning of their own.
fn next(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 33) as f64) / 2_147_483_648.0 * 2.0 - 1.0
}

/// Packs the lower triangle of a dense symmetric matrix into CSC, keeping every diagonal entry
/// and every non-zero below it.
fn lower_triangle_csc(dense: &[Vec<f64>]) -> (Vec<i64>, Vec<i32>, Vec<f64>) {
    let n = dense.len();
    let mut column_starts = Vec::with_capacity(n + 1);
    let mut row_indices = Vec::new();
    let mut values = Vec::new();

    for j in 0..n {
        column_starts.push(row_indices.len() as i64);
        for (i, row) in dense.iter().enumerate().skip(j) {
            if i == j || row[j] != 0.0 {
                row_indices.push(i as i32);
                values.push(row[j]);
            }
        }
    }
    column_starts.push(row_indices.len() as i64);

    (column_starts, row_indices, values)
}

/// Factors a lower-triangle CSC matrix as LDLᵀ-TPP and reports its inertia.
fn inertia_of_csc(n: usize, column_starts: &[i64], row_indices: &[i32], values: &[f64]) -> Inertia {
    let structure = SparseStructure::from_csc(
        n,
        n,
        column_starts,
        row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure).unwrap();
    symbolic.factorize(values).unwrap().inertia().unwrap()
}

/// Sylvester's law of inertia: `CᵀDC` has the inertia of `D` for any non-singular `C`. That
/// yields a large sparse indefinite matrix whose answer is known without computing an eigenvalue.
///
/// `C` is unit upper triangular, so it is non-singular whatever its off-diagonal entries, and
/// banded, so `CᵀDC` stays sparse.
///
/// The answer stays a property of the matrix rather than of the factorization's tolerances because
/// the result's eigenvalues are bounded away from zero. `D`'s entries being bounded away from zero
/// is not on its own enough for that: the bound is `min|D|` times the square of `C`'s smallest
/// singular value, so it depends on `C`'s conditioning too. Both hold here, with a wide margin —
/// `min|D|` is 0.5 and the assembled matrix's smallest eigenvalue is about 0.198.
#[test]
fn a_congruent_matrix_has_the_inertia_of_its_diagonal() {
    if !SUPPORTED {
        return;
    }

    const N: usize = 80;
    let mut seed = 0x5eed_1234;

    let d: Vec<f64> = (0..N)
        .map(|i| {
            let magnitude = 0.5 + 2.0 * (i % 7) as f64 / 7.0;
            if i % 3 == 0 { -magnitude } else { magnitude }
        })
        .collect();

    let mut c = vec![vec![0.0; N]; N];
    for (i, row) in c.iter_mut().enumerate() {
        row[i] = 1.0;
        for entry in row.iter_mut().take((i + 4).min(N)).skip(i + 1) {
            *entry = 0.5 * next(&mut seed);
        }
    }

    let mut a = vec![vec![0.0; N]; N];
    for i in 0..N {
        for j in 0..N {
            a[i][j] = (0..N).map(|k| c[k][i] * d[k] * c[k][j]).sum();
        }
    }

    let (column_starts, row_indices, values) = lower_triangle_csc(&a);
    let inertia = inertia_of_csc(N, &column_starts, &row_indices, &values);

    let negative = d.iter().filter(|v| **v < 0.0).count();
    assert_eq!(
        inertia,
        Inertia {
            positive: N - negative,
            zero: 0,
            negative
        }
    );
}

/// A saddle-point matrix `[[H, Bᵀ], [B, 0]]` with `H` positive definite and `B` of full row rank
/// has exactly one negative eigenvalue per constraint, whatever the values.
#[test]
fn a_saddle_point_system_has_one_negative_eigenvalue_per_constraint() {
    if !SUPPORTED {
        return;
    }

    const PRIMAL: usize = 48;
    const CONSTRAINTS: usize = 24;
    let size = PRIMAL + CONSTRAINTS;

    let mut a = vec![vec![0.0; size]; size];

    // H: the 1-D Laplacian, positive definite.
    for i in 0..PRIMAL {
        a[i][i] = 4.0;
        if i + 1 < PRIMAL {
            a[i][i + 1] = -1.0;
            a[i + 1][i] = -1.0;
        }
    }

    // B: each constraint couples two neighbouring variables. Every row has its leading entry in a
    // distinct column, so the rows are independent and B has full row rank.
    for r in 0..CONSTRAINTS {
        let row = PRIMAL + r;
        a[row][r] = 1.0;
        a[r][row] = 1.0;
        a[row][r + 1] = -1.0;
        a[r + 1][row] = -1.0;
    }

    // The trailing block stays zero. Its diagonal is stored anyway, so no column is empty.
    let (column_starts, row_indices, values) = lower_triangle_csc(&a);
    let inertia = inertia_of_csc(size, &column_starts, &row_indices, &values);

    assert_eq!(
        inertia,
        Inertia {
            positive: PRIMAL,
            zero: 0,
            negative: CONSTRAINTS
        }
    );
}

/// A second opinion on a tridiagonal matrix, from a Sturm sequence computed here rather than from
/// Accelerate.
///
/// The matrix is diagonally dominant with mixed signs: each diagonal entry is at least 2 in
/// magnitude and each Gershgorin radius at most 1, so no disc reaches zero and the sign of every
/// eigenvalue follows the sign of its diagonal entry. That keeps the expected answer independent
/// of any tolerance, and gives two derivations to agree with the factorization instead of one.
#[test]
fn a_sturm_sequence_agrees_on_a_dominant_tridiagonal_matrix() {
    if !SUPPORTED {
        return;
    }

    const N: usize = 64;
    let mut seed = 0xfeed_9876;

    let diagonal: Vec<f64> = (0..N)
        .map(|i| {
            let magnitude = 2.0 + next(&mut seed).abs();
            if i % 4 == 1 { -magnitude } else { magnitude }
        })
        .collect();
    let off: Vec<f64> = (0..N - 1).map(|_| 0.5 * next(&mut seed)).collect();

    // Sturm sequence at a shift of zero: the leading principal minors' pivot recurrence, whose
    // negative count is the number of eigenvalues below zero.
    let mut sturm_negative = usize::from(diagonal[0] < 0.0);
    let mut pivot = diagonal[0];
    for i in 1..N {
        pivot = diagonal[i] - off[i - 1] * off[i - 1] / pivot;
        sturm_negative += usize::from(pivot < 0.0);
    }

    let by_dominance = diagonal.iter().filter(|v| **v < 0.0).count();
    assert_eq!(
        sturm_negative, by_dominance,
        "the two derivations of the expected answer disagree"
    );

    let mut a = vec![vec![0.0; N]; N];
    for i in 0..N {
        a[i][i] = diagonal[i];
        if i + 1 < N {
            a[i][i + 1] = off[i];
            a[i + 1][i] = off[i];
        }
    }

    let (column_starts, row_indices, values) = lower_triangle_csc(&a);
    let inertia = inertia_of_csc(N, &column_starts, &row_indices, &values);

    assert_eq!(
        inertia,
        Inertia {
            positive: N - by_dominance,
            zero: 0,
            negative: by_dominance
        }
    );
}

/// The counts are scalars, not blocks, through the safe API as well: a block-diagonal matrix of
/// 2x2 blocks reports twice as many eigenvalues as it has blocks.
///
/// The blocks are deliberately of two kinds so that the two counts differ: `[[1, 2], [2, 1]]` has
/// eigenvalues 3 and -1, contributing one of each sign, while `[[2, 0], [0, 3]]` contributes two
/// positive. Equal counts would be satisfied just as well by an implementation that swapped them.
#[test]
fn the_counts_are_scalars_when_the_block_size_is_above_one() {
    if !SUPPORTED {
        return;
    }

    const INDEFINITE_BLOCKS: usize = 16;
    const DEFINITE_BLOCKS: usize = 16;
    const BLOCKS: usize = INDEFINITE_BLOCKS + DEFINITE_BLOCKS;

    let column_starts: Vec<i64> = (0..=BLOCKS as i64).collect();
    let row_indices: Vec<i32> = (0..BLOCKS as i32).collect();
    let values: Vec<f64> = (0..BLOCKS)
        .flat_map(|block| {
            if block < INDEFINITE_BLOCKS {
                [1.0, 2.0, 2.0, 1.0]
            } else {
                [2.0, 0.0, 0.0, 3.0]
            }
        })
        .collect();

    let structure = SparseStructure::from_csc(
        BLOCKS,
        BLOCKS,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap()
    .with_block_size(2)
    .unwrap();

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure).unwrap();
    let inertia = symbolic.factorize(&values).unwrap().inertia().unwrap();

    assert_eq!(
        inertia,
        Inertia {
            positive: INDEFINITE_BLOCKS + 2 * DEFINITE_BLOCKS,
            zero: 0,
            negative: INDEFINITE_BLOCKS
        }
    );
    assert_eq!(
        inertia.positive + inertia.zero + inertia.negative,
        2 * BLOCKS,
        "counts are not in scalars"
    );
}

/// The split between `zero` and the other two counts is a report about the factorization, not
/// only about the matrix.
///
/// `A = [[1, 1], [1, 1 + 1e-8]]` has determinant 1e-8 and both eigenvalues positive, so its
/// inertia is `(2, 0, 0)`. Eliminating the first column leaves a second pivot of 1e-8. Under
/// Accelerate's default zero tolerance — around 1e-4 times the double epsilon — that pivot is far
/// from zero and is counted positive; widening the tolerance past it moves the same pivot into
/// `zero` without the matrix having changed.
///
/// The effect is observed only for a pivot produced by elimination. A matrix carrying the small
/// value on its diagonal from the start, `diag(1, 1e-8)`, keeps its `(2, 0, 0)` count at every
/// zero tolerance up to and including one that exceeds its largest entry, at which point both
/// pivots turn zero at once. Whatever the tolerance is compared against, it is not simply the
/// stored diagonal entry.
#[test]
fn the_zero_count_follows_the_zero_tolerance() {
    if !SUPPORTED {
        return;
    }

    let column_starts = [0i64, 2, 3];
    let row_indices = [0i32, 1, 1];
    let values = [1.0f64, 1.0, 1.0 + 1e-8];
    let structure = SparseStructure::from_csc(
        2,
        2,
        &column_starts,
        &row_indices,
        Attributes::symmetric(Triangle::Lower),
    )
    .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure).unwrap();

    let default = symbolic.factorize(&values).unwrap().inertia().unwrap();
    assert_eq!(
        default,
        Inertia {
            positive: 2,
            zero: 0,
            negative: 0
        }
    );

    let widened = Factorization::with_options(
        &symbolic,
        &values,
        NumericOptions::new().zero_tolerance(1e-6),
    )
    .unwrap()
    .inertia()
    .unwrap();
    assert_eq!(
        widened,
        Inertia {
            positive: 1,
            zero: 1,
            negative: 0
        }
    );
}

/// The counterpart to every `SUPPORTED` guard above: in a build without the underlying function
/// the query still exists and reports that it is unavailable, rather than being absent from the
/// API or answering with something invented.
#[test]
fn an_unsupported_build_reports_it() {
    if SUPPORTED {
        return;
    }

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure()).unwrap();
    let error = symbolic
        .factorize(&DEFINITE)
        .unwrap()
        .inertia()
        .unwrap_err();
    assert_eq!(error.status(), Some(Status::UnsupportedOs));
}

/// Concurrently queries inertia and solves through a shared `Factorization`. Apple does not
/// document the query's thread safety, so every thread checks its result.
#[test]
fn inertia_is_safe_to_query_from_many_threads() {
    if !SUPPORTED {
        return;
    }

    let symbolic = SymbolicFactorization::new(FactorizationKind::LdltTpp, &structure()).unwrap();
    let factorization = symbolic.factorize(&INDEFINITE).unwrap();
    let expected = Inertia {
        positive: 2,
        zero: 0,
        negative: 1,
    };

    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..250 {
                    assert_eq!(factorization.inertia().unwrap(), expected);
                    // Interleaved with a solve, so the two share whatever state they share.
                    let x = factorization.solve_vec(&[1.0, 1.0, 1.0]).unwrap();
                    assert_eq!(x.len(), 3);
                }
            });
        }
    });
}
