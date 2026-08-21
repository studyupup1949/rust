//! Dense views describe column-major right-hand sides and solutions.
//!
//! These support several right-hand sides and caller-owned output storage. For one right-hand side,
//! [`Factorization::solve_vec`](crate::Factorization::solve_vec) takes a slice and returns the
//! solution.
//!
//! An explicit column stride permits a view into a larger column-major matrix without a copy.

use crate::error::{DenseDimension, DenseField, InputError};
use crate::scalar::Scalar;

/// Views a shared column-major right-hand side.
#[derive(Debug, Clone, Copy)]
pub struct DenseRef<'a, T> {
    data: &'a [T],
    rows: usize,
    columns: usize,
    column_stride: usize,
}

/// Views an exclusive column-major solution.
#[derive(Debug)]
pub struct DenseMut<'a, T> {
    data: &'a mut [T],
    rows: usize,
    columns: usize,
    column_stride: usize,
}

/// Checks the shape a view claims against the storage it was given.
///
/// A view that overruns its slice would be an out-of-bounds write inside Accelerate, so every
/// incompatibility is reported before the raw view is built.
fn check(len: usize, rows: usize, columns: usize, column_stride: usize) -> Result<(), InputError> {
    if rows == 0 {
        return Err(InputError::DenseZeroDimension {
            dimension: DenseDimension::Rows,
        });
    }
    if columns == 0 {
        return Err(InputError::DenseZeroDimension {
            dimension: DenseDimension::Columns,
        });
    }
    if column_stride < rows {
        return Err(InputError::DenseStrideTooSmall {
            rows,
            column_stride,
        });
    }
    let required = column_stride
        .checked_mul(columns - 1)
        .and_then(|entries_before_last_column| entries_before_last_column.checked_add(rows))
        .ok_or(InputError::DenseStorageArithmeticOverflow {
            rows,
            columns,
            column_stride,
        })?;
    for (field, value) in [
        (DenseField::Rows, rows),
        (DenseField::Columns, columns),
        (DenseField::ColumnStride, column_stride),
    ] {
        if value > i32::MAX as usize {
            return Err(InputError::DenseRepresentationOverflow { field, value });
        }
    }
    if len < required {
        return Err(InputError::DenseStorageTooShort {
            required,
            actual: len,
        });
    }
    Ok(())
}

impl<'a, T: Scalar> DenseRef<'a, T> {
    /// Views compact column-major `data` as `rows` by `columns`.
    ///
    /// Consecutive columns begin `rows` elements apart. `data` must therefore contain at least
    /// `rows * columns` elements.
    ///
    /// # Errors
    ///
    /// As [`from_column_major_slice_with_stride`](Self::from_column_major_slice_with_stride), except
    /// the derived stride cannot be smaller than `rows`.
    pub fn from_column_major_slice(
        data: &'a [T],
        rows: usize,
        columns: usize,
    ) -> Result<Self, InputError> {
        Self::from_column_major_slice_with_stride(data, rows, columns, rows)
    }

    /// Views column-major `data` as `rows` by `columns` with `column_stride` elements between columns.
    ///
    /// Each column occupies its first `rows` elements. `column_stride` may exceed `rows` when the
    /// view omits trailing elements in each column of a larger allocation.
    ///
    /// # Errors
    ///
    /// Returns [`InputError`] if either dimension is zero, `column_stride` is below `rows`, the
    /// storage calculation overflows, the dimensions do not fit Accelerate's integer ABI, or
    /// `data` is too short for the shape described.
    pub fn from_column_major_slice_with_stride(
        data: &'a [T],
        rows: usize,
        columns: usize,
        column_stride: usize,
    ) -> Result<Self, InputError> {
        Self::new(data, rows, columns, column_stride)
    }

    /// Builds a view after checking the declared shape against the storage.
    fn new(
        data: &'a [T],
        rows: usize,
        columns: usize,
        column_stride: usize,
    ) -> Result<Self, InputError> {
        check(data.len(), rows, columns, column_stride)?;
        Ok(Self {
            data,
            rows,
            columns,
            column_stride,
        })
    }

