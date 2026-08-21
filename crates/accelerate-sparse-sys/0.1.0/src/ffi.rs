//! These declarations mirror `shim.h`; shim `_Static_assert`s verify selectors on every build.

use core::ffi::{c_char, c_int, c_long, c_uint};

/// Success.
pub const ACCSP_STATUS_OK: c_int = 0;
/// A pivot could not be formed. Recoverable: the matrix is not factorizable by the requested
/// method, which for a Cholesky factorization means it is not positive definite.
pub const ACCSP_STATUS_FACTORIZATION_FAILED: c_int = -1;
/// The matrix is structurally singular.
pub const ACCSP_STATUS_MATRIX_IS_SINGULAR: c_int = -2;
/// Accelerate reported an internal failure or ran out of resources.
pub const ACCSP_STATUS_INTERNAL_ERROR: c_int = -3;
/// A parameter did not satisfy Accelerate's own checks.
pub const ACCSP_STATUS_PARAMETER_ERROR: c_int = -4;
/// A handle could not be allocated. Reported by the shim rather than by Accelerate, which is
/// never reached in this case.
pub const ACCSP_STATUS_ALLOCATION_FAILED: c_int = -100;
/// A solve was asked of a factorization whose last attempt failed. Reported by the shim:
/// Accelerate refuses this through its error callback rather than with a distinct code.
pub const ACCSP_STATUS_NOT_FACTORED: c_int = -101;
/// A factorization was requested that the running OS is too old to provide — LU below macOS 15.5.
/// Reported by the shim, which intercepts the case before Accelerate would trap on it.
pub const ACCSP_STATUS_UNSUPPORTED_OS: c_int = -102;

/// Cholesky (`LLᵀ`). Requires a symmetric positive-definite matrix.
pub const ACCSP_KIND_CHOLESKY: c_int = 0;
/// `LDLᵀ` with no pivoting (1×1 pivots only). Fails in the numeric phase when a pivot is too
/// small, so it suits matrices known not to need pivoting.
pub const ACCSP_KIND_LDLT_UNPIVOTED: c_int = 2;
/// `LDLᵀ` with supernode Bunch-Kaufman pivoting. Handles symmetric indefinite matrices.
pub const ACCSP_KIND_LDLT_SBK: c_int = 3;
/// `LDLᵀ` with full threshold partial pivoting. Handles symmetric indefinite matrices.
pub const ACCSP_KIND_LDLT_TPP: c_int = 4;
/// QR of a general `m × n` matrix. Solves the least-squares problem: the right-hand side has `m`
/// rows and the solution has `n`.
pub const ACCSP_KIND_QR: c_int = 40;
/// Cholesky of the normal equations `AᵀA = RᵀR`, keeping `R` and discarding the orthogonal
/// factor. The right-hand side and solution both have `n` rows, where `n` is the column count
/// of `A`.
pub const ACCSP_KIND_CHOLESKY_ATA: c_int = 41;
/// LU of a general square matrix, no pivoting. Needs the macOS 15.5 SDK to build and that OS to
/// run; on an older OS the shim reports [`ACCSP_STATUS_UNSUPPORTED_OS`] rather than trapping.
pub const ACCSP_KIND_LU_UNPIVOTED: c_int = 81;
/// LU of a general square matrix, pivoting confined to each supernode rather than searching a
/// whole column. Same OS requirement as [`ACCSP_KIND_LU_UNPIVOTED`].
pub const ACCSP_KIND_LU_SPP: c_int = 82;
/// LU of a general square matrix, threshold partial pivoting. Same OS requirement as
/// [`ACCSP_KIND_LU_UNPIVOTED`].
pub const ACCSP_KIND_LU_TPP: c_int = 83;

/// No structural symmetry is assumed; every stored entry is read.
pub const ACCSP_MATRIX_ORDINARY: c_int = 0;
/// Symmetric, with one triangle read and the other implied.
pub const ACCSP_MATRIX_SYMMETRIC: c_int = 3;
/// Read the upper triangle.
pub const ACCSP_TRIANGLE_UPPER: c_int = 0;
/// Read the lower triangle.
pub const ACCSP_TRIANGLE_LOWER: c_int = 1;

