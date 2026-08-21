//! Every way a pattern or a call can be rejected, and the thread-safety bounds.
//!
//! These matter more than they look. Past the validation layer the index arrays are raw pointers
//! into a C framework, so a check that silently weakened would turn a diagnosable error into an
//! out-of-bounds read inside Accelerate.

#![cfg(target_os = "macos")]

use accelerate_sparse::{
    Attributes, DenseMut, DenseRef, Factorization, FactorizationKind, SparseStructure,
    SymbolicFactorization, Triangle,
    error::{DenseDimension, DenseField, IndexSource, InputError, StructureError},
    options::{OrderMethod, SymbolicOptions},
};

fn symmetric() -> Attributes {
    Attributes::symmetric(Triangle::Lower)
}

#[test]
fn rejects_a_zero_dimension() {
    // Accelerate raises SIGTRAP on an empty matrix rather than returning a status, so this one
    // must never reach it.
    let native_column_starts = [0i64];
    let native_row_indices: [i32; 0] = [];
    let converted_column_starts = [0usize];
    let converted_row_indices: [usize; 0] = [];
    let expected = Err(StructureError::InvalidDimension {
        rows: 0,
        columns: 0,
    });

    assert_eq!(
        SparseStructure::from_csc(
            0,
            0,
            &native_column_starts,
            &native_row_indices,
            symmetric()
        ),
        expected
    );
    assert_eq!(
        SparseStructure::convert_from_csc(
            0,
            0,
            &converted_column_starts,
            &converted_row_indices,
            symmetric()
        ),
        expected
    );
}

#[test]
fn rejects_a_non_square_symmetric_matrix() {
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    assert_eq!(
        SparseStructure::from_csc(3, 2, &column_starts, &row_indices, symmetric()),
        Err(StructureError::NotSquare {
            rows: 3,
            columns: 2
        })
    );
}

#[test]
fn rejects_wrong_column_starts_length() {
    let column_starts = [0i64, 2, 4]; // three entries where a 3-column matrix needs four
    let row_indices = [0i32, 1, 1, 2, 2];
    assert_eq!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::ColumnStartsLength {
            expected: 4,
            actual: 3
        })
    );
}

/// Monotonicity alone leaves the first entry unconstrained upward, which stays inside the arrays
/// but silently factors a different matrix: the leading entries would belong to no column.
#[test]
fn rejects_column_starts_that_skip_a_prefix() {
    let column_starts = [1i64, 3, 5, 6];
    let row_indices = [0i32, 0, 1, 1, 2, 2];
    assert_eq!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::ColumnStartsNotZeroBased { first: 1 })
    );
}

/// A descending pair would make a column's index range run backwards over raw pointers.
#[test]
fn rejects_descending_column_starts() {
    let column_starts = [0i64, 3, 1, 5];
    let row_indices = [0i32, 1, 2, 1, 2];
    assert_eq!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::ColumnStartsNotMonotone {
            index: 2,
            previous: 3,
            current: 1
        })
    );
}

#[test]
fn rejects_a_non_zero_count_mismatch() {
    let column_starts = [0i64, 2, 4, 4]; // ends at 4, but five row indices were given
    let row_indices = [0i32, 1, 1, 2, 2];
    assert_eq!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::NonZeroCountMismatch {
            column_starts_end: 4,
            row_indices: 5
        })
    );
}

#[test]
fn rejects_an_out_of_range_row_index() {
    let column_starts = [0i64, 1, 2, 3];
    let row_indices = [0i32, 1, 3]; // 3 is outside a 3-row matrix
    assert_eq!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::RowIndexOutOfRange {
            position: 2,
            row_index: 3,
            rows: 3
        })
    );
}

#[test]
fn rejects_a_negative_row_index() {
    let column_starts = [0i64, 1, 2, 3];
    let row_indices = [0i32, -1, 2];
    assert!(matches!(
        SparseStructure::from_csc(3, 3, &column_starts, &row_indices, symmetric()),
        Err(StructureError::RowIndexOutOfRange { row_index: -1, .. })
    ));
}

#[test]
fn rejects_an_index_that_does_not_fit() {
    // Accelerate stores row indices as i32, so a usize beyond that range cannot be narrowed.
    let column_starts = [0usize, 1];
    let row_indices = [usize::MAX];
    assert_eq!(
        SparseStructure::convert_from_csc(1, 1, &column_starts, &row_indices, symmetric()),
        Err(StructureError::IndexOverflow {
            what: IndexSource::RowIndices
        })
    );
}

