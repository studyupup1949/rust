//! Assembly of coordinate (triplet) input into compressed-column form.
//!
//! The folding convention matters here: unlike the compressed-column path, where the solver
//! ignores entries outside a symmetric matrix's declared triangle, assembly folds them onto their
//! mirror. These tests pin both the arrays produced and that a factorization of them solves the
//! intended system.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, FactorizationKind, SparseStructure, SymbolicFactorization, Triangle,
    error::{IndexSource, StructureError},
};

fn symmetric() -> Attributes {
    Attributes::symmetric(Triangle::Lower)
}

#[test]
fn assembles_out_of_order_input_sums_duplicates_and_solves() {
    // [[4, 1, 0], [1, 3, 1], [0, 1, 2]]: lower-triangle entries out of order, with the diagonal
    // 3 split into two duplicates that must sum in input order.
    let rows = [1usize, 2, 0, 1, 2, 1];
    let columns = [0usize, 2, 0, 1, 1, 1];
    let values = [1.0f64, 2.0, 4.0, 1.0, 1.0, 2.0];

    let (structure, values) =
        SparseStructure::from_coordinates(3, 3, &rows, &columns, &values, symmetric()).unwrap();
    assert_eq!(structure.column_starts(), &[0, 2, 4, 5]);
    assert_eq!(structure.row_indices(), &[0, 1, 1, 2, 2]);
    assert_eq!(values, [4.0, 1.0, 3.0, 1.0, 2.0]);
    assert_eq!(structure.block_size(), 1);

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let x = symbolic
        .factorize(&values)
        .unwrap()
        .solve_vec(&[1.0, 2.0, 3.0])
        .unwrap();
    for (computed, expected) in x.iter().zip([2.0 / 9.0, 1.0 / 9.0, 13.0 / 9.0]) {
        assert!((computed - expected).abs() < 1e-12, "got {computed}");
    }
}

/// Two-term sums are commutative in IEEE arithmetic, so pinning the documented input-order
/// summation needs at least three duplicates whose sum depends on association.
#[test]
fn duplicates_are_summed_in_input_order() {
    let rows = [0usize, 0, 0];
    let columns = [0usize, 0, 0];
    let values = [1e16f64, 1.0, 1.0];

    let (structure, values) =
        SparseStructure::from_coordinates(1, 1, &rows, &columns, &values, Attributes::ordinary())
            .unwrap();
    assert_eq!(structure.stored_entries(), 1);
    // (1e16 + 1.0) + 1.0 rounds to 1e16 at each step; summing the small terms first would give
    // 1e16 + 2.0, which f64 represents exactly and which would fail here.
    assert_eq!(values, [1e16]);
}

#[test]
fn explicit_zeros_are_kept_as_stored_entries() {
    let rows = [0usize, 1];
    let columns = [0usize, 1];
    let values = [0.0f64, 3.0];

    let (structure, values) =
        SparseStructure::from_coordinates(2, 2, &rows, &columns, &values, Attributes::ordinary())
            .unwrap();
    assert_eq!(structure.stored_entries(), 2);
    assert_eq!(structure.row_indices(), &[0, 1]);
    assert_eq!(values, [0.0, 3.0]);
}

#[test]
fn folds_an_undeclared_triangle_entry_onto_its_mirror() {
    // (0, 1) lies in the undeclared upper triangle; it folds onto (1, 0) and sums with it.
    let rows = [0usize, 0, 1, 1];
    let columns = [0usize, 1, 0, 1];
    let values = [4.0, 0.5, 0.5, 3.0];

    let (structure, values) =
        SparseStructure::from_coordinates(2, 2, &rows, &columns, &values, symmetric()).unwrap();
    assert_eq!(structure.column_starts(), &[0, 2, 3]);
    assert_eq!(structure.row_indices(), &[0, 1, 1]);
    assert_eq!(values, [4.0, 1.0, 3.0]);
}