/// Let Accelerate choose the fill-reducing ordering.
pub const ACCSP_ORDER_DEFAULT: u8 = 0;
/// Use a caller-supplied permutation.
pub const ACCSP_ORDER_USER: u8 = 1;
/// Approximate minimum degree.
pub const ACCSP_ORDER_AMD: u8 = 2;
/// Metis nested dissection.
pub const ACCSP_ORDER_METIS: u8 = 3;
/// Column approximate minimum degree.
///
/// Intended for the normal equations, not for a symmetric factorization. Pairing it with one does
/// not fail: Accelerate spins forever inside the numeric phase once the matrix is large enough to
/// reach its supernodal path.
pub const ACCSP_ORDER_COLAMD: u8 = 4;

/// Not a subfactor. Returned by Accelerate when a factorization cannot supply the piece asked for.
pub const ACCSP_SUBFACTOR_INVALID: u8 = 0;
/// The permutation. Available from every factorization kind; the row permutation for LU.
pub const ACCSP_SUBFACTOR_P: u8 = 1;
/// The diagonal scaling. Available only where scaling was actually applied, which under
/// Accelerate's default options means `LDLᵀ` but not Cholesky.
pub const ACCSP_SUBFACTOR_S: u8 = 2;
/// The `L` factor of a Cholesky or `LDLᵀ` factorization.
pub const ACCSP_SUBFACTOR_L: u8 = 3;
/// The `D` factor of an `LDLᵀ` factorization.
pub const ACCSP_SUBFACTOR_D: u8 = 4;
/// The half-solve of a Cholesky or `LDLᵀ` factorization: `PLP'` applied, `PLDP'` transposed.
/// Solve only.
pub const ACCSP_SUBFACTOR_PLPS: u8 = 5;
/// The orthogonal factor of a QR factorization, or the column permutation of an LU one.
pub const ACCSP_SUBFACTOR_Q: u8 = 6;
/// The `R` factor of a QR or `CholeskyAtA` factorization.
pub const ACCSP_SUBFACTOR_R: u8 = 7;
/// The half-solve of a QR or `CholeskyAtA` factorization. Solve only.
pub const ACCSP_SUBFACTOR_RP: u8 = 8;
/// The row scaling of an LU factorization. Needs the macOS 15.5 SDK to be drift-checked, though
/// the value is always declared.
pub const ACCSP_SUBFACTOR_SR: u8 = 9;
/// The column scaling of an LU factorization. As [`ACCSP_SUBFACTOR_SR`].
pub const ACCSP_SUBFACTOR_SC: u8 = 10;

/// Opaque symbolic factorization handle.
#[repr(C)]
pub struct accsp_symbolic_t {
    _private: [u8; 0],
}

/// Opaque double-precision subfactor handle.
#[repr(C)]
pub struct accsp_subfactor_d_t {
    _private: [u8; 0],
}

/// Opaque single-precision subfactor handle.
#[repr(C)]
pub struct accsp_subfactor_f_t {
    _private: [u8; 0],
}

/// Opaque double-precision numeric factorization handle.
#[repr(C)]
pub struct accsp_numeric_d_t {
    _private: [u8; 0],
}

/// Opaque single-precision numeric factorization handle.
#[repr(C)]
pub struct accsp_numeric_f_t {
    _private: [u8; 0],
}

/// Controls how stored matrix entries are interpreted.
///
/// `triangle` is meaningful only when `kind` selects a symmetric or triangular matrix. The fields
/// are semantic selectors; the shim assembles Accelerate's implementation-defined bitfield.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct accsp_attributes {
    /// One of the `ACCSP_MATRIX_*` selectors.
    pub kind: c_int,
    /// One of the `ACCSP_TRIANGLE_*` selectors.
    pub triangle: c_int,
    /// Non-zero to have Accelerate treat the matrix as transposed.
    pub transpose: c_int,
    /// Entries per block edge; `1` for a scalar matrix.
    pub block_size: u8,
}