/// The other three `IndexOverflow` sources each carry a distinct `what`, and each is rejected
/// before the arrays are inspected, so a swapped or mislabelled arm would otherwise go unnoticed.
#[test]
fn rejects_a_row_count_that_does_not_fit() {
    let column_starts = [0usize];
    let row_indices: [usize; 0] = [];
    assert_eq!(
        SparseStructure::convert_from_csc(
            i32::MAX as usize + 1,
            1,
            &column_starts,
            &row_indices,
            symmetric()
        ),
        Err(StructureError::IndexOverflow {
            what: IndexSource::Rows
        })
    );
}

#[test]
fn rejects_a_column_count_that_does_not_fit() {
    let column_starts = [0usize];
    let row_indices: [usize; 0] = [];
    assert_eq!(
        SparseStructure::convert_from_csc(
            1,
            i32::MAX as usize + 1,
            &column_starts,
            &row_indices,
            symmetric()
        ),
        Err(StructureError::IndexOverflow {
            what: IndexSource::Columns
        })
    );
}

#[test]
fn native_and_conversion_paths_reject_dimension_overflow_identically() {
    let column_starts = [0i64];
    let row_indices: [i32; 0] = [];

    // Assert each concrete rejection as well as parity. Parity alone would pass if both paths
    // accepted an over-`i32` dimension.
    let rows = i32::MAX as usize + 1;
    let row_overflow = Err(StructureError::IndexOverflow {
        what: IndexSource::Rows,
    });
    assert_eq!(
        SparseStructure::from_csc(rows, 1, &column_starts, &row_indices, symmetric()),
        row_overflow
    );
    assert_eq!(
        SparseStructure::convert_from_csc(rows, 1, &column_starts, &row_indices, symmetric()),
        row_overflow
    );

    let columns = i32::MAX as usize + 1;
    let column_overflow = Err(StructureError::IndexOverflow {
        what: IndexSource::Columns,
    });
    assert_eq!(
        SparseStructure::from_csc(1, columns, &column_starts, &row_indices, symmetric()),
        column_overflow
    );
    assert_eq!(
        SparseStructure::convert_from_csc(1, columns, &column_starts, &row_indices, symmetric()),
        column_overflow
    );
}

#[test]
fn native_csc_borrows_its_arrays_and_exposes_them() {
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let structure =
        SparseStructure::from_csc(2, 2, &column_starts, &row_indices, symmetric()).unwrap();

    assert_eq!(structure.column_starts(), column_starts);
    assert_eq!(structure.row_indices(), row_indices);
    assert_eq!(structure.column_starts().as_ptr(), column_starts.as_ptr());
    assert_eq!(structure.row_indices().as_ptr(), row_indices.as_ptr());
}

#[test]
fn into_owned_detaches_borrows_and_moves_already_owned_arrays() {
    fn assert_static<'a>(structure: &'a SparseStructure<'static>) -> &'a SparseStructure<'static> {
        structure
    }

    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let borrowed =
        SparseStructure::from_csc(2, 2, &column_starts, &row_indices, symmetric()).unwrap();
    let owned = borrowed.clone().into_owned();
    assert_eq!(*assert_static(&owned), borrowed);
    assert_ne!(owned.column_starts().as_ptr(), column_starts.as_ptr());
    assert_ne!(owned.row_indices().as_ptr(), row_indices.as_ptr());

    // Arrays that convert_from_csc already owns move rather than being copied again.
    let converted =
        SparseStructure::convert_from_csc(2, 2, &[0usize, 1, 2], &[0usize, 1], symmetric())
            .unwrap();
    let starts_ptr = converted.column_starts().as_ptr();
    let indices_ptr = converted.row_indices().as_ptr();
    let moved = converted.into_owned();
    assert_eq!(moved.column_starts().as_ptr(), starts_ptr);
    assert_eq!(moved.row_indices().as_ptr(), indices_ptr);
}

#[test]
fn rejects_a_column_start_that_does_not_fit() {
    // Column starts are narrowed to i64, which usize::MAX exceeds on this 64-bit target.
    let column_starts = [0usize, usize::MAX];
    let row_indices = [0usize];
    assert_eq!(
        SparseStructure::convert_from_csc(1, 1, &column_starts, &row_indices, symmetric()),
        Err(StructureError::IndexOverflow {
            what: IndexSource::ColumnStarts
        })
    );
}

