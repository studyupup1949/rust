//! Proves the shim links and that a factor/solve/refactor cycle reaches Accelerate and comes
//! back with the right answer. The safe layer is what makes any of this pleasant to call; this
//! only has to show that the flattened ABI is wired correctly.

#![cfg(target_os = "macos")]

use accelerate_sparse_sys as sys;
use core::ffi::c_int;
use std::ptr;

// Symmetric positive-definite A = [[4,1,0],[1,3,1],[0,1,2]], lower triangle in column-major
// order. With b = [1,2,3] the solution is x = [2/9, 1/9, 13/9].
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [c_int; 5] = [0, 1, 1, 2, 2];
const VALUES: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];
const RHS: [f64; 3] = [1.0, 2.0, 3.0];

fn expected() -> [f64; 3] {
    [2.0 / 9.0, 1.0 / 9.0, 13.0 / 9.0]
}

fn assert_close(got: &[f64], want: &[f64]) {
    assert_eq!(got.len(), want.len());
    for (i, (g, w)) in got.iter().zip(want).enumerate() {
        assert!((g - w).abs() < 1e-9, "component {i}: got {g}, want {w}");
    }
}

fn symmetric_lower() -> sys::accsp_attributes {
    sys::accsp_attributes {
        kind: sys::ACCSP_MATRIX_SYMMETRIC,
        triangle: sys::ACCSP_TRIANGLE_LOWER,
        transpose: 0,
        block_size: 1,
    }
}

/// Builds a symbolic factorization of the test matrix for `kind`.
///
/// # Safety
///
/// The returned handle must be freed with `accsp_symbolic_free`.
unsafe fn symbolic(kind: c_int) -> (*mut sys::accsp_symbolic_t, c_int) {
    let attributes = symmetric_lower();
    let mut status = 0;
    // SAFETY: the pattern arrays are the module constants above, which describe a valid
    // symmetric CSC structure: four column starts for three columns, non-decreasing and ending
    // at the row-index count, and every row index below three.
    let handle = unsafe {
        sys::accsp_symbolic_new(
            kind,
            3,
            3,
            COLUMN_STARTS.as_ptr(),
            ROW_INDICES.as_ptr(),
            &attributes,
            ptr::null(),
            &mut status,
        )
    };
    (handle, status)
}

fn dense(data: *mut f64) -> sys::accsp_dense_d {
    sys::accsp_dense_d {
        row_count: 3,
        column_count: 1,
        column_stride: 3,
        data,
    }
}

