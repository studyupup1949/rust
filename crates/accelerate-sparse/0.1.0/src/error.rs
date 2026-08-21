//! Errors distinguish structure validation, input validation, and solver outcomes.
//!
//! [`StructureError`] reports malformed sparse storage before it reaches Accelerate. [`Error`]
//! separates locally rejected caller input, carried by [`InputError`], from a [`Status`] reported
//! by Accelerate or by factorization state. The remaining enums name the specific field, operand,
//! or index a variant refers to.

use accelerate_sparse_sys as sys;
use core::fmt;

use crate::{FactorizationKind, SubfactorKind, options::OrderMethod};

/// Represents a solver or factorization outcome.
///
/// Mirrors the framework's own status codes. `NotFactored` is this crate's addition, covering a
/// solve attempted against a factorization whose last attempt failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Status {
    /// A pivot could not be formed. Recoverable: the matrix is not factorizable by the requested
    /// method, which for Cholesky means it is not positive definite.
    FactorizationFailed,
    /// The matrix is structurally singular.
    MatrixIsSingular,
    /// Accelerate reported an internal failure or exhausted a resource.
    InternalError,
    /// A parameter failed Accelerate's own checks.
    ///
    /// Unexpected from a validated solve. Public paths include
    /// [`inertia`](crate::Factorization::inertia) on an unsupported kind and LU requested from a
    /// build whose SDK lacks it.
    ParameterError,
    /// An object was used after release. Unreachable through the safe API.
    Released,
    /// A solve was attempted against a factorization whose last attempt failed. Not an
    /// Accelerate code.
    NotFactored,
    /// A handle could not be allocated.
    AllocationFailed,
    /// The running OS, or the SDK this was built against, is too old for what was asked.
    ///
    /// Not an Accelerate code. Two operations carry a version floor:
    ///
    /// - LU needs macOS 15.5. An older OS reports this; an SDK without LU reports
    ///   [`ParameterError`](Self::ParameterError).
    /// - [`Factorization::inertia`](crate::Factorization::inertia) needs macOS 13.0. An older OS or
    ///   SDK reports this.
    UnsupportedOs,
}

impl Status {
    /// Maps a raw shim status, or `None` for success.
    pub(crate) fn from_raw(status: core::ffi::c_int) -> Option<Self> {
        match status {
            sys::ACCSP_STATUS_OK => None,
            sys::ACCSP_STATUS_FACTORIZATION_FAILED => Some(Self::FactorizationFailed),
            sys::ACCSP_STATUS_MATRIX_IS_SINGULAR => Some(Self::MatrixIsSingular),
            sys::ACCSP_STATUS_PARAMETER_ERROR => Some(Self::ParameterError),
            sys::ACCSP_STATUS_ALLOCATION_FAILED => Some(Self::AllocationFailed),
            sys::ACCSP_STATUS_NOT_FACTORED => Some(Self::NotFactored),
            sys::ACCSP_STATUS_UNSUPPORTED_OS => Some(Self::UnsupportedOs),
            // Accelerate spells "released" as -INT_MAX rather than as a small negative, so it is
            // matched by value here and everything else unrecognised folds into an internal
            // error rather than being silently dropped.
            s if s == -core::ffi::c_int::MAX => Some(Self::Released),
            _ => Some(Self::InternalError),
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::FactorizationFailed => "the matrix could not be factored by this method",
            Self::MatrixIsSingular => "the matrix is structurally singular",
            Self::InternalError => "Accelerate reported an internal error",
            Self::ParameterError => "Accelerate rejected a parameter",
            Self::Released => "the factorization was already released",
            Self::NotFactored => "the last factorization failed, so there is nothing to solve with",
            Self::AllocationFailed => "a factorization handle could not be allocated",
            Self::UnsupportedOs => "this factorization is not available on this OS version",
        };
        f.write_str(text)
    }
}

/// Identifies an operand's role in an operation.
///
/// Names the operand whose shape was incompatible without making callers parse an error message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OperandRole {
    /// The right-hand side of a solve.
    RightHandSide,
    /// The solution written by a solve.
    Solution,
    /// The single buffer used for an in-place operation.
    InPlace,
    /// The operand multiplied by a subfactor.
    Multiplicand,
    /// The result written by a subfactor multiplication.
    Product,
}

impl fmt::Display for OperandRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::RightHandSide => "right-hand side",
            Self::Solution => "solution",
            Self::InPlace => "in-place operand",
            Self::Multiplicand => "multiplicand",
            Self::Product => "product",
        };
        f.write_str(name)
    }
}