#[test]
fn rejects_a_zero_block_size() {
    let column_starts = [0i64, 1];
    let row_indices = [0i32];
    let structure =
        SparseStructure::from_csc(1, 1, &column_starts, &row_indices, symmetric()).unwrap();
    assert_eq!(
        structure.with_block_size(0),
        Err(StructureError::ZeroBlockSize)
    );
}

/// The conversion path must accept the index types a caller is likely to already hold.
#[test]
fn converts_from_usize_indices() {
    let column_starts = [0usize, 2, 4, 5];
    let row_indices = [0usize, 1, 1, 2, 2];
    let structure =
        SparseStructure::convert_from_csc(3, 3, &column_starts, &row_indices, symmetric()).unwrap();
    assert_eq!(structure.rows(), 3);
    assert_eq!(structure.stored_entries(), 5);
    assert_eq!(structure.column_starts(), &[0i64, 2, 4, 5]);
    assert_eq!(structure.row_indices(), &[0i32, 1, 1, 2, 2]);

    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let factorization = symbolic.factorize(&[4.0f64, 1.0, 3.0, 1.0, 2.0]).unwrap();
    let x = factorization.solve_vec(&[1.0, 2.0, 3.0]).unwrap();
    assert!((x[0] - 2.0 / 9.0).abs() < 1e-9);
}

// --- local input errors and documented panics --------------------------------------------------

fn spd() -> SparseStructure<'static> {
    const CS: [i64; 4] = [0, 2, 4, 5];
    const RI: [i32; 5] = [0, 1, 1, 2, 2];
    SparseStructure::from_csc(3, 3, &CS, &RI, Attributes::symmetric(Triangle::Lower)).unwrap()
}

#[test]
fn factorize_rejects_a_wrong_values_length() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    assert_eq!(
        symbolic.factorize(&[1.0, 2.0]).unwrap_err().input(),
        Some(&InputError::ValuesLength {
            expected: 5,
            actual: 2,
        })
    );
}

#[test]
fn refactor_rejects_a_wrong_values_length_without_unfactoring() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let mut factorization = symbolic.factorize(&[4.0, 1.0, 3.0, 1.0, 2.0]).unwrap();
    assert_eq!(
        factorization.refactor(&[1.0, 2.0]).unwrap_err().input(),
        Some(&InputError::ValuesLength {
            expected: 5,
            actual: 2,
        })
    );
    assert!(factorization.is_factored());
    let _solution = factorization.solve_vec(&[1.0, 2.0, 3.0]).unwrap();
}

/// A block pattern counts scalar tile entries rather than stored block indices in its values
/// requirement, and the returned error reports that distinction numerically.
#[test]
fn factorize_reports_the_scalar_value_count_for_a_block_pattern() {
    const CS: [i64; 3] = [0, 1, 2];
    const RI: [i32; 2] = [0, 1];
    let structure =
        SparseStructure::from_csc(2, 2, &CS, &RI, Attributes::symmetric(Triangle::Lower))
            .unwrap()
            .with_block_size(2)
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    assert_eq!(
        symbolic.factorize(&[1.0, 1.0]).unwrap_err().input(),
        Some(&InputError::ValuesLength {
            expected: 8,
            actual: 2,
        })
    );
}

#[test]
fn solve_rejects_a_wrong_operand_length() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&[4.0, 1.0, 3.0, 1.0, 2.0]).unwrap();
    assert_eq!(
        factorization.solve_vec(&[1.0, 2.0]).unwrap_err().input(),
        Some(&InputError::OperandRows {
            operand: accelerate_sparse::error::OperandRole::RightHandSide,
            expected: 3,
            actual: 2,
        })
    );
}