    /// Views `data` as a single right-hand side.
    ///
    /// # Errors
    ///
    /// Returns [`InputError`] if `data` is empty or does not fit Accelerate's integer ABI.
    pub fn from_vector(data: &'a [T]) -> Result<Self, InputError> {
        Self::new(data, data.len(), 1, data.len())
    }

    /// Rows in the view.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Columns in the view, i.e. the number of right-hand sides.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the full backing slice supplied at construction.
    ///
    /// The slice includes per-column padding and trailing elements outside the view's
    /// `rows` by `columns` shape.
    pub fn backing_slice(&self) -> &[T] {
        self.data
    }

    /// Returns the number of elements between the starts of consecutive columns.
    pub fn column_stride(&self) -> usize {
        self.column_stride
    }

    pub(crate) fn raw(&self) -> T::Dense {
        T::dense(
            self.rows as i32,
            self.columns as i32,
            self.column_stride as i32,
            self.data.as_ptr().cast_mut(),
        )
    }
}

impl<'a, T: Scalar> TryFrom<&'a [T]> for DenseRef<'a, T> {
    type Error = InputError;

    fn try_from(data: &'a [T]) -> Result<Self, Self::Error> {
        Self::from_vector(data)
    }
}

impl<'a, T: Scalar> DenseMut<'a, T> {
    /// Views compact column-major `data` as `rows` by `columns`.
    ///
    /// Consecutive columns begin `rows` elements apart. `data` must therefore contain at least
    /// `rows * columns` elements.
    ///
    /// # Errors
    ///
    /// As [`DenseRef::from_column_major_slice`].
    pub fn from_column_major_slice(
        data: &'a mut [T],
        rows: usize,
        columns: usize,
    ) -> Result<Self, InputError> {
        Self::from_column_major_slice_with_stride(data, rows, columns, rows)
    }

    /// Views column-major `data` as `rows` by `columns` with `column_stride` elements between columns.
    ///
    /// Each column occupies its first `rows` elements. `column_stride` may exceed `rows` when the
    /// view omits trailing elements in each column of a larger allocation.
    ///
    /// # Errors
    ///
    /// As [`DenseRef::from_column_major_slice_with_stride`].
    pub fn from_column_major_slice_with_stride(
        data: &'a mut [T],
        rows: usize,
        columns: usize,
        column_stride: usize,
    ) -> Result<Self, InputError> {
        Self::new(data, rows, columns, column_stride)
    }

    /// Builds a view after checking the declared shape against the storage.
    fn new(
        data: &'a mut [T],
        rows: usize,
        columns: usize,
        column_stride: usize,
    ) -> Result<Self, InputError> {
        check(data.len(), rows, columns, column_stride)?;
        Ok(Self {
            data,
            rows,
            columns,
            column_stride,
        })
    }

    /// Views `data` as a single solution vector.
    ///
    /// # Errors
    ///
    /// As [`DenseRef::from_vector`].
    pub fn from_vector(data: &'a mut [T]) -> Result<Self, InputError> {
        let len = data.len();
        Self::new(data, len, 1, len)
    }

    /// Rows in the view.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Columns in the view.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the full backing slice supplied at construction.
    ///
    /// As [`DenseRef::backing_slice`].
    pub fn backing_slice(&self) -> &[T] {
        self.data
    }

    /// Returns the full mutable backing slice supplied at construction.
    ///
    /// Returns the same storage as [`backing_slice`](Self::backing_slice), mutably.
    pub fn backing_slice_mut(&mut self) -> &mut [T] {
        self.data
    }

    /// Returns the number of elements between the starts of consecutive columns.
    pub fn column_stride(&self) -> usize {
        self.column_stride
    }

    pub(crate) fn raw_mut(&mut self) -> T::Dense {
        T::dense(
            self.rows as i32,
            self.columns as i32,
            self.column_stride as i32,
            self.data.as_mut_ptr(),
        )
    }
}

impl<'a, T: Scalar> TryFrom<&'a mut [T]> for DenseMut<'a, T> {
    type Error = InputError;

    fn try_from(data: &'a mut [T]) -> Result<Self, Self::Error> {
        Self::from_vector(data)
    }
}