/// Identifies a dense-view dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DenseDimension {
    /// The number of rows.
    Rows,
    /// The number of columns.
    Columns,
}

impl fmt::Display for DenseDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rows => "rows",
            Self::Columns => "columns",
        })
    }
}

/// Identifies a dense-view field passed through Accelerate's integer ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DenseField {
    /// The row count.
    Rows,
    /// The column count.
    Columns,
    /// The column stride.
    ColumnStride,
}

impl fmt::Display for DenseField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rows => "row count",
            Self::Columns => "column count",
            Self::ColumnStride => "column stride",
        })
    }
}

/// Describes an input incompatibility checked locally.
///
/// These describe inputs the safe layer can reject before calling Accelerate. They are separate
/// from [`Status`], which reports outcomes from the framework or from factorization state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InputError {
    /// A factorization kind requiring a square matrix was given another shape.
    FactorizationRequiresSquare {
        /// Requested factorization kind.
        kind: FactorizationKind,
        /// Matrix rows.
        rows: usize,
        /// Matrix columns.
        columns: usize,
    },
    /// A factorization kind requiring at least as many rows as columns was given a wide matrix.
    FactorizationRequiresRowsAtLeastColumns {
        /// Requested factorization kind.
        kind: FactorizationKind,
        /// Matrix rows.
        rows: usize,
        /// Matrix columns.
        columns: usize,
    },
    /// An ordering was selected for a factorization kind it cannot order.
    OrderingUnavailable {
        /// Selected ordering.
        order: OrderMethod,
        /// Requested factorization kind.
        kind: FactorizationKind,
    },
    /// A values slice did not match the sparsity pattern's scalar entry count.
    ValuesLength {
        /// Entries required by the pattern.
        expected: usize,
        /// Entries supplied in the values slice.
        actual: usize,
    },
    /// A dense view had a zero row or column count.
    DenseZeroDimension {
        /// Whether the zero dimension was rows or columns.
        dimension: DenseDimension,
    },
    /// A dense view's column stride was smaller than its row count.
    DenseStrideTooSmall {
        /// Rows in the view.
        rows: usize,
        /// Column stride supplied for the view.
        column_stride: usize,
    },
    /// A dense view's required-storage calculation overflowed `usize`.
    DenseStorageArithmeticOverflow {
        /// Rows in the view.
        rows: usize,
        /// Columns in the view.
        columns: usize,
        /// Column stride supplied for the view.
        column_stride: usize,
    },
    /// A dense view's backing slice was too short for its declared shape.
    DenseStorageTooShort {
        /// Elements required by the declared shape.
        required: usize,
        /// Elements in the backing slice.
        actual: usize,
    },
    /// A dense view field could not be represented by Accelerate's integer ABI.
    DenseRepresentationOverflow {
        /// Field whose value does not fit.
        field: DenseField,
        /// Value that could not be represented.
        value: usize,
    },
    /// An operand had a row count incompatible with the operation.
    OperandRows {
        /// Role of the mismatched operand.
        operand: OperandRole,
        /// Rows required by the operation.
        expected: usize,
        /// Rows supplied by the operand.
        actual: usize,
    },
    /// Two operands carried different numbers of columns.
    OperandColumns {
        /// First operand's role.
        first: OperandRole,
        /// Columns carried by the first operand.
        first_columns: usize,
        /// Second operand's role.
        second: OperandRole,
        /// Columns carried by the second operand.
        second_columns: usize,
    },
    /// A factorization cannot supply the requested subfactor.
    SubfactorUnavailable {
        /// Requested subfactor.
        subfactor: SubfactorKind,
        /// Parent factorization kind.
        factorization: FactorizationKind,
    },
    /// A subfactor cannot be used for multiplication.
    MultiplyUnsupported {
        /// Subfactor that only supports solving.
        subfactor: SubfactorKind,
    },
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FactorizationRequiresSquare {
                kind,
                rows,
                columns,
            } => write!(
                f,
                "{kind:?} requires a square matrix, but the matrix is {rows} by {columns}"
            ),
            Self::FactorizationRequiresRowsAtLeastColumns {
                kind,
                rows,
                columns,
            } => write!(
                f,
                "{kind:?} requires at least as many rows as columns, but the matrix is {rows} by {columns}"
            ),
            Self::OrderingUnavailable { order, kind } => write!(
                f,
                "the {order:?} ordering does not apply to a {kind:?} factorization"
            ),
            Self::ValuesLength { expected, actual } => write!(
                f,
                "the values slice has {actual} scalar entries, but this pattern requires {expected}"
            ),
            Self::DenseZeroDimension { dimension } => {
                write!(f, "a dense view must have at least one {dimension}")
            }
            Self::DenseStrideTooSmall {
                rows,
                column_stride,
            } => write!(
                f,
                "column stride {column_stride} is smaller than the row count {rows}"
            ),
            Self::DenseStorageArithmeticOverflow {
                rows,
                columns,
                column_stride,
            } => write!(
                f,
                "a {rows} by {columns} dense view with column stride {column_stride} exceeds the addressable storage size"
            ),
            Self::DenseStorageTooShort { required, actual } => write!(
                f,
                "a dense view needs {required} elements, but the slice has {actual}"
            ),
            Self::DenseRepresentationOverflow { field, value } => write!(
                f,
                "dense view {field} {value} exceeds the width Accelerate accepts"
            ),
            Self::OperandRows {
                operand,
                expected,
                actual,
            } => write!(
                f,
                "the {operand} must have {expected} scalar rows, got {actual}"
            ),
            Self::OperandColumns {
                first,
                first_columns,
                second,
                second_columns,
            } => write!(
                f,
                "the {first} has {first_columns} columns, but the {second} has {second_columns}"
            ),
            Self::SubfactorUnavailable {
                subfactor,
                factorization,
            } => write!(
                f,
                "{factorization:?} cannot supply the {subfactor:?} subfactor"
            ),
            Self::MultiplyUnsupported { subfactor } => {
                write!(f, "the {subfactor:?} subfactor cannot be multiplied")
            }
        }
    }
}