/// A right-hand side counted in blocks rather than scalars. Accelerate would report this through
/// its error callback and leave the solution untouched while returning nothing the shim can see,
/// so the returned error is what stands between a caller and a silently unsolved system. The matrix has 4
/// scalar rows (2 blocks of 2); passing the block count of 2 must be rejected.
#[test]
fn solve_rejects_an_operand_counted_in_blocks() {
    // Two 2x2 blocks on the diagonal: a 4x4 matrix stored as 2x2 blocks.
    const CS: [i64; 3] = [0, 1, 2];
    const RI: [i32; 2] = [0, 1];
    let structure =
        SparseStructure::from_csc(2, 2, &CS, &RI, Attributes::symmetric(Triangle::Lower))
            .unwrap()
            .with_block_size(2)
            .unwrap();
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure).unwrap();
    let factorization = symbolic
        .factorize(&[4.0, 1.0, 1.0, 3.0, 5.0, 1.0, 1.0, 2.0])
        .unwrap();
    assert_eq!(
        factorization.solve_vec(&[1.0, 2.0]).unwrap_err().input(),
        Some(&InputError::OperandRows {
            operand: accelerate_sparse::error::OperandRole::RightHandSide,
            expected: 4,
            actual: 2,
        })
    );
}

#[test]
fn dense_view_rejects_a_stride_below_the_row_count() {
    let data = [0.0f64; 6];
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&data, 3, 2, 2).unwrap_err(),
        InputError::DenseStrideTooSmall {
            rows: 3,
            column_stride: 2,
        }
    );
}

#[test]
fn column_major_dense_constructors_make_compact_and_strided_views() {
    let shared = [1.0f64; 6];
    let compact_ref = DenseRef::from_column_major_slice(&shared, 2, 3).unwrap();
    assert_eq!((compact_ref.rows(), compact_ref.columns()), (2, 3));
    assert_eq!(compact_ref.column_stride(), 2);

    let strided_ref = DenseRef::from_column_major_slice_with_stride(&shared, 2, 2, 4).unwrap();
    assert_eq!((strided_ref.rows(), strided_ref.columns()), (2, 2));
    assert_eq!(strided_ref.column_stride(), 4);

    let mut compact_data = [0.0f64; 6];
    let compact_mut = DenseMut::from_column_major_slice(&mut compact_data, 2, 3).unwrap();
    assert_eq!((compact_mut.rows(), compact_mut.columns()), (2, 3));
    assert_eq!(compact_mut.column_stride(), 2);

    let mut strided_data = [0.0f64; 6];
    let strided_mut =
        DenseMut::from_column_major_slice_with_stride(&mut strided_data, 2, 2, 4).unwrap();
    assert_eq!((strided_mut.rows(), strided_mut.columns()), (2, 2));
    assert_eq!(strided_mut.column_stride(), 4);
}

#[test]
fn dense_ref_exposes_its_complete_original_backing_slice() {
    let data = [1.0f64, 2.0, 99.0, 3.0, 4.0, 98.0, 97.0];
    let view = DenseRef::from_column_major_slice_with_stride(&data, 2, 2, 3).unwrap();

    assert_eq!(view.backing_slice(), data);
    assert_eq!(view.backing_slice().as_ptr(), data.as_ptr());
}

#[test]
fn dense_mut_exposes_its_complete_original_backing_slice() {
    let mut data = [1.0f64, 2.0, 99.0, 3.0, 4.0, 98.0, 97.0];
    let original_ptr = data.as_ptr();

    {
        let mut view = DenseMut::from_column_major_slice_with_stride(&mut data, 2, 2, 3).unwrap();
        assert_eq!(view.backing_slice(), [1.0, 2.0, 99.0, 3.0, 4.0, 98.0, 97.0]);
        assert_eq!(view.backing_slice().as_ptr(), original_ptr);
        assert_eq!(
            view.backing_slice_mut().as_mut_ptr(),
            original_ptr.cast_mut()
        );

        view.backing_slice_mut()[2] = -99.0;
        view.backing_slice_mut()[6] = -97.0;
    }

    assert_eq!(data, [1.0, 2.0, -99.0, 3.0, 4.0, 98.0, -97.0]);
}

#[test]
fn column_major_dense_constructors_reuse_shape_validation() {
    let shared = [0.0f64; 5];
    assert_eq!(
        DenseRef::from_column_major_slice(&shared, 2, 3).unwrap_err(),
        InputError::DenseStorageTooShort {
            required: 6,
            actual: 5,
        }
    );
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&shared, 3, 1, 2).unwrap_err(),
        InputError::DenseStrideTooSmall {
            rows: 3,
            column_stride: 2,
        }
    );

    let mut compact_data = [0.0f64; 5];
    assert_eq!(
        DenseMut::from_column_major_slice(&mut compact_data, 2, 3).unwrap_err(),
        InputError::DenseStorageTooShort {
            required: 6,
            actual: 5,
        }
    );
    let mut strided_data = [0.0f64; 5];
    assert_eq!(
        DenseMut::from_column_major_slice_with_stride(&mut strided_data, 3, 1, 2).unwrap_err(),
        InputError::DenseStrideTooSmall {
            rows: 3,
            column_stride: 2,
        }
    );
}

