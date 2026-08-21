//! Sparse structures validate patterns and describe how matrix storage is read.

use accelerate_sparse_sys as sys;
use std::borrow::Cow;

use crate::error::{IndexSource, StructureError};
use crate::scalar::Scalar;

/// Selects the stored half of a symmetric or triangular matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Triangle {
    /// Entries with `row >= column`.
    Lower,
    /// Entries with `row <= column`.
    Upper,
}

/// Declares structural assumptions about stored matrix entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MatrixKind {
    /// No symmetry is assumed and every stored entry is read.
    Ordinary,
    /// Symmetric, with one triangle read and the other implied by reflection.
    Symmetric,
}

/// Controls how stored matrix entries are interpreted.
///
/// Entries outside a symmetric matrix's declared triangle are ignored. A full symmetric matrix can
/// therefore be passed unfiltered, and its row-major arrays used as column-major.
/// **This is observed behaviour, not documented by Apple for this path.** Apple's coordinate-format
/// converters instead transpose and sum such entries into their mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Attributes {
    kind: MatrixKind,
    triangle: Triangle,
    transpose: bool,
}

impl Attributes {
    /// A symmetric matrix with `triangle` as the half to read.
    pub fn symmetric(triangle: Triangle) -> Self {
        Self {
            kind: MatrixKind::Symmetric,
            triangle,
            transpose: false,
        }
    }

    /// A matrix with no assumed symmetry.
    pub fn ordinary() -> Self {
        Self {
            kind: MatrixKind::Ordinary,
            triangle: Triangle::Lower,
            transpose: false,
        }
    }

    /// Has Accelerate treat the stored matrix as transposed.
    pub fn with_transpose(mut self, transpose: bool) -> Self {
        self.transpose = transpose;
        self
    }

    /// What structure the entries are assumed to have.
    pub fn kind(&self) -> MatrixKind {
        self.kind
    }

    /// Which half is read. Meaningful only for a symmetric matrix.
    pub fn triangle(&self) -> Triangle {
        self.triangle
    }

    /// Whether the stored matrix is treated as transposed.
    pub fn is_transposed(&self) -> bool {
        self.transpose
    }

    fn to_raw(self, block_size: u8) -> sys::accsp_attributes {
        sys::accsp_attributes {
            kind: match self.kind {
                MatrixKind::Ordinary => sys::ACCSP_MATRIX_ORDINARY,
                MatrixKind::Symmetric => sys::ACCSP_MATRIX_SYMMETRIC,
            },
            triangle: match self.triangle {
                Triangle::Lower => sys::ACCSP_TRIANGLE_LOWER,
                Triangle::Upper => sys::ACCSP_TRIANGLE_UPPER,
            },
            transpose: i32::from(self.transpose),
            block_size,
        }
    }
}

/// Stores a validated sparsity pattern in compressed-column form.
///
/// Accelerate stores column starts as `i64` and row indices as `i32`, a split no Rust sparse
/// library matches. [`from_csc`](Self::from_csc) borrows arrays that already have those widths;
/// [`convert_from_csc`](Self::convert_from_csc) narrows from any other pair and allocates only
/// what it must.
///
/// Validation happens here rather than later because past this point the arrays are raw pointers
/// into a C framework, where a malformed pattern reads out of bounds instead of failing.
///
/// With a [`block_size`](Self::with_block_size) above one every dimension here counts blocks
/// rather than scalars. Dense right-hand sides and solutions are the exception and are always
/// counted in scalars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseStructure<'a> {
    column_starts: Cow<'a, [i64]>,
    row_indices: Cow<'a, [i32]>,
    rows: i32,
    columns: i32,
    attributes: Attributes,
    block_size: u8,
}

impl<'a> SparseStructure<'a> {
    /// Borrows a pattern whose index arrays already have the widths Accelerate stores.
    ///
    /// `column_starts` has one entry per column plus one, is non-decreasing, begins at zero, and
    /// ends at the number of row indices given.
    ///
    /// # Errors
    ///
    /// Returns a [`StructureError`] describing the first violation found.
    pub fn from_csc(
        rows: usize,
        columns: usize,
        column_starts: &'a [i64],
        row_indices: &'a [i32],
        attributes: Attributes,
    ) -> Result<Self, StructureError> {
        let (rows, columns) = Self::validate_dimensions(rows, columns)?;
        Self::build(
            rows,
            columns,
            Cow::Borrowed(column_starts),
            Cow::Borrowed(row_indices),
            attributes,
            1,
        )
    }