/// Describes a column-major dense operand with an explicit column stride.
///
/// `column_stride` is in elements and must be at least `row_count`, which is what lets a
/// right-hand side block that is a slice of a larger matrix pass through without a copy.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct accsp_dense_d {
    /// Rows in the operand.
    pub row_count: c_int,
    /// Columns in the operand, i.e. the number of right-hand sides.
    pub column_count: c_int,
    /// Elements between the starts of consecutive columns.
    pub column_stride: c_int,
    /// Element storage, at least `column_stride * column_count` long.
    pub data: *mut f64,
}

/// Describes a single-precision dense operand, as [`accsp_dense_d`].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct accsp_dense_f {
    /// Rows in the operand.
    pub row_count: c_int,
    /// Columns in the operand, i.e. the number of right-hand sides.
    pub column_count: c_int,
    /// Elements between the starts of consecutive columns.
    pub column_stride: c_int,
    /// Element storage, at least `column_stride * column_count` long.
    pub data: *mut f32,
}

/// Options for the symbolic phase. A null pointer selects Accelerate's defaults, which a zeroed
/// struct does not reproduce.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct accsp_symbolic_options {
    /// Reserved by Accelerate; zero selects its default behaviour.
    pub control: c_uint,
    /// One of the `ACCSP_ORDER_*` selectors.
    pub order_method: u8,
}

/// Options for the numeric phase. A null pointer selects Accelerate's defaults, which a zeroed
/// struct does not reproduce: both tolerances default to non-zero values.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct accsp_numeric_options {
    /// Reserved by Accelerate; zero selects its default behaviour.
    pub control: c_uint,
    /// Scaling strategy selector.
    pub scaling_method: u8,
    /// Relative threshold below which a pivot is rejected.
    pub pivot_tolerance: f64,
    /// Magnitude at or below which an entry is treated as zero.
    pub zero_tolerance: f64,
}

