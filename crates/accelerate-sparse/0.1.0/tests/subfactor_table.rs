//! The validity table, checked against Accelerate itself.
//!
//! Its own test binary, and deliberately so. The check has to reach past `Factorization::subfactor`
//! to the raw entry points, because that method consults the table it is supposed to be validating —
//! and the raw symbolic constructor does not take the lock the safe layer uses to serialize LU, which
//! Accelerate cannot run concurrently. Mixing raw LU construction with the safe layer's in one
//! process crashes it. A separate binary is a separate process, so nothing here can race.

#![cfg(target_os = "macos")]

use accelerate_sparse::{FactorizationKind, SubfactorKind};

// A = [[4,1,0],[1,3,1],[0,1,2]], symmetric positive definite, lower triangle.
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 5];
const ROW_INDICES: [i32; 5] = [0, 1, 1, 2, 2];
const VALUES: [f64; 5] = [4.0, 1.0, 3.0, 1.0, 2.0];

// The same pattern, indefinite.
const INDEFINITE: [f64; 5] = [1.0, 2.0, 1.0, 2.0, 1.0];

const SELECTORS: [(SubfactorKind, u8); 10] = [
    (SubfactorKind::P, accelerate_sparse::sys::ACCSP_SUBFACTOR_P),
    (SubfactorKind::S, accelerate_sparse::sys::ACCSP_SUBFACTOR_S),
    (SubfactorKind::L, accelerate_sparse::sys::ACCSP_SUBFACTOR_L),
    (SubfactorKind::D, accelerate_sparse::sys::ACCSP_SUBFACTOR_D),
    (
        SubfactorKind::Plps,
        accelerate_sparse::sys::ACCSP_SUBFACTOR_PLPS,
    ),
    (SubfactorKind::Q, accelerate_sparse::sys::ACCSP_SUBFACTOR_Q),
    (SubfactorKind::R, accelerate_sparse::sys::ACCSP_SUBFACTOR_R),
    (
        SubfactorKind::Rp,
        accelerate_sparse::sys::ACCSP_SUBFACTOR_RP,
    ),
    (
        SubfactorKind::Sr,
        accelerate_sparse::sys::ACCSP_SUBFACTOR_SR,
    ),
    (
        SubfactorKind::Sc,
        accelerate_sparse::sys::ACCSP_SUBFACTOR_SC,
    ),
];

/// A factorization to interrogate: the pattern, the values, and both spellings of its kind.
struct Parent<'a> {
    raw_kind: i32,
    kind: FactorizationKind,
    rows: i32,
    columns: i32,
    column_starts: &'a [i64],
    row_indices: &'a [i32],
    values: &'a [f64],
    symmetric: bool,
}

/// Asks Accelerate, through the raw layer, which pieces a factorization will actually hand over,
/// and checks each answer against the table.
fn assert_table_matches_accelerate(parent: Parent<'_>) {
    use accelerate_sparse::sys;

    let Parent {
        raw_kind,
        kind,
        rows,
        columns,
        column_starts,
        row_indices,
        values,
        symmetric,
    } = parent;

    let attributes = sys::accsp_attributes {
        kind: if symmetric {
            sys::ACCSP_MATRIX_SYMMETRIC
        } else {
            sys::ACCSP_MATRIX_ORDINARY
        },
        triangle: sys::ACCSP_TRIANGLE_LOWER,
        transpose: 0,
        block_size: 1,
    };

    // SAFETY: the arrays describe a valid CSC structure — `column_starts` has `columns + 1` entries,
    // is non-decreasing and ends at the row-index count, every row index is below `rows` — and
    // `values` has one entry per stored non-zero. Every handle is freed exactly once below.
    unsafe {
        let mut status = 0;
        let symbolic = sys::accsp_symbolic_new(
            raw_kind,
            rows,
            columns,
            column_starts.as_ptr(),
            row_indices.as_ptr(),
            &attributes,
            core::ptr::null(),
            &mut status,
        );
        assert_eq!(status, sys::ACCSP_STATUS_OK, "{kind:?}: symbolic phase");
        let numeric =
            sys::accsp_numeric_new_d(symbolic, values.as_ptr(), core::ptr::null(), &mut status);
        assert_eq!(status, sys::ACCSP_STATUS_OK, "{kind:?}: numeric phase");

        for (piece, selector) in SELECTORS {
            let mut status = 0;
            let handle = sys::accsp_subfactor_new_d(numeric, selector, &mut status);
            let accelerate_supplies = !handle.is_null();
            if accelerate_supplies {
                assert_eq!(status, sys::ACCSP_STATUS_OK);
                sys::accsp_subfactor_free_d(handle);
            } else {
                assert_eq!(status, sys::ACCSP_STATUS_PARAMETER_ERROR);
            }
            assert_eq!(
                piece.applies_to(kind),
                accelerate_supplies,
                "{kind:?} + {piece:?}: the table says {}, Accelerate says {}",
                piece.applies_to(kind),
                accelerate_supplies
            );
        }

        sys::accsp_numeric_free_d(numeric);
        sys::accsp_symbolic_free(symbolic);
    }
}