#[test]
fn folds_toward_a_declared_upper_triangle_as_well() {
    // With the upper triangle declared, (1, 0) is the entry that folds, onto (0, 1).
    let rows = [1usize, 0, 1];
    let columns = [0usize, 0, 1];
    let values = [1.0, 4.0, 3.0];

    let (structure, values) = SparseStructure::from_coordinates(
        2,
        2,
        &rows,
        &columns,
        &values,
        Attributes::symmetric(Triangle::Upper),
    )
    .unwrap();
    assert_eq!(structure.column_starts(), &[0, 1, 3]);
    assert_eq!(structure.row_indices(), &[0, 0, 1]);
    assert_eq!(values, [4.0, 1.0, 3.0]);
}

#[test]
fn an_ordinary_matrix_is_not_folded() {
    let rows = [0usize, 1, 0, 1];
    let columns = [0usize, 0, 1, 1];
    let values = [1.0, 2.0, 3.0, 4.0];

    let (structure, values) =
        SparseStructure::from_coordinates(2, 2, &rows, &columns, &values, Attributes::ordinary())
            .unwrap();
    assert_eq!(structure.column_starts(), &[0, 2, 4]);
    assert_eq!(structure.row_indices(), &[0, 1, 0, 1]);
    assert_eq!(values, [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn an_empty_coordinate_list_yields_an_empty_pattern() {
    let rows: [usize; 0] = [];
    let columns: [usize; 0] = [];
    let values: [f64; 0] = [];

    let (structure, values) =
        SparseStructure::from_coordinates(2, 2, &rows, &columns, &values, Attributes::ordinary())
            .unwrap();
    assert_eq!(structure.stored_entries(), 0);
    assert_eq!(structure.column_starts(), &[0, 0, 0]);
    assert!(values.is_empty());
}

#[test]
fn rejects_mismatched_slice_lengths() {
    assert_eq!(
        SparseStructure::from_coordinates(
            2,
            2,
            &[0usize, 1],
            &[0usize],
            &[1.0, 2.0],
            Attributes::ordinary()
        )
        .unwrap_err(),
        StructureError::CoordinateLengthMismatch {
            row_indices: 2,
            column_indices: 1,
            values: 2,
        }
    );
}

#[test]
fn rejects_out_of_range_and_negative_indices() {
    let ordinary = Attributes::ordinary();
    assert_eq!(
        SparseStructure::from_coordinates(2, 2, &[2usize], &[0usize], &[1.0], ordinary)
            .unwrap_err(),
        StructureError::RowIndexOutOfRange {
            position: 0,
            row_index: 2,
            rows: 2,
        }
    );
    assert_eq!(
        SparseStructure::from_coordinates(2, 2, &[0usize], &[2usize], &[1.0], ordinary)
            .unwrap_err(),
        StructureError::ColumnIndexOutOfRange {
            position: 0,
            column_index: 2,
            columns: 2,
        }
    );
    assert_eq!(
        SparseStructure::from_coordinates(2, 2, &[-1i64], &[0i64], &[1.0], ordinary).unwrap_err(),
        StructureError::RowIndexOutOfRange {
            position: 0,
            row_index: -1,
            rows: 2,
        }
    );
}

#[test]
fn rejects_indices_no_dimension_could_hold() {
    let ordinary = Attributes::ordinary();
    assert_eq!(
        SparseStructure::from_coordinates(2, 2, &[u128::MAX], &[0u128], &[1.0], ordinary)
            .unwrap_err(),
        StructureError::IndexOverflow {
            what: IndexSource::RowIndices,
        }
    );
    assert_eq!(
        SparseStructure::from_coordinates(2, 2, &[0u128], &[u128::MAX], &[1.0], ordinary)
            .unwrap_err(),
        StructureError::IndexOverflow {
            what: IndexSource::ColumnIndices,
        }
    );
}

#[test]
fn rejects_a_non_square_symmetric_coordinate_list() {
    assert_eq!(
        SparseStructure::from_coordinates(3, 2, &[0usize], &[0usize], &[1.0], symmetric())
            .unwrap_err(),
        StructureError::NotSquare {
            rows: 3,
            columns: 2,
        }
    );
}