#[test]
fn cholesky_factors_and_solves() {
    // SAFETY: each handle is used only while live and freed exactly once; `VALUES` has one entry
    // per stored non-zero, and both dense operands own three elements with a stride of three.
    unsafe {
        let (sym, status) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert!(!sym.is_null());
        assert!(sys::accsp_symbolic_factor_size_d(sym) > 0);

        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert!(!num.is_null());

        let mut x = [0.0f64; 3];
        let b = RHS;
        let status = sys::accsp_solve_d(num, &dense(b.as_ptr().cast_mut()), &dense(x.as_mut_ptr()));
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_close(&x, &expected());

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

#[test]
fn refactor_updates_the_solution() {
    // SAFETY: as above. `doubled` has the same length as `VALUES`, so it matches the pattern the
    // handle was built with.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        // Refactoring onto 2A must halve the solution, which a no-op refactor would not do.
        let doubled: Vec<f64> = VALUES.iter().map(|v| 2.0 * v).collect();
        assert_eq!(
            sys::accsp_numeric_refactor_d(num, doubled.as_ptr(), ptr::null()),
            sys::ACCSP_STATUS_OK
        );

        let mut x = [0.0f64; 3];
        let b = RHS;
        sys::accsp_solve_d(num, &dense(b.as_ptr().cast_mut()), &dense(x.as_mut_ptr()));
        let halved: Vec<f64> = expected().iter().map(|e| e / 2.0).collect();
        assert_close(&x, &halved);

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

#[test]
fn solve_in_place_matches_out_of_place() {
    // SAFETY: as above; `xb` is a distinct three-element buffer used for one call.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_LDLT_SBK);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let mut xb = RHS;
        let status = sys::accsp_solve_in_place_d(num, &dense(xb.as_mut_ptr()));
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_close(&xb, &expected());

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// Cholesky rejects an indefinite matrix with a numeric-phase status instead of trapping; the
/// pattern alone remains valid.
#[test]
fn cholesky_reports_a_status_on_an_indefinite_matrix() {
    // A = [[0,1],[1,0]], eigenvalues ±1, lower triangle in column-major order.
    const CS: [i64; 3] = [0, 2, 3];
    const RI: [c_int; 3] = [0, 1, 1];
    const V: [f64; 3] = [0.0, 1.0, 0.0];

    let attributes = symmetric_lower();
    // SAFETY: a valid two-column symmetric pattern; handles are freed on both paths.
    unsafe {
        let mut status = 0;
        let sym = sys::accsp_symbolic_new(
            sys::ACCSP_KIND_CHOLESKY,
            2,
            2,
            CS.as_ptr(),
            RI.as_ptr(),
            &attributes,
            ptr::null(),
            &mut status,
        );
        assert_eq!(status, sys::ACCSP_STATUS_OK, "the pattern itself is fine");

        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, V.as_ptr(), ptr::null(), &mut status);
        assert!(num.is_null());
        assert_eq!(status, sys::ACCSP_STATUS_FACTORIZATION_FAILED);

        sys::accsp_symbolic_free(sym);
    }
}

/// A numeric factorization must outlive the symbolic one it came from, for both solving and
/// refactoring. Accelerate retains the analysis internally and the shim keeps its own copy of the
/// pattern; getting this wrong is a use-after-free rather than a wrong answer.
#[test]
fn numeric_outlives_its_symbolic() {
    // SAFETY: `sym` is freed while `num` is still live, which is the property under test; `num`
    // is freed at the end.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        sys::accsp_symbolic_free(sym);

        let mut x = [0.0f64; 3];
        let b = RHS;
        sys::accsp_solve_d(num, &dense(b.as_ptr().cast_mut()), &dense(x.as_mut_ptr()));
        assert_close(&x, &expected());

        let doubled: Vec<f64> = VALUES.iter().map(|v| 2.0 * v).collect();
        assert_eq!(
            sys::accsp_numeric_refactor_d(num, doubled.as_ptr(), ptr::null()),
            sys::ACCSP_STATUS_OK
        );

        sys::accsp_numeric_free_d(num);
    }
}

/// LU of a general (unsymmetric) square matrix, when the SDK provides it. Confirms the selector
/// reaches Accelerate's LU path through the OS guard and comes back with the right answer.
///
/// A = [[2,1,0],[0,3,1],[1,0,4]] column-major, ordinary. With b = [1,2,3] the solution is
/// x = [0.28, 0.44, 0.68].
#[cfg(accsp_have_lu)]
#[test]
fn lu_factors_and_solves_an_unsymmetric_matrix() {
    let column_starts = [0i64, 2, 4, 6];
    let row_indices = [0i32, 2, 0, 1, 1, 2];
    let values = [2.0f64, 1.0, 1.0, 3.0, 1.0, 4.0];
    let attributes = sys::accsp_attributes {
        kind: sys::ACCSP_MATRIX_ORDINARY,
        triangle: sys::ACCSP_TRIANGLE_LOWER,
        transpose: 0,
        block_size: 1,
    };

    // SAFETY: a valid 3x3 ordinary CSC structure; `values` has one entry per stored non-zero,
    // and each dense operand owns three elements with a stride of three. Handles are freed once.
    unsafe {
        let mut status = 0;
        let sym = sys::accsp_symbolic_new(
            sys::ACCSP_KIND_LU_TPP,
            3,
            3,
            column_starts.as_ptr(),
            row_indices.as_ptr(),
            &attributes,
            ptr::null(),
            &mut status,
        );
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert!(!sym.is_null());

        let num = sys::accsp_numeric_new_d(sym, values.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let mut x = [0.0f64; 3];
        let b = [1.0f64, 2.0, 3.0];
        let status = sys::accsp_solve_d(num, &dense(b.as_ptr().cast_mut()), &dense(x.as_mut_ptr()));
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_close(&x, &[0.28, 0.44, 0.68]);

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// Reading the inertia off a TPP factorization, for a positive-definite matrix and for an
/// indefinite one sharing its pattern.
///
/// The module matrix is positive definite, so every pivot is positive. Substituting the values
/// makes it `[[1,2,0],[2,1,2],[0,2,1]]`, a symmetric tridiagonal Toeplitz matrix whose
/// eigenvalues are `1 + 4cos(kπ/4)` for `k = 1, 2, 3` — two positive, one negative, none zero.
#[cfg(accsp_have_inertia)]
#[test]
fn inertia_counts_the_pivots_of_a_tpp_factorization() {
    // SAFETY: handles are used only while live and freed exactly once; each values slice has one
    // entry per stored non-zero of the module pattern, and the three out-parameters are locals.
    unsafe {
        let (sym, status) = symbolic(sys::ACCSP_KIND_LDLT_TPP);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let (mut positive, mut zero, mut negative) = (0, 0, 0);
        let status = sys::accsp_get_inertia_d(num, &mut positive, &mut zero, &mut negative);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!((positive, zero, negative), (3, 0, 0));

        let indefinite = [1.0f64, 2.0, 1.0, 2.0, 1.0];
        assert_eq!(
            sys::accsp_numeric_refactor_d(num, indefinite.as_ptr(), ptr::null()),
            sys::ACCSP_STATUS_OK
        );
        let status = sys::accsp_get_inertia_d(num, &mut positive, &mut zero, &mut negative);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!((positive, zero, negative), (2, 0, 1));

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// The single-precision entry point reaches the same answer through the `Float` overload.
///
/// Uses the indefinite values rather than the module's positive-definite ones: `(3, 0, 0)` would be
/// satisfied by a stub returning the dimension as the positive count, or by any confusion of the
/// zero and negative out-parameters, and this is the only single-precision assertion in this crate.
#[cfg(accsp_have_inertia)]
#[test]
fn inertia_is_available_in_single_precision() {
    // SAFETY: as above, with a single-precision values slice matching the same pattern.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_LDLT_TPP);
        let mut status = 0;
        let values: [f32; 5] = [1.0, 2.0, 1.0, 2.0, 1.0];
        let num = sys::accsp_numeric_new_f(sym, values.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let (mut positive, mut zero, mut negative) = (0, 0, 0);
        let status = sys::accsp_get_inertia_f(num, &mut positive, &mut zero, &mut negative);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!((positive, zero, negative), (2, 0, 1));

        sys::accsp_numeric_free_f(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// The counts are scalar pivots, not blocks. Apple documents neither way, and the distinction is
/// invisible at block size one, so it is pinned here: a block-diagonal matrix of two 2x2 blocks,
/// `diag([[1,2],[2,1]], I)`, is 2 blocks but 4 scalars, with eigenvalues 3, -1, 1, 1. Counts
/// summing to 4 mean scalars; summing to 2 would mean blocks.
#[cfg(accsp_have_inertia)]
#[test]
fn inertia_counts_scalars_rather_than_blocks() {
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let values = [1.0f64, 2.0, 2.0, 1.0, 1.0, 0.0, 0.0, 1.0];
    let attributes = sys::accsp_attributes {
        kind: sys::ACCSP_MATRIX_SYMMETRIC,
        triangle: sys::ACCSP_TRIANGLE_LOWER,
        transpose: 0,
        block_size: 2,
    };

    // SAFETY: a valid 2x2 block CSC structure — three column starts for two columns, ending at
    // the row-index count — and `values` carries one 2x2 block per stored non-zero. Handles are
    // used only while live and freed exactly once.
    unsafe {
        let mut status = 0;
        let sym = sys::accsp_symbolic_new(
            sys::ACCSP_KIND_LDLT_TPP,
            2,
            2,
            column_starts.as_ptr(),
            row_indices.as_ptr(),
            &attributes,
            ptr::null(),
            &mut status,
        );
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let num = sys::accsp_numeric_new_d(sym, values.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let (mut positive, mut zero, mut negative) = (0, 0, 0);
        let status = sys::accsp_get_inertia_d(num, &mut positive, &mut zero, &mut negative);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!((positive, zero, negative), (3, 0, 1));
        assert_eq!(positive + zero + negative, 4, "counts are not in scalars");

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// Built against an SDK without `SparseGetInertia`, the entry point still exists and reports that
/// it is unavailable. The declaration is unconditional so the ABI does not change shape with the
/// SDK, which only means something if the fallback returns rather than doing something worse.
#[cfg(not(accsp_have_inertia))]
#[test]
fn inertia_reports_unsupported_without_sdk_support() {
    // SAFETY: as in the other cases; the handle is live for the call and freed once.
    unsafe {
        let (sym, status) = symbolic(sys::ACCSP_KIND_LDLT_TPP);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let (mut positive, mut zero, mut negative) = (-777, -777, -777);
        let status = sys::accsp_get_inertia_d(num, &mut positive, &mut zero, &mut negative);
        assert_eq!(status, sys::ACCSP_STATUS_UNSUPPORTED_OS);
        assert_eq!((positive, zero, negative), (-777, -777, -777));

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// Accelerate accepts this query only for TPP. It refuses any other kind through the error
/// callback and a non-zero return, which the shim translates into a parameter error; the counts
/// must be left alone rather than filled with whatever the failed call left behind.
///
/// This is also the first behavioural separation of the TPP selector from SBK: the two agree on
/// every solve small enough to check by hand, so nothing before this could tell them apart.
#[cfg(accsp_have_inertia)]
#[test]
fn inertia_is_refused_for_every_other_kind() {
    for kind in [
        sys::ACCSP_KIND_CHOLESKY,
        sys::ACCSP_KIND_LDLT_UNPIVOTED,
        sys::ACCSP_KIND_LDLT_SBK,
    ] {
        // SAFETY: as above; the sentinels are locals and are only read back.
        unsafe {
            let (sym, status) = symbolic(kind);
            assert_eq!(status, sys::ACCSP_STATUS_OK);

            let mut status = 0;
            let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
            assert_eq!(status, sys::ACCSP_STATUS_OK);

            let (mut positive, mut zero, mut negative) = (-777, -777, -777);
            let status = sys::accsp_get_inertia_d(num, &mut positive, &mut zero, &mut negative);
            assert_eq!(
                status,
                sys::ACCSP_STATUS_PARAMETER_ERROR,
                "kind {kind} should refuse an inertia query"
            );
            assert_eq!(
                (positive, zero, negative),
                (-777, -777, -777),
                "kind {kind} wrote counts despite failing"
            );

            sys::accsp_numeric_free_d(num);
            sys::accsp_symbolic_free(sym);
        }
    }
}

/// Subfactor extraction, and the composition that shows the pieces are the real thing.
///
/// Accelerate's Cholesky is `A = P L L' P'`, not `A = L L'` — the fill-reducing permutation is part
/// of it. Multiplying a vector by `P'`, `L'`, `L` and `P` in turn must therefore reproduce `A x`,
/// which a naive `L L' x` does not. `A x` is computed here by a matrix-vector product that shares
/// no code with Accelerate.
#[test]
fn subfactors_of_a_cholesky_compose_back_to_the_matrix() {
    let x = [1.0f64, 2.0, 3.0];
    // A = [[4,1,0],[1,3,1],[0,1,2]] times x, by hand.
    let expected = [
        4.0 * x[0] + 1.0 * x[1],
        1.0 * x[0] + 3.0 * x[1] + 1.0 * x[2],
        1.0 * x[1] + 2.0 * x[2],
    ];

    // SAFETY: every handle is used only while live and freed exactly once. The module pattern is a
    // valid symmetric 3x3 structure and `VALUES` has one entry per stored non-zero. All four
    // subfactors of this factorization are 3 by 3, so every dense operand below owns three
    // elements with a stride of three — the shim does not check this and Accelerate does not
    // either for a symmetric parent.
    unsafe {
        let (sym, status) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let l = sys::accsp_subfactor_new_d(num, sys::ACCSP_SUBFACTOR_L, &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!(sys::accsp_subfactor_contents_d(l), sys::ACCSP_SUBFACTOR_L);
        assert_eq!(sys::accsp_subfactor_is_transposed_d(l), 0);

        let p = sys::accsp_subfactor_new_d(num, sys::ACCSP_SUBFACTOR_P, &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        let lt = sys::accsp_subfactor_transpose_d(l, &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);
        assert_eq!(sys::accsp_subfactor_is_transposed_d(lt), 1);
        let pt = sys::accsp_subfactor_transpose_d(p, &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        // Workspace is reported, not required: the entry points allocate it themselves.
        let (mut static_bytes, mut per_rhs) = (0usize, 0usize);
        sys::accsp_subfactor_workspace_d(l, &mut static_bytes, &mut per_rhs);
        assert!(
            per_rhs > 0,
            "a subfactor solve needs scratch space per right-hand side"
        );

        // y = P L L' P' x, one factor at a time.
        let mut after_pt = [0.0f64; 3];
        let mut after_lt = [0.0f64; 3];
        let mut after_l = [0.0f64; 3];
        let mut got = [0.0f64; 3];
        let steps = [
            (pt, x.as_ptr().cast_mut(), after_pt.as_mut_ptr()),
            (lt, after_pt.as_mut_ptr(), after_lt.as_mut_ptr()),
            (l, after_lt.as_mut_ptr(), after_l.as_mut_ptr()),
            (p, after_l.as_mut_ptr(), got.as_mut_ptr()),
        ];
        for (subfactor, input, output) in steps {
            assert_eq!(
                sys::accsp_subfactor_multiply_d(subfactor, &dense(input), &dense(output)),
                sys::ACCSP_STATUS_OK
            );
        }
        assert_close(&got, &expected);

        // The same pieces without the permutation do not reconstruct the matrix, which is what
        // makes the composition above a real check rather than a coincidence.
        let mut naive = [0.0f64; 3];
        let mut half = [0.0f64; 3];
        sys::accsp_subfactor_multiply_d(
            lt,
            &dense(x.as_ptr().cast_mut()),
            &dense(half.as_mut_ptr()),
        );
        sys::accsp_subfactor_multiply_d(l, &dense(half.as_mut_ptr()), &dense(naive.as_mut_ptr()));
        assert!(
            naive
                .iter()
                .zip(&expected)
                .any(|(g, w)| (g - w).abs() > 1e-9),
            "L L' alone reproduced A, so the permutation is not being applied"
        );

        sys::accsp_subfactor_free_d(pt);
        sys::accsp_subfactor_free_d(lt);
        sys::accsp_subfactor_free_d(p);
        sys::accsp_subfactor_free_d(l);
        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// Multiplying by the half-solve subfactor must be refused by the shim rather than reaching
/// Accelerate, which traps the process when the operands are the right shape. The wrong shape is
/// caught by Accelerate's own check first, so a shape guard would not have covered this.
#[test]
fn multiplying_by_the_half_solve_is_refused() {
    // SAFETY: handles are used only while live and freed once; both operands own three elements
    // with a stride of three, which is the correct shape for this 3x3 factorization — the point
    // being that a correct shape is exactly the dangerous case.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let plps = sys::accsp_subfactor_new_d(num, sys::ACCSP_SUBFACTOR_PLPS, &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        let mut x = [1.0f64, 0.0, 0.0];
        let mut y = [0.0f64; 3];
        assert_eq!(
            sys::accsp_subfactor_multiply_d(plps, &dense(x.as_mut_ptr()), &dense(y.as_mut_ptr())),
            sys::ACCSP_STATUS_PARAMETER_ERROR
        );
        assert_eq!(
            sys::accsp_subfactor_multiply_in_place_d(plps, &dense(x.as_mut_ptr())),
            sys::ACCSP_STATUS_PARAMETER_ERROR
        );
        // Solving with it is the supported direction and must still work.
        assert_eq!(
            sys::accsp_subfactor_solve_d(plps, &dense(x.as_mut_ptr()), &dense(y.as_mut_ptr())),
            sys::ACCSP_STATUS_OK
        );

        sys::accsp_subfactor_free_d(plps);
        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}

/// A factorization that has no such piece reports it at extraction, not on first use. Accelerate
/// signals it by handing back a handle marked invalid whose parameter check fires later, so
/// catching it here is what keeps that out of reach.
#[test]
fn an_unavailable_subfactor_is_refused_at_extraction() {
    // SAFETY: as above; no subfactor handle is produced on the failing paths.
    unsafe {
        let (sym, _) = symbolic(sys::ACCSP_KIND_CHOLESKY);
        let mut status = 0;
        let num = sys::accsp_numeric_new_d(sym, VALUES.as_ptr(), ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK);

        // D belongs to LDL', Q and R to the rectangular kinds, and S exists only where scaling was
        // applied — which Cholesky does not do under Accelerate's defaults.
        for selector in [
            sys::ACCSP_SUBFACTOR_D,
            sys::ACCSP_SUBFACTOR_Q,
            sys::ACCSP_SUBFACTOR_R,
            sys::ACCSP_SUBFACTOR_S,
        ] {
            let mut status = 0;
            let sub = sys::accsp_subfactor_new_d(num, selector, &mut status);
            assert!(
                sub.is_null(),
                "selector {selector} should not yield a handle"
            );
            assert_eq!(
                status,
                sys::ACCSP_STATUS_PARAMETER_ERROR,
                "selector {selector}"
            );
        }

        sys::accsp_numeric_free_d(num);
        sys::accsp_symbolic_free(sym);
    }
}