#[test]
fn dense_view_rejects_a_slice_that_is_too_short() {
    let mut data = [0.0f64; 6];
    assert_eq!(
        DenseMut::from_column_major_slice_with_stride(&mut data, 3, 2, 5).unwrap_err(),
        InputError::DenseStorageTooShort {
            required: 8,
            actual: 6,
        }
    );
}

#[test]
fn dense_view_rejects_zero_rows() {
    let data = [0.0f64; 4];
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&data, 0, 1, 1).unwrap_err(),
        InputError::DenseZeroDimension {
            dimension: DenseDimension::Rows,
        }
    );
}

#[test]
fn dense_view_rejects_zero_columns() {
    let mut data = [0.0f64; 4];
    assert_eq!(
        DenseMut::from_column_major_slice_with_stride(&mut data, 1, 0, 1).unwrap_err(),
        InputError::DenseZeroDimension {
            dimension: DenseDimension::Columns,
        }
    );
}

#[test]
fn dense_view_rejects_an_empty_vector() {
    let data: [f64; 0] = [];
    assert_eq!(
        DenseRef::from_vector(&data).unwrap_err(),
        InputError::DenseZeroDimension {
            dimension: DenseDimension::Rows,
        }
    );
}

#[test]
fn dense_view_rejects_storage_arithmetic_overflow() {
    let data: [f64; 0] = [];
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&data, 1, usize::MAX, usize::MAX)
            .unwrap_err(),
        InputError::DenseStorageArithmeticOverflow {
            rows: 1,
            columns: usize::MAX,
            column_stride: usize::MAX,
        }
    );
}

#[test]
fn dense_view_rejects_values_outside_accelerates_integer_abi() {
    let data = [0.0f64];
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&data, 1, i32::MAX as usize + 1, 1)
            .unwrap_err(),
        InputError::DenseRepresentationOverflow {
            field: DenseField::Columns,
            value: i32::MAX as usize + 1,
        }
    );
}

#[test]
fn dense_view_rejects_every_field_outside_accelerates_integer_abi() {
    let data = [0.0f64];
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(
            &data,
            i32::MAX as usize + 1,
            1,
            i32::MAX as usize + 1,
        )
        .unwrap_err(),
        InputError::DenseRepresentationOverflow {
            field: DenseField::Rows,
            value: i32::MAX as usize + 1,
        }
    );
    assert_eq!(
        DenseRef::from_column_major_slice_with_stride(&data, 1, 1, i32::MAX as usize + 1)
            .unwrap_err(),
        InputError::DenseRepresentationOverflow {
            field: DenseField::ColumnStride,
            value: i32::MAX as usize + 1,
        }
    );
}

#[test]
fn dense_slice_conversions_are_fallible() {
    let data = [1.0f64, 2.0];
    let view = DenseRef::try_from(data.as_slice()).unwrap();
    assert_eq!((view.rows(), view.columns()), (2, 1));

    let empty: [f64; 0] = [];
    assert_eq!(
        DenseRef::try_from(empty.as_slice()).unwrap_err(),
        InputError::DenseZeroDimension {
            dimension: DenseDimension::Rows,
        }
    );

    let mut data = [1.0f64, 2.0];
    let view = DenseMut::try_from(data.as_mut_slice()).unwrap();
    assert_eq!((view.rows(), view.columns()), (2, 1));

    let mut empty: [f64; 0] = [];
    assert_eq!(
        DenseMut::try_from(empty.as_mut_slice()).unwrap_err(),
        InputError::DenseZeroDimension {
            dimension: DenseDimension::Rows,
        }
    );
}

/// `solve_into` rejects right-hand side and solution views carrying different numbers of columns.
#[test]
fn solve_into_rejects_operands_that_disagree_on_columns() {
    let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &spd()).unwrap();
    let factorization = symbolic.factorize(&[4.0, 1.0, 3.0, 1.0, 2.0]).unwrap();
    let b = [1.0f64, 2.0, 3.0]; // 3 by 1
    let mut x = [0.0f64; 6]; // 3 by 2
    let error = factorization
        .solve_into(
            DenseRef::from_column_major_slice_with_stride(&b, 3, 1, 3).unwrap(),
            DenseMut::from_column_major_slice_with_stride(&mut x, 3, 2, 3).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.input(),
        Some(&InputError::OperandColumns {
            first: accelerate_sparse::error::OperandRole::RightHandSide,
            first_columns: 1,
            second: accelerate_sparse::error::OperandRole::Solution,
            second_columns: 2,
        })
    );
    assert_eq!(x, [0.0; 6]);
}