    /// Converts a pattern from any index representation, narrowing to the widths Accelerate
    /// stores.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::IndexOverflow`] if a value does not fit, and otherwise the same
    /// violations [`from_csc`](Self::from_csc) reports.
    pub fn convert_from_csc<P, I>(
        rows: usize,
        columns: usize,
        column_starts: &[P],
        row_indices: &[I],
        attributes: Attributes,
    ) -> Result<Self, StructureError>
    where
        P: Copy,
        I: Copy,
        i64: TryFrom<P>,
        i32: TryFrom<I>,
    {
        let (rows, columns) = Self::validate_dimensions(rows, columns)?;

        let column_starts = column_starts
            .iter()
            .map(|&start| {
                i64::try_from(start).map_err(|_| StructureError::IndexOverflow {
                    what: IndexSource::ColumnStarts,
                })
            })
            .collect::<Result<Vec<i64>, _>>()?;
        let row_indices = row_indices
            .iter()
            .map(|&index| {
                i32::try_from(index).map_err(|_| StructureError::IndexOverflow {
                    what: IndexSource::RowIndices,
                })
            })
            .collect::<Result<Vec<i32>, _>>()?;

        Self::build(
            rows,
            columns,
            Cow::Owned(column_starts),
            Cow::Owned(row_indices),
            attributes,
            1,
        )
    }

