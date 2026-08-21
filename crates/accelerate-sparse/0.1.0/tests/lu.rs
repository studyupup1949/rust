//! LU of a general square matrix — the unsymmetric direct solve.
//!
//! LU needs the macOS 15.5 SDK to build and that OS to run. These tests exercise the path that is
//! reachable on the build host; the unsupported-OS branch (which returns [`Status::UnsupportedOs`]
//! rather than trapping) cannot be exercised on a host new enough to build LU at all, so it is
//! verified by inspection of the shim guard, and here only by the positive control that a
//! supported OS does *not* report it.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, error::InputError,
};
use std::thread;

// A = [[2, 1, 0], [0, 3, 1], [1, 0, 4]], unsymmetric, column-major, stored whole (ordinary).
// With b = [1, 2, 3] the solution is x = [0.28, 0.44, 0.68].
const COLUMN_STARTS: [i64; 4] = [0, 2, 4, 6];
const ROW_INDICES: [i32; 6] = [0, 2, 0, 1, 1, 2];
const VALUES: [f64; 6] = [2.0, 1.0, 1.0, 3.0, 1.0, 4.0];
const RHS: [f64; 3] = [1.0, 2.0, 3.0];

const LU_KINDS: [FactorizationKind; 3] = [
    FactorizationKind::LuUnpivoted,
    FactorizationKind::LuSpp,
    FactorizationKind::LuTpp,
];

fn square() -> SparseStructure<'static> {
    SparseStructure::from_csc(3, 3, &COLUMN_STARTS, &ROW_INDICES, Attributes::ordinary())
        .expect("a general 3 by 3 pattern is well formed")
}

/// `‖A x − b‖`, with a full (unsymmetric) matvec that shares no code with the solver.
fn residual(x: &[f64]) -> f64 {
    let mut ax = [0.0; 3];
    for column in 0..3 {
        for k in COLUMN_STARTS[column] as usize..COLUMN_STARTS[column + 1] as usize {
            ax[ROW_INDICES[k] as usize] += VALUES[k] * x[column];
        }
    }
    ax.iter()
        .zip(&RHS)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f64>()
        .sqrt()
}

#[test]
fn the_lu_variants_solve_an_unsymmetric_system() {
    for kind in LU_KINDS {
        let symbolic = SymbolicFactorization::new(kind, &square()).unwrap();
        let factorization = symbolic.factorize(&VALUES).unwrap();
        let x = factorization.solve_vec(&RHS).unwrap();
        let r = residual(&x);
        assert!(r < 1e-9, "{kind:?}: residual {r} is too large");
    }
}

/// The per-iteration path works for LU too: refactoring onto 2A halves the solution.
#[test]
fn lu_refactor_reuses_the_analysis() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::LuTpp, &square()).unwrap();
    let mut factorization = symbolic.factorize(&VALUES).unwrap();

    let doubled: Vec<f64> = VALUES.iter().map(|v| 2.0 * v).collect();
    factorization.refactor(&doubled).unwrap();
    let x = factorization.solve_vec(&RHS).unwrap();
    // 2A x = b means A x = b/2, so the residual of x against the original A and b/2 is what holds.
    let mut ax = [0.0; 3];
    for column in 0..3 {
        for k in COLUMN_STARTS[column] as usize..COLUMN_STARTS[column + 1] as usize {
            ax[ROW_INDICES[k] as usize] += VALUES[k] * x[column];
        }
    }
    for (a, b) in ax.iter().zip(&RHS) {
        assert!((a - b / 2.0).abs() < 1e-9);
    }
}

/// LU is a square factorization, so a rectangular matrix is rejected before Accelerate is reached.
#[test]
fn lu_rejects_a_rectangular_matrix() {
    let column_starts = [0i64, 2, 4];
    let row_indices = [0i32, 1, 0, 1];
    let structure =
        SparseStructure::from_csc(3, 2, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    assert_eq!(
        SymbolicFactorization::new(FactorizationKind::LuTpp, &structure)
            .unwrap_err()
            .input(),
        Some(&InputError::FactorizationRequiresSquare {
            kind: FactorizationKind::LuTpp,
            rows: 3,
            columns: 2,
        })
    );
}

/// Positive control for the OS guard: a host new enough to build LU can actually run it, so the
/// analysis succeeds rather than reporting [`Status::UnsupportedOs`] (old OS) or
/// [`Status::ParameterError`] (SDK without LU). The negative cases are not reachable here and are
/// covered by inspection of the shim.
#[test]
fn lu_is_available_on_a_host_that_builds_it() {
    SymbolicFactorization::new(FactorizationKind::LuTpp, &square())
        .and_then(|symbolic| symbolic.factorize(&VALUES))
        .expect("a host that built LU must be able to run it");
}

/// Building LU factorizations on many threads at once. Accelerate's LU symbolic phase is not
/// concurrency-safe and crashes the process without serialization; the crate serializes it, so
/// this must survive and every solve must be correct. Without the lock this segfaults the runner
/// — read a crash here as that lock having regressed, not as a flaky test.
#[test]
fn concurrent_lu_factorization_is_serialized() {
    let handles: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(|| {
                for _ in 0..50 {
                    let symbolic =
                        SymbolicFactorization::new(FactorizationKind::LuTpp, &square()).unwrap();
                    let factorization = symbolic.factorize(&VALUES).unwrap();
                    let x = factorization.solve_vec(&RHS).unwrap();
                    assert!(residual(&x) < 1e-9);
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("no worker thread panicked or crashed");
    }
}