impl std::error::Error for InputError {}

/// Represents a failure returned by this crate.
///
/// [`Input`](Self::Input) identifies an incompatible local input, reachable with
/// [`input`](Self::input); its `Display` forwards to the wrapped [`InputError`]. [`Status`](Self::Status)
/// carries an outcome reported by Accelerate or by this crate's factorization state, with an
/// optional diagnostic recorded by the framework callback. Neither variant reports a
/// [`source`](std::error::Error::source): the whole message is in the `Display`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A local input failed a check.
    Input(InputError),
    /// An outcome reported by Accelerate or factorization state.
    Status {
        /// The reported outcome.
        status: Status,
        /// Best-effort diagnostic recorded by Accelerate.
        detail: Option<String>,
    },
}

impl Error {
    pub(crate) fn with_detail(status: Status, detail: Option<String>) -> Self {
        Self::Status { status, detail }
    }

    /// Returns the framework or factorization status, if this error has one.
    pub fn status(&self) -> Option<Status> {
        match self {
            Self::Input(_) => None,
            Self::Status { status, .. } => Some(*status),
        }
    }

    /// Returns the local input failure, if this error has one.
    pub fn input(&self) -> Option<&InputError> {
        match self {
            Self::Input(input) => Some(input),
            Self::Status { .. } => None,
        }
    }

    /// Returns the diagnostic Accelerate produced, if it produced one.
    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::Input(_) => None,
            Self::Status { detail, .. } => detail.as_deref(),
        }
    }
}

impl From<Status> for Error {
    fn from(status: Status) -> Self {
        Self::Status {
            status,
            detail: None,
        }
    }
}

impl From<InputError> for Error {
    fn from(input: InputError) -> Self {
        Self::Input(input)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(input) => input.fmt(f),
            Self::Status {
                status,
                detail: Some(detail),
            } => write!(f, "{status}: {}", detail.trim_end()),
            Self::Status {
                status,
                detail: None,
            } => status.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            // Transparent over the wrapped input error: `Display` already carries its full
            // message, so forward the chain rather than re-emitting that message as a source.
            Self::Input(input) => input.source(),
            Self::Status { .. } => None,
        }
    }
}

/// Identifies the source of a value that overflowed Accelerate's stored width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IndexSource {
    /// The `column_starts` array, stored as `i64`.
    ColumnStarts,
    /// The `row_indices` array, stored as `i32`.
    RowIndices,
    /// The `column_indices` array of a coordinate list.
    ColumnIndices,
    /// The row count, stored as `i32`.
    Rows,
    /// The column count, stored as `i32`.
    Columns,
}