    /// Assembles an unsorted coordinate list into a compressed-column pattern and its values.
    ///
    /// Each position across the three slices is one stored entry: `values[p]` at row
    /// `row_indices[p]`, column `column_indices[p]`. Entries may arrive in any order. Duplicates
    /// are summed, in the order given. For a symmetric matrix, an entry outside the declared
    /// triangle is folded onto its mirror inside it and summed there — the convention Accelerate
    /// documents for its own coordinate converters. That differs from compressed-column input,
    /// where the solver ignores such entries; see [`Attributes`].
    ///
    /// The returned values match the returned pattern entry for entry and are what
    /// [`SymbolicFactorization::factorize`](crate::SymbolicFactorization::factorize) takes. The
    /// pattern's block size is one: the coordinate form is scalar. Explicit zeros are kept as
    /// stored entries.
    ///
    /// ```
    /// use accelerate_sparse::{Attributes, SparseStructure, Triangle};
    ///
    /// # fn main() -> Result<(), accelerate_sparse::error::StructureError> {
    /// // The lower triangle of [[4, 1, 0], [1, 3, 1], [0, 1, 2]], in arbitrary order, with the
    /// // (2, 1) entry supplied as its upper-triangle mirror and the diagonal 3 split in two.
    /// let row_indices = [2usize, 0, 1, 1, 1, 1];
    /// let column_indices = [2usize, 0, 0, 1, 2, 1];
    /// let values = [2.0, 4.0, 1.0, 1.5, 1.0, 1.5];
    ///
    /// let (structure, values) = SparseStructure::from_coordinates(
    ///     3,
    ///     3,
    ///     &row_indices,
    ///     &column_indices,
    ///     &values,
    ///     Attributes::symmetric(Triangle::Lower),
    /// )?;
    /// assert_eq!(structure.column_starts(), &[0, 2, 4, 5]);
    /// assert_eq!(structure.row_indices(), &[0, 1, 1, 2, 2]);
    /// assert_eq!(values, [4.0, 1.0, 3.0, 1.0, 2.0]);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::CoordinateLengthMismatch`] if the three slices disagree in
    /// length, [`StructureError::RowIndexOutOfRange`] or
    /// [`StructureError::ColumnIndexOutOfRange`] for an entry outside the matrix,
    /// [`StructureError::IndexOverflow`] for an index value no matrix dimension could hold, and
    /// otherwise the violations [`from_csc`](Self::from_csc) reports.
    pub fn from_coordinates<R, C, T>(
        rows: usize,
        columns: usize,
        row_indices: &[R],
        column_indices: &[C],
        values: &[T],
        attributes: Attributes,
    ) -> Result<(SparseStructure<'static>, Vec<T>), StructureError>
    where
        R: Copy,
        C: Copy,
        i64: TryFrom<R> + TryFrom<C>,
        T: Scalar + core::ops::AddAssign,
    {
        let (raw_rows, raw_columns) = Self::validate_dimensions(rows, columns)?;
        if row_indices.len() != column_indices.len() || row_indices.len() != values.len() {
            return Err(StructureError::CoordinateLengthMismatch {
                row_indices: row_indices.len(),
                column_indices: column_indices.len(),
                values: values.len(),
            });
        }
        let fold = attributes.kind() == MatrixKind::Symmetric;
        if fold && rows != columns {
            return Err(StructureError::NotSquare { rows, columns });
        }

        let mut entries: Vec<(i32, i32, T)> = Vec::with_capacity(values.len());
        for (position, ((&row, &column), &value)) in row_indices
            .iter()
            .zip(column_indices)
            .zip(values)
            .enumerate()
        {
            let row = i64::try_from(row).map_err(|_| StructureError::IndexOverflow {
                what: IndexSource::RowIndices,
            })?;
            let column = i64::try_from(column).map_err(|_| StructureError::IndexOverflow {
                what: IndexSource::ColumnIndices,
            })?;
            if row < 0 || row >= i64::from(raw_rows) {
                return Err(StructureError::RowIndexOutOfRange {
                    position,
                    row_index: row,
                    rows,
                });
            }
            if column < 0 || column >= i64::from(raw_columns) {
                return Err(StructureError::ColumnIndexOutOfRange {
                    position,
                    column_index: column,
                    columns,
                });
            }
            let (mut row, mut column) = (row as i32, column as i32);
            let outside = fold
                && match attributes.triangle() {
                    Triangle::Lower => row < column,
                    Triangle::Upper => row > column,
                };
            if outside {
                core::mem::swap(&mut row, &mut column);
            }
            entries.push((column, row, value));
        }

        // A stable sort keeps duplicates in input order, so their sum is the one the caller can
        // predict from the slice.
        entries.sort_by_key(|&(column, row, _)| (column, row));

        let mut column_starts = vec![0i64; columns + 1];
        let mut out_rows: Vec<i32> = Vec::with_capacity(entries.len());
        let mut out_values: Vec<T> = Vec::with_capacity(entries.len());
        let mut last = None;
        for (column, row, value) in entries {
            if last == Some((column, row)) {
                let merged = out_values
                    .last_mut()
                    .expect("a duplicate follows the entry it merges into");
                *merged += value;
            } else {
                column_starts[column as usize + 1] += 1;
                out_rows.push(row);
                out_values.push(value);
                last = Some((column, row));
            }
        }
        for column in 1..=columns {
            column_starts[column] += column_starts[column - 1];
        }

        let structure = SparseStructure::build(
            raw_rows,
            raw_columns,
            Cow::Owned(column_starts),
            Cow::Owned(out_rows),
            attributes,
            1,
        )?;
        Ok((structure, out_values))
    }

    /// Groups the pattern into `block_size` by `block_size` submatrices.
    ///
    /// Above one, this changes what the indices mean: a row or column index names a block rather
    /// than a scalar, so a pattern declared with `rows` and `columns` describes a matrix of
    /// `rows * block_size` scalar rows and `columns * block_size` scalar columns.
    ///
    /// The values slice then holds one `block_size` by `block_size` submatrix per stored index,
    /// each column-major, concatenated in index order — `block_size * block_size` values wherever
    /// the scalar form took one. Right-hand sides and solutions passed to a solve stay scalar
    /// vectors.
    ///
    /// For a symmetric matrix, the following is **observed behaviour not documented by Apple**:
    ///
    /// - A tile on the diagonal is read only on its declared side. Values stored in its other
    ///   half are ignored, as entries in the undeclared triangle of a scalar matrix are.
    /// - A tile off the diagonal is read as stored, and the matching tile in the undeclared
    ///   triangle is its *transpose*.
    /// - Tiles stored entirely inside the undeclared triangle are dropped, not folded into their
    ///   mirror.
    ///
    /// # Errors
    ///
    /// Returns [`StructureError::ZeroBlockSize`] if `block_size` is zero.
    pub fn with_block_size(mut self, block_size: u8) -> Result<Self, StructureError> {
        if block_size == 0 {
            return Err(StructureError::ZeroBlockSize);
        }
        self.block_size = block_size;
        Ok(self)
    }

    fn build(
        rows: i32,
        columns: i32,
        column_starts: Cow<'a, [i64]>,
        row_indices: Cow<'a, [i32]>,
        attributes: Attributes,
        block_size: u8,
    ) -> Result<Self, StructureError> {
        if attributes.kind() == MatrixKind::Symmetric && rows != columns {
            return Err(StructureError::NotSquare {
                rows: rows as usize,
                columns: columns as usize,
            });
        }

        let expected = columns as usize + 1;
        if column_starts.len() != expected {
            return Err(StructureError::ColumnStartsLength {
                expected,
                actual: column_starts.len(),
            });
        }
        if column_starts[0] != 0 {
            return Err(StructureError::ColumnStartsNotZeroBased {
                first: column_starts[0],
            });
        }

        // Monotonicity plus a final entry equal to the row-index count is what bounds every
        // interior entry: Accelerate reads column `j` as `row_indices[starts[j]..starts[j + 1]]`,
        // so a non-decreasing sequence ending inside the array keeps every such range inside it.
        for (index, pair) in column_starts.windows(2).enumerate() {
            if pair[1] < pair[0] {
                return Err(StructureError::ColumnStartsNotMonotone {
                    index: index + 1,
                    previous: pair[0],
                    current: pair[1],
                });
            }
        }

        let last = column_starts[columns as usize];
        if last != row_indices.len() as i64 {
            return Err(StructureError::NonZeroCountMismatch {
                column_starts_end: last,
                row_indices: row_indices.len(),
            });
        }

        for (position, &row_index) in row_indices.iter().enumerate() {
            if row_index < 0 || row_index >= rows {
                return Err(StructureError::RowIndexOutOfRange {
                    position,
                    row_index: i64::from(row_index),
                    rows: rows as usize,
                });
            }
        }

        Ok(Self {
            column_starts,
            row_indices,
            rows,
            columns,
            attributes,
            block_size,
        })
    }

    /// Rows in the matrix, in blocks if the block size is above one.
    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    /// Columns in the matrix, in blocks if the block size is above one.
    pub fn columns(&self) -> usize {
        self.columns as usize
    }

    /// Stored non-zeros, in blocks if the block size is above one.
    ///
    /// The values slice is longer than this by the square of the block size; see
    /// [`value_count`](Self::value_count).
    pub fn stored_entries(&self) -> usize {
        self.row_indices.len()
    }

    /// Entries the values slice must carry for this pattern.
    pub fn value_count(&self) -> usize {
        let block = usize::from(self.block_size);
        self.stored_entries() * block * block
    }

    /// How the stored entries are interpreted.
    pub fn attributes(&self) -> Attributes {
        self.attributes
    }

    /// Entries per block edge.
    pub fn block_size(&self) -> u8 {
        self.block_size
    }

    /// Converts both index arrays to owned storage, detaching the structure from its borrows.
    ///
    /// Arrays that were already owned move without copying; borrowed arrays are cloned. Use this
    /// to store a structure built with [`from_csc`](Self::from_csc) beyond the lifetime of the
    /// arrays it borrowed.
    pub fn into_owned(self) -> SparseStructure<'static> {
        SparseStructure {
            column_starts: Cow::Owned(self.column_starts.into_owned()),
            row_indices: Cow::Owned(self.row_indices.into_owned()),
            rows: self.rows,
            columns: self.columns,
            attributes: self.attributes,
            block_size: self.block_size,
        }
    }