#[test]
fn a_square_only_factorization_rejects_a_rectangular_matrix() {
    // Reached through `ordinary`, since a symmetric structure is already rejected as non-square
    // at construction.
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let structure =
        SparseStructure::from_csc(3, 2, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    assert_eq!(
        SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)
            .unwrap_err()
            .input(),
        Some(&InputError::FactorizationRequiresSquare {
            kind: FactorizationKind::Cholesky,
            rows: 3,
            columns: 2,
        })
    );
}

/// `CholeskyAtA` on a matrix with fewer rows than columns has singular normal equations and no
/// solution to return; without this guard Accelerate factors it with a success status and writes
/// only `min(rows, columns)` entries, leaving the tail of the solution as whatever the caller's
/// right-hand side held. The shape is structural, so it is rejected at analysis rather than left to
/// surface as a silently short solve.
#[test]
fn cholesky_ata_rejects_a_stored_wide_matrix() {
    // A = [[1, 1, 0], [0, 1, 1]] — 2 rows, 3 columns.
    let column_starts = [0i64, 1, 3, 4];
    let row_indices = [0i32, 0, 1, 1];
    let structure =
        SparseStructure::from_csc(2, 3, &column_starts, &row_indices, Attributes::ordinary())
            .unwrap();
    assert_eq!(
        SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure)
            .unwrap_err()
            .input(),
        Some(&InputError::FactorizationRequiresRowsAtLeastColumns {
            kind: FactorizationKind::CholeskyAtA,
            rows: 2,
            columns: 3,
        })
    );
}

/// The precondition is on the *effective* shape: a stored tall matrix flagged transposed is
/// effectively wide and must be rejected the same way, so the guard cannot be read off the stored
/// dimensions alone.
#[test]
fn cholesky_ata_rejects_when_a_transpose_makes_the_matrix_wide() {
    // Stored 3×2, transposed to an effective 2×3.
    let column_starts = [0i64, 1, 2];
    let row_indices = [0i32, 1];
    let structure = SparseStructure::from_csc(
        3,
        2,
        &column_starts,
        &row_indices,
        Attributes::ordinary().with_transpose(true),
    )
    .unwrap();
    assert_eq!(
        SymbolicFactorization::new(FactorizationKind::CholeskyAtA, &structure)
            .unwrap_err()
            .input(),
        Some(&InputError::FactorizationRequiresRowsAtLeastColumns {
            kind: FactorizationKind::CholeskyAtA,
            rows: 2,
            columns: 3,
        })
    );
}

/// COLAMD paired with a symmetric factorization must be rejected *before* Accelerate is reached,
/// because Accelerate does not reject it — it spins forever in the numeric phase at supernodal
/// size. The guard is in the symbolic phase, and this 3×3 case never reaches the numeric phase
/// anyway, so a regressed guard fails cleanly here (`with_options` returning `Ok`) rather than
/// hanging. The guard is what keeps a *large* symmetric COLAMD call from hanging in the first
/// place.
#[test]
fn colamd_with_a_symmetric_kind_is_rejected_before_accelerate() {
    assert_eq!(
        SymbolicFactorization::with_options(
            FactorizationKind::Cholesky,
            &spd(),
            SymbolicOptions::new().order_method(OrderMethod::Colamd),
        )
        .unwrap_err()
        .input(),
        Some(&InputError::OrderingUnavailable {
            order: OrderMethod::Colamd,
            kind: FactorizationKind::Cholesky,
        })
    );
}

// --- thread-safety bounds ---------------------------------------------------------------------

/// Pins the one bound that is *held*: `Factorization: Sync`. Every withheld direction — both of
/// `SymbolicFactorization`'s, and `Factorization: !Send` — is pinned by a `compile_fail` doctest on
/// its type, which a change here cannot silently widen.
#[test]
fn the_thread_safety_bounds_are_what_they_claim() {
    fn assert_sync<T: Sync>() {}

    assert_sync::<Factorization<f64>>();
}