impl fmt::Display for IndexSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ColumnStarts => "column_starts",
            Self::RowIndices => "row_indices",
            Self::ColumnIndices => "column_indices",
            Self::Rows => "row count",
            Self::Columns => "column count",
        })
    }
}

/// Describes a sparsity pattern that Accelerate cannot accept.
///
/// Every variant is checked before the arrays reach the framework: past this layer they are raw
/// pointers into C, where a violation reads out of bounds instead of producing an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StructureError {
    /// A dimension was zero. Accelerate raises `SIGTRAP` on an empty matrix rather
    /// than reporting a failure, so it is rejected here instead.
    InvalidDimension {
        /// Rows given.
        rows: usize,
        /// Columns given.
        columns: usize,
    },
    /// A symmetric matrix was given unequal dimensions.
    NotSquare {
        /// Rows given.
        rows: usize,
        /// Columns given.
        columns: usize,
    },
    /// `column_starts` did not have one entry per column plus one.
    ColumnStartsLength {
        /// Length required, `columns + 1`.
        expected: usize,
        /// Length given.
        actual: usize,
    },
    /// `column_starts` did not begin at zero, which would leave the leading stored entries in no
    /// column at all.
    ColumnStartsNotZeroBased {
        /// First entry given.
        first: i64,
    },
    /// `column_starts` decreased, which would make a column's index range run backwards.
    ColumnStartsNotMonotone {
        /// Position of the offending entry.
        index: usize,
        /// Entry before it.
        previous: i64,
        /// Entry at `index`.
        current: i64,
    },
    /// The final column start disagreed with the number of row indices supplied.
    NonZeroCountMismatch {
        /// Final entry of `column_starts`.
        column_starts_end: i64,
        /// Number of row indices given.
        row_indices: usize,
    },
    /// A row index fell outside the matrix.
    RowIndexOutOfRange {
        /// Position within `row_indices`.
        position: usize,
        /// The offending index.
        row_index: i64,
        /// Rows in the matrix.
        rows: usize,
    },
    /// A coordinate entry's column index fell outside the matrix.
    ColumnIndexOutOfRange {
        /// Position within `column_indices`.
        position: usize,
        /// The offending index.
        column_index: i64,
        /// Columns in the matrix.
        columns: usize,
    },
    /// The slices of a coordinate list disagreed in length.
    CoordinateLengthMismatch {
        /// Entries in `row_indices`.
        row_indices: usize,
        /// Entries in `column_indices`.
        column_indices: usize,
        /// Entries in `values`.
        values: usize,
    },
    /// An index did not fit the width Accelerate stores it in: `i64` for column starts, `i32` for
    /// row indices and dimensions.
    IndexOverflow {
        /// Where the overflowing value came from.
        what: IndexSource,
    },
    /// The block size was zero.
    ZeroBlockSize,
}

impl fmt::Display for StructureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension { rows, columns } => write!(
                f,
                "matrix dimensions must be positive, got {rows} by {columns}"
            ),
            Self::NotSquare { rows, columns } => write!(
                f,
                "a symmetric matrix must be square, got {rows} by {columns}"
            ),
            Self::ColumnStartsLength { expected, actual } => write!(
                f,
                "column_starts must have {expected} entries, got {actual}"
            ),
            Self::ColumnStartsNotZeroBased { first } => {
                write!(f, "column_starts must begin at 0, got {first}")
            }
            Self::ColumnStartsNotMonotone {
                index,
                previous,
                current,
            } => write!(
                f,
                "column_starts must be non-decreasing, got {previous} then {current} at index {index}"
            ),
            Self::NonZeroCountMismatch {
                column_starts_end,
                row_indices,
            } => write!(
                f,
                "column_starts ends at {column_starts_end} but {row_indices} row indices were given"
            ),
            Self::RowIndexOutOfRange {
                position,
                row_index,
                rows,
            } => write!(
                f,
                "row index {row_index} at position {position} is outside a matrix with {rows} rows"
            ),
            Self::ColumnIndexOutOfRange {
                position,
                column_index,
                columns,
            } => write!(
                f,
                "column index {column_index} at position {position} is outside a matrix with {columns} columns"
            ),
            Self::CoordinateLengthMismatch {
                row_indices,
                column_indices,
                values,
            } => write!(
                f,
                "coordinate slices disagree in length: {row_indices} row indices, {column_indices} column indices, {values} values"
            ),
            Self::IndexOverflow { what } => match what {
                IndexSource::ColumnStarts
                | IndexSource::RowIndices
                | IndexSource::ColumnIndices => write!(
                    f,
                    "an index in {what} exceeds the width Accelerate stores it in"
                ),
                IndexSource::Rows | IndexSource::Columns => {
                    write!(f, "the {what} exceeds the width Accelerate stores it in")
                }
            },
            Self::ZeroBlockSize => f.write_str("block size must be at least 1"),
        }
    }
}