    /// Returns the validated column starts in Accelerate's native `i64` width.
    ///
    /// The slice has [`columns() + 1`](Self::columns) entries, begins at zero, and is
    /// non-decreasing.
    pub fn column_starts(&self) -> &[i64] {
        &self.column_starts
    }

    /// Returns the validated row indices in Accelerate's native `i32` width.
    ///
    /// Every entry is a row in [`0..rows()`](Self::rows).
    pub fn row_indices(&self) -> &[i32] {
        &self.row_indices
    }

    pub(crate) fn raw_attributes(&self) -> sys::accsp_attributes {
        self.attributes.to_raw(self.block_size)
    }

    /// Checks public dimensions before arrays are copied or passed to Accelerate.
    fn validate_dimensions(rows: usize, columns: usize) -> Result<(i32, i32), StructureError> {
        if rows == 0 || columns == 0 {
            return Err(StructureError::InvalidDimension { rows, columns });
        }
        let rows = i32::try_from(rows).map_err(|_| StructureError::IndexOverflow {
            what: IndexSource::Rows,
        })?;
        let columns = i32::try_from(columns).map_err(|_| StructureError::IndexOverflow {
            what: IndexSource::Columns,
        })?;
        Ok((rows, columns))
    }

    /// Returns the validated row count for the Accelerate ABI.
    pub(crate) fn raw_rows(&self) -> i32 {
        self.rows
    }

    /// Returns the validated column count for the Accelerate ABI.
    pub(crate) fn raw_columns(&self) -> i32 {
        self.columns
    }
}