/// The table is right in both directions, for every kind: it neither claims a piece Accelerate
/// withholds nor withholds one Accelerate supplies.
#[test]
fn the_validity_table_agrees_with_accelerate_in_both_directions() {
    use accelerate_sparse::sys;

    for (raw, kind, values) in [
        (
            sys::ACCSP_KIND_CHOLESKY,
            FactorizationKind::Cholesky,
            VALUES,
        ),
        (
            sys::ACCSP_KIND_LDLT_UNPIVOTED,
            FactorizationKind::LdltUnpivoted,
            INDEFINITE,
        ),
        (
            sys::ACCSP_KIND_LDLT_SBK,
            FactorizationKind::LdltSbk,
            INDEFINITE,
        ),
        (
            sys::ACCSP_KIND_LDLT_TPP,
            FactorizationKind::LdltTpp,
            INDEFINITE,
        ),
    ] {
        assert_table_matches_accelerate(Parent {
            raw_kind: raw,
            kind,
            rows: 3,
            columns: 3,
            column_starts: &COLUMN_STARTS,
            row_indices: &ROW_INDICES,
            values: &values,
            symmetric: true,
        });
    }

    let rect_starts = [0i64, 4, 7, 9];
    let rect_rows = [0i32, 1, 2, 3, 1, 2, 3, 2, 3];
    let rect_values = [1.0f64, 1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 1.0, 1.0];
    for (raw, kind) in [
        (sys::ACCSP_KIND_QR, FactorizationKind::Qr),
        (sys::ACCSP_KIND_CHOLESKY_ATA, FactorizationKind::CholeskyAtA),
    ] {
        assert_table_matches_accelerate(Parent {
            raw_kind: raw,
            kind,
            rows: 4,
            columns: 3,
            column_starts: &rect_starts,
            row_indices: &rect_rows,
            values: &rect_values,
            symmetric: false,
        });
    }

    let lu_starts = [0i64, 2, 4, 6];
    let lu_rows = [0i32, 2, 0, 1, 1, 2];
    let lu_values = [2.0f64, 1.0, 1.0, 3.0, 1.0, 4.0];
    for (raw, kind) in [
        (sys::ACCSP_KIND_LU_UNPIVOTED, FactorizationKind::LuUnpivoted),
        (sys::ACCSP_KIND_LU_SPP, FactorizationKind::LuSpp),
        (sys::ACCSP_KIND_LU_TPP, FactorizationKind::LuTpp),
    ] {
        assert_table_matches_accelerate(Parent {
            raw_kind: raw,
            kind,
            rows: 3,
            columns: 3,
            column_starts: &lu_starts,
            row_indices: &lu_rows,
            values: &lu_values,
            symmetric: false,
        });
    }
}