impl std::error::Error for StructureError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_non_empty_message() {
        for status in [
            Status::FactorizationFailed,
            Status::MatrixIsSingular,
            Status::InternalError,
            Status::ParameterError,
            Status::Released,
            Status::NotFactored,
            Status::AllocationFailed,
            Status::UnsupportedOs,
        ] {
            assert!(
                !status.to_string().is_empty(),
                "{status:?} formats to nothing"
            );
        }
    }

    /// Pins the raw-to-enum mapping for the crate's own status codes. These are the ones a caller
    /// can hit but whose runtime path is awkward to reach — `UnsupportedOs` in particular is only
    /// produced on an OS too old to test here, so its mapping is pinned directly instead.
    #[test]
    fn from_raw_maps_the_crate_status_codes() {
        assert_eq!(Status::from_raw(sys::ACCSP_STATUS_OK), None);
        assert_eq!(
            Status::from_raw(sys::ACCSP_STATUS_NOT_FACTORED),
            Some(Status::NotFactored)
        );
        assert_eq!(
            Status::from_raw(sys::ACCSP_STATUS_ALLOCATION_FAILED),
            Some(Status::AllocationFailed)
        );
        assert_eq!(
            Status::from_raw(sys::ACCSP_STATUS_UNSUPPORTED_OS),
            Some(Status::UnsupportedOs)
        );
    }

    #[test]
    fn error_carries_the_status_alone_when_there_is_no_detail() {
        let error = Error::with_detail(Status::FactorizationFailed, None);
        assert_eq!(error.status(), Some(Status::FactorizationFailed));
        assert_eq!(error.input(), None);
        assert_eq!(error.detail(), None);
        assert_eq!(error.to_string(), Status::FactorizationFailed.to_string());
    }

    #[test]
    fn error_keeps_the_detail_raw_and_trims_it_only_for_display() {
        let error = Error::with_detail(
            Status::ParameterError,
            Some("Accelerate said no\n".to_string()),
        );
        // `detail` is exactly what the callback recorded; `Display` trims the trailing newline.
        assert_eq!(error.detail(), Some("Accelerate said no\n"));
        assert_eq!(
            error.to_string(),
            format!("{}: Accelerate said no", Status::ParameterError)
        );
    }

    #[test]
    fn input_error_is_distinct_from_a_status_and_keeps_its_structure() {
        let input = InputError::OperandRows {
            operand: OperandRole::RightHandSide,
            expected: 3,
            actual: 2,
        };
        let error = Error::from(input.clone());

        assert_eq!(error.status(), None);
        assert_eq!(error.input(), Some(&input));
        assert_eq!(error.detail(), None);
        assert_eq!(
            error.to_string(),
            "the right-hand side must have 3 scalar rows, got 2"
        );
    }

    /// [`Error::Input`] is transparent over its [`InputError`]: the whole message is in `Display`,
    /// and `source` does not repeat it. Neither variant reports a source, so a chain-walking
    /// formatter renders the message exactly once.
    #[test]
    fn neither_variant_repeats_its_message_through_source() {
        let input = InputError::ValuesLength {
            expected: 5,
            actual: 2,
        };
        let expected = input.to_string();
        let error = Error::from(input);
        assert_eq!(error.to_string(), expected);
        assert!(std::error::Error::source(&error).is_none());

        let status = Error::from(Status::FactorizationFailed);
        assert!(std::error::Error::source(&status).is_none());
    }

    #[test]
    fn input_error_display_names_the_relevant_values() {
        let shown = InputError::ValuesLength {
            expected: 5,
            actual: 2,
        }
        .to_string();
        assert!(shown.contains('5') && shown.contains('2'), "got {shown:?}");
    }

    #[test]
    fn structure_error_display_names_what_disagreed() {
        assert!(
            StructureError::IndexOverflow {
                what: IndexSource::ColumnStarts,
            }
            .to_string()
            .contains("column_starts")
        );
        let shown = StructureError::NotSquare {
            rows: 3,
            columns: 2,
        }
        .to_string();
        assert!(shown.contains('3') && shown.contains('2'), "got {shown:?}");
    }
}