unsafe extern "C" {
    /// Fills `out` with Accelerate's default symbolic-phase options.
    ///
    /// # Safety
    ///
    /// `out` must be a writable, suitably aligned [`accsp_symbolic_options`].
    pub fn accsp_default_symbolic_options(out: *mut accsp_symbolic_options);

    /// Fills `out` with Accelerate's default double-precision numeric-phase options.
    ///
    /// A zeroed struct is not equivalent: both tolerances default to non-zero values.
    ///
    /// # Safety
    ///
    /// `out` must be a writable, suitably aligned [`accsp_numeric_options`].
    pub fn accsp_default_numeric_options_d(out: *mut accsp_numeric_options);

    /// Fills `out` with Accelerate's default single-precision numeric-phase options.
    ///
    /// These are not the double-precision defaults: both tolerances are looser, in step with the
    /// element type's own precision.
    ///
    /// # Safety
    ///
    /// `out` must be a writable, suitably aligned [`accsp_numeric_options`].
    pub fn accsp_default_numeric_options_f(out: *mut accsp_numeric_options);

    /// Installs the callback Accelerate reports parameter violations through.
    ///
    /// Without one, Accelerate's parameter checks abort the process rather than returning a
    /// status, because its default options leave the callback unset. The shim installs the
    /// registered handler on every options struct it builds.
    ///
    /// # Safety
    ///
    /// `handler` must be callable from any thread and must not unwind. Passing `None` restores
    /// the do-nothing default, which still returns normally rather than trapping.
    pub fn accsp_set_error_handler(handler: Option<unsafe extern "C" fn(message: *const c_char)>);

    /// Analyses a sparsity pattern.
    ///
    /// Writes the outcome through `out_status` and returns null unless it is
    /// [`ACCSP_STATUS_OK`]. The arrays are copied and need not outlive the call.
    ///
    /// For an LU `kind` on an OS older than macOS 15.5, writes [`ACCSP_STATUS_UNSUPPORTED_OS`]
    /// and returns null rather than letting Accelerate trap.
    ///
    /// # Safety
    ///
    /// `column_starts` must have `columns + 1` readable entries and `row_indices` must have
    /// `column_starts[columns]`. The pattern must be one Accelerate can traverse without reading
    /// outside those arrays: non-decreasing column starts, and row indices within `rows`.
    ///
    /// For an LU `kind` this is **not thread-safe**: concurrent calls corrupt Accelerate state.
    /// The caller must serialize LU analyses. Other kinds have no such restriction.
    pub fn accsp_symbolic_new(
        kind: c_int,
        rows: c_int,
        columns: c_int,
        column_starts: *const c_long,
        row_indices: *const c_int,
        attributes: *const accsp_attributes,
        options: *const accsp_symbolic_options,
        out_status: *mut c_int,
    ) -> *mut accsp_symbolic_t;

    /// Releases a symbolic factorization.
    ///
    /// Numeric factorizations built from it stay valid: Accelerate retains the analysis inside
    /// each of them.
    ///
    /// # Safety
    ///
    /// `symbolic` must be null, or a handle from [`accsp_symbolic_new`] not yet freed.
    pub fn accsp_symbolic_free(symbolic: *mut accsp_symbolic_t);

    /// Returns the bytes Accelerate reports it needs for a double-precision numeric
    /// factorization built from this analysis.
    ///
    /// # Safety
    ///
    /// `symbolic` must be a live handle from [`accsp_symbolic_new`].
    pub fn accsp_symbolic_factor_size_d(symbolic: *const accsp_symbolic_t) -> usize;

    /// Returns the bytes Accelerate reports it needs for a single-precision numeric factorization
    /// built from this analysis.
    ///
    /// One analysis serves both element types and carries a separate figure for each.
    ///
    /// # Safety
    ///
    /// `symbolic` must be a live handle from [`accsp_symbolic_new`].
    pub fn accsp_symbolic_factor_size_f(symbolic: *const accsp_symbolic_t) -> usize;

    /// Numerically factors `values` against a previous analysis.
    ///
    /// Writes the outcome through `out_status` and returns null unless it is
    /// [`ACCSP_STATUS_OK`]. The returned handle keeps its own copy of the pattern and stays valid
    /// after `symbolic` is freed.
    ///
    /// # Safety
    ///
    /// `symbolic` must be a live handle, and `values` must have one readable entry per stored
    /// non-zero of the pattern it was built from.
    pub fn accsp_numeric_new_d(
        symbolic: *const accsp_symbolic_t,
        values: *const f64,
        options: *const accsp_numeric_options,
        out_status: *mut c_int,
    ) -> *mut accsp_numeric_d_t;

    /// Numerically factors single-precision `values`, as [`accsp_numeric_new_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_numeric_new_d`].
    pub fn accsp_numeric_new_f(
        symbolic: *const accsp_symbolic_t,
        values: *const f32,
        options: *const accsp_numeric_options,
        out_status: *mut c_int,
    ) -> *mut accsp_numeric_f_t;

    /// Re-forms a factorization from new values for the same pattern.
    ///
    /// A failure leaves the handle allocated but unfactored: another refactor may be attempted,
    /// and is the intended recovery, while solving is refused until one succeeds.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle and `values` must have one readable entry per stored
    /// non-zero of its pattern.
    pub fn accsp_numeric_refactor_d(
        numeric: *mut accsp_numeric_d_t,
        values: *const f64,
        options: *const accsp_numeric_options,
    ) -> c_int;

    /// Re-forms a single-precision factorization, as [`accsp_numeric_refactor_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_numeric_refactor_d`].
    pub fn accsp_numeric_refactor_f(
        numeric: *mut accsp_numeric_f_t,
        values: *const f32,
        options: *const accsp_numeric_options,
    ) -> c_int;

    /// Releases a numeric factorization.
    ///
    /// # Safety
    ///
    /// `numeric` must be null, or a handle from [`accsp_numeric_new_d`] not yet freed.
    pub fn accsp_numeric_free_d(numeric: *mut accsp_numeric_d_t);

    /// Releases a single-precision numeric factorization.
    ///
    /// # Safety
    ///
    /// `numeric` must be null, or a handle from [`accsp_numeric_new_f`] not yet freed.
    pub fn accsp_numeric_free_f(numeric: *mut accsp_numeric_f_t);

    /// Returns the status of the last factorization performed on this handle.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle from [`accsp_numeric_new_d`].
    pub fn accsp_numeric_status_d(numeric: *const accsp_numeric_d_t) -> c_int;

    /// Returns the status of the last factorization performed on this handle.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle from [`accsp_numeric_new_f`].
    pub fn accsp_numeric_status_f(numeric: *const accsp_numeric_f_t) -> c_int;

    /// Solves with the right-hand side and the solution in separate operands.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle. If its effective scalar matrix shape is `m × n`, `x` must
    /// have `n` rows; `b` must have `m` rows for QR and `n` for every other factorization kind.
    /// Both operands must have the same column count and own enough elements for their declared
    /// shapes, readable for `b` and writable for `x`.
    pub fn accsp_solve_d(
        numeric: *const accsp_numeric_d_t,
        b: *const accsp_dense_d,
        x: *const accsp_dense_d,
    ) -> c_int;

    /// Solves in single precision, as [`accsp_solve_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_solve_d`].
    pub fn accsp_solve_f(
        numeric: *const accsp_numeric_f_t,
        b: *const accsp_dense_f,
        x: *const accsp_dense_f,
    ) -> c_int;

    /// Solves with the right-hand side overwritten by the solution.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle. If its effective scalar matrix shape is `m × n`, `xb` must
    /// have `max(m, n)` rows and be readable and writable. The input occupies the leading rows
    /// required by the factorization kind and the `n`-row solution is written at the beginning.
    ///
    /// The larger physical shape is required even when both logical vectors are shorter. For a
    /// tall `CholeskyAtA` factorization, passing only `n` rows is observed to return success without
    /// writing the solution.
    pub fn accsp_solve_in_place_d(
        numeric: *const accsp_numeric_d_t,
        xb: *const accsp_dense_d,
    ) -> c_int;

    /// Solves in single precision with the right-hand side overwritten by the solution.
    ///
    /// # Safety
    ///
    /// As [`accsp_solve_in_place_d`], with a single-precision factorization and operand.
    pub fn accsp_solve_in_place_f(
        numeric: *const accsp_numeric_f_t,
        xb: *const accsp_dense_f,
    ) -> c_int;

    /// Counts the positive, zero and negative pivots of a factorization.
    ///
    /// Accelerate accepts this only for a factorization built as [`ACCSP_KIND_LDLT_TPP`] and
    /// rejects every other kind, which is reported as [`ACCSP_STATUS_PARAMETER_ERROR`]. Returns
    /// [`ACCSP_STATUS_UNSUPPORTED_OS`] below macOS 13.0 or when the building SDK did not provide
    /// the underlying function, and [`ACCSP_STATUS_NOT_FACTORED`] if the last factorization
    /// attempt failed — which is checked before either of the others.
    ///
    /// The counts are written only when [`ACCSP_STATUS_OK`] is returned. They are in scalars, so
    /// they sum to the matrix dimension in scalars whatever the block size — *observed*, since
    /// Accelerate documents the counts only as pivots.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle from [`accsp_numeric_new_d`], and each out-parameter must
    /// be non-null and point to a writable, suitably aligned `int`.
    pub fn accsp_get_inertia_d(
        numeric: *const accsp_numeric_d_t,
        positive: *mut c_int,
        zero: *mut c_int,
        negative: *mut c_int,
    ) -> c_int;

    /// Counts pivots in single precision, as [`accsp_get_inertia_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_get_inertia_d`], with `numeric` from [`accsp_numeric_new_f`].
    pub fn accsp_get_inertia_f(
        numeric: *const accsp_numeric_f_t,
        positive: *mut c_int,
        zero: *mut c_int,
        negative: *mut c_int,
    ) -> c_int;

    /// Extracts one piece of a factorization as an object the operations below accept.
    ///
    /// `subfactor` is one of the `ACCSP_SUBFACTOR_*` values. Writes the outcome through
    /// `out_status` and returns null unless it is [`ACCSP_STATUS_OK`]:
    /// [`ACCSP_STATUS_PARAMETER_ERROR`] when the factorization has no such piece, and
    /// [`ACCSP_STATUS_NOT_FACTORED`] if its last attempt failed. The handle keeps the
    /// factorization alive, so it outlives the numeric handle it came from.
    ///
    /// The subfactor applies its parent's live state, so a successful refactor changes its
    /// operations. A failed refactor invalidates its results without changing its reported status;
    /// do not use it after one.
    ///
    /// # Safety
    ///
    /// `numeric` must be a live handle from [`accsp_numeric_new_d`], `out_status` must be writable,
    /// and the returned handle must be released with [`accsp_subfactor_free_d`].
    pub fn accsp_subfactor_new_d(
        numeric: *const accsp_numeric_d_t,
        subfactor: u8,
        out_status: *mut c_int,
    ) -> *mut accsp_subfactor_d_t;

    /// Extracts a subfactor in single precision, as [`accsp_subfactor_new_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_new_d`], with `numeric` from [`accsp_numeric_new_f`].
    pub fn accsp_subfactor_new_f(
        numeric: *const accsp_numeric_f_t,
        subfactor: u8,
        out_status: *mut c_int,
    ) -> *mut accsp_subfactor_f_t;

    /// A separately reference-counted handle on the transpose of a subfactor.
    ///
    /// The original stays valid, and both must be freed.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live, `out_status` must be writable, and the result must be released
    /// with [`accsp_subfactor_free_d`].
    pub fn accsp_subfactor_transpose_d(
        subfactor: *const accsp_subfactor_d_t,
        out_status: *mut c_int,
    ) -> *mut accsp_subfactor_d_t;

    /// Transposes a subfactor in single precision, as [`accsp_subfactor_transpose_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_transpose_d`].
    pub fn accsp_subfactor_transpose_f(
        subfactor: *const accsp_subfactor_f_t,
        out_status: *mut c_int,
    ) -> *mut accsp_subfactor_f_t;

    /// Releases a subfactor handle.
    ///
    /// # Safety
    ///
    /// `subfactor` must come from [`accsp_subfactor_new_d`] or [`accsp_subfactor_transpose_d`] and
    /// must not have been released already. Null is accepted and ignored.
    pub fn accsp_subfactor_free_d(subfactor: *mut accsp_subfactor_d_t);

    /// Releases a single-precision subfactor handle.
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_free_d`].
    pub fn accsp_subfactor_free_f(subfactor: *mut accsp_subfactor_f_t);

    /// Which piece this handle holds, as one of the `ACCSP_SUBFACTOR_*` values.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live.
    pub fn accsp_subfactor_contents_d(subfactor: *const accsp_subfactor_d_t) -> u8;

    /// Which piece a single-precision handle holds, as [`accsp_subfactor_contents_d`].
    ///
    /// # Safety
    ///
    /// `subfactor` must be live.
    pub fn accsp_subfactor_contents_f(subfactor: *const accsp_subfactor_f_t) -> u8;

    /// Whether this subfactor is to be applied transposed.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live.
    pub fn accsp_subfactor_is_transposed_d(subfactor: *const accsp_subfactor_d_t) -> c_int;

    /// Whether a single-precision subfactor is transposed, as
    /// [`accsp_subfactor_is_transposed_d`].
    ///
    /// # Safety
    ///
    /// `subfactor` must be live.
    pub fn accsp_subfactor_is_transposed_f(subfactor: *const accsp_subfactor_f_t) -> c_int;

    /// Scratch space an operation on this subfactor needs, as
    /// `static_bytes + columns * per_rhs_bytes`.
    ///
    /// The operations below allocate this on every call.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live and both out-parameters writable.
    pub fn accsp_subfactor_workspace_d(
        subfactor: *const accsp_subfactor_d_t,
        static_bytes: *mut usize,
        per_rhs_bytes: *mut usize,
    );

    /// Scratch space for a single-precision subfactor, as [`accsp_subfactor_workspace_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_workspace_d`].
    pub fn accsp_subfactor_workspace_f(
        subfactor: *const accsp_subfactor_f_t,
        static_bytes: *mut usize,
        per_rhs_bytes: *mut usize,
    );

    /// Solves `subfactor * x = b`.
    ///
    /// A shape mismatch is reported only through the error callback and leaves both operands
    /// untouched. This layer does not validate shapes.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live. If it is `m` by `n`, `b` must have `m` rows and `x` must have `n`,
    /// both with the same column count, and each must own `column_stride * column_count` elements.
    /// The shim header tabulates `m` and `n` per parent kind and subfactor; the safe crate derives
    /// and checks them.
    pub fn accsp_subfactor_solve_d(
        subfactor: *const accsp_subfactor_d_t,
        b: *const accsp_dense_d,
        x: *const accsp_dense_d,
    ) -> c_int;

    /// Solves in single precision, as [`accsp_subfactor_solve_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_solve_d`].
    pub fn accsp_subfactor_solve_f(
        subfactor: *const accsp_subfactor_f_t,
        b: *const accsp_dense_f,
        x: *const accsp_dense_f,
    ) -> c_int;

    /// Solves with the right-hand side overwritten by the solution.
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_solve_d`], with one operand of `max(m, n)` rows that is both read and
    /// written.
    pub fn accsp_subfactor_solve_in_place_d(
        subfactor: *const accsp_subfactor_d_t,
        xb: *const accsp_dense_d,
    ) -> c_int;

    /// Solves in place in single precision, as [`accsp_subfactor_solve_in_place_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_solve_in_place_d`].
    pub fn accsp_subfactor_solve_in_place_f(
        subfactor: *const accsp_subfactor_f_t,
        xb: *const accsp_dense_f,
    ) -> c_int;

    /// Multiplies `y = subfactor * x`.
    ///
    /// The half-solve selectors are solve-only. [`ACCSP_SUBFACTOR_PLPS`] is refused by the shim,
    /// because letting it reach Accelerate traps the process; [`ACCSP_SUBFACTOR_RP`] is passed
    /// through and reported by Accelerate.
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_solve_d`], with the roles reversed: `x` must have `n` rows and `y`
    /// must have `m`.
    pub fn accsp_subfactor_multiply_d(
        subfactor: *const accsp_subfactor_d_t,
        x: *const accsp_dense_d,
        y: *const accsp_dense_d,
    ) -> c_int;

    /// Multiplies in single precision, as [`accsp_subfactor_multiply_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_multiply_d`].
    pub fn accsp_subfactor_multiply_f(
        subfactor: *const accsp_subfactor_f_t,
        x: *const accsp_dense_f,
        y: *const accsp_dense_f,
    ) -> c_int;

    /// Multiplies with the operand overwritten by the result.
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_multiply_d`], with one operand of `max(m, n)` rows.
    pub fn accsp_subfactor_multiply_in_place_d(
        subfactor: *const accsp_subfactor_d_t,
        xy: *const accsp_dense_d,
    ) -> c_int;

    /// Multiplies in place in single precision, as [`accsp_subfactor_multiply_in_place_d`].
    ///
    /// # Safety
    ///
    /// As [`accsp_subfactor_multiply_in_place_d`].
    pub fn accsp_subfactor_multiply_in_place_f(
        subfactor: *const accsp_subfactor_f_t,
        xy: *const accsp_dense_f,
    ) -> c_int;
}
