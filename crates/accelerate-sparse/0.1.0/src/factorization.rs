//! Symbolic analysis and numeric factorization are exposed as separate types.

use accelerate_sparse_sys as sys;
use core::ffi::{c_int, c_void};
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::sync::{Mutex, PoisonError};

use crate::dense::{DenseMut, DenseRef};
use crate::diagnostics;
use crate::scalar::Scalar;
use crate::structure::SparseStructure;
use crate::{
    FactorizationKind, SubfactorKind,
    error::{Error, InputError, OperandRole, Status},
    options::{NumericOptions, SymbolicOptions},
};

/// Serializes LU symbolic factorization.
///
/// Concurrent LU symbolic calls corrupt Accelerate state and crash the process. This observed
/// restriction does not apply to other kinds or to numeric factorization, refactor, and solve. The
/// lock protects no Rust invariant, so poisoning is ignored.
static LU_SYMBOLIC_LOCK: Mutex<()> = Mutex::new(());

/// Serializes the calls that change a *factorization's* reference count.
///
/// Subfactor extraction, transposition, and release change the parent's undocumented, non-atomic
/// reference count and are reachable through a shared [`Factorization`].
///
/// It does **not** cover the analysis's reference count, which
/// [`SymbolicFactorization::factorize`] retains and both destructors release: the analysis is
/// neither [`Send`] nor [`Sync`], so every mutation of its count stays on the creating thread.
///
/// Other operations remain concurrent. The lock protects no Rust invariant, so poisoning is
/// ignored.
static REFCOUNT_LOCK: Mutex<()> = Mutex::new(());

/// Runs `f` with [`REFCOUNT_LOCK`] held.
fn with_refcount_lock<R>(f: impl FnOnce() -> R) -> R {
    let _guard = REFCOUNT_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    f()
}

/// Turns a raw status into a result, attaching whatever the error callback recorded.
fn finish(status: c_int) -> Result<(), Error> {
    match Status::from_raw(status) {
        None => Ok(()),
        Some(status) => Err(Error::with_detail(status, diagnostics::take())),
    }
}

/// Turns the outcome of a subfactor application into a result.
///
/// Subfactor entry points report a rejected shape only through the callback and leave operands
/// unchanged; the shim otherwise returns success. Since shapes are checked locally, a recorded
/// error means this crate's shape model disagreed with Accelerate's.
fn finish_application(status: c_int) -> Result<(), Error> {
    finish(status)?;
    match diagnostics::take() {
        None => Ok(()),
        Some(detail) => Err(Error::with_detail(Status::ParameterError, Some(detail))),
    }
}

/// Returns the effective matrix dimensions in scalars.
fn effective_scalar_dimensions(
    rows: i32,
    columns: i32,
    block_size: u8,
    transpose: bool,
) -> (usize, usize) {
    let block = usize::from(block_size);
    let rows = rows as usize * block;
    let columns = columns as usize * block;
    if transpose {
        (columns, rows)
    } else {
        (rows, columns)
    }
}

/// Returns the row count required by a right-hand side.
fn required_right_hand_side_rows(
    kind: FactorizationKind,
    effective_rows: usize,
    effective_columns: usize,
) -> usize {
    if matches!(kind, FactorizationKind::Qr) {
        effective_rows
    } else {
        effective_columns
    }
}

/// Stores the reusable analysis of a fixed sparsity pattern.
///
/// The ordering and the elimination structure depend only on where the non-zeros are, so this is
/// computed once and reused for every set of values that shares the pattern.
///
/// # Thread safety
///
/// Neither [`Send`] nor [`Sync`]. Concurrent factorization would race Accelerate's undocumented,
/// non-atomic reference count. A [`Factorization`] retains and releases the analysis without
/// borrowing it, so the analysis must also stay on its creating thread.
///
/// `compile_fail` tests pin both bounds:
///
/// ```compile_fail
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<accelerate_sparse::SymbolicFactorization>();
/// ```
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<accelerate_sparse::SymbolicFactorization>();
/// ```
pub struct SymbolicFactorization {
    handle: NonNull<sys::accsp_symbolic_t>,
    kind: FactorizationKind,
    rows: i32,
    columns: i32,
    block_size: u8,
    transpose: bool,
    value_count: usize,
}

// The handle holds a `NonNull`, which is neither `Send` nor `Sync`; both are left unimplemented so
// the analysis stays on its creating thread. See the type's `# Thread safety` docs for why moving
// it would let two threads drive one Accelerate reference count.

impl SymbolicFactorization {
    /// Analyses `structure` for `kind`.
    ///
    /// # Errors
    ///
    /// Returns [`InputError`] when `kind` and the structure shape are incompatible, or when the
    /// selected ordering does not apply to `kind`. The `CholeskyAtA` shape check uses the effective
    /// dimensions after any [`with_transpose`](crate::Attributes::with_transpose) attribute. Otherwise
    /// returns the [`Status`] Accelerate reported, with any diagnostic it produced.
    pub fn new(kind: FactorizationKind, structure: &SparseStructure<'_>) -> Result<Self, Error> {
        Self::with_options(kind, structure, SymbolicOptions::new())
    }

    /// Analyses `structure` for `kind` with explicit options.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new).
    pub fn with_options(
        kind: FactorizationKind,
        structure: &SparseStructure<'_>,
        options: SymbolicOptions,
    ) -> Result<Self, Error> {
        if kind.requires_square() && structure.rows() != structure.columns() {
            return Err(InputError::FactorizationRequiresSquare {
                kind,
                rows: structure.rows(),
                columns: structure.columns(),
            }
            .into());
        }
        // The row-versus-column precondition is on the effective shape, so apply any transpose
        // attribute first, the same way the operand-sizing dimensions do.
        let (effective_rows, effective_columns) = if structure.attributes().is_transposed() {
            (structure.columns(), structure.rows())
        } else {
            (structure.rows(), structure.columns())
        };
        if kind.requires_rows_ge_columns() && effective_rows < effective_columns {
            return Err(InputError::FactorizationRequiresRowsAtLeastColumns {
                kind,
                rows: effective_rows,
                columns: effective_columns,
            }
            .into());
        }
        if let Some(order) = options.chosen_order() {
            if !order.applies_to(kind) {
                return Err(InputError::OrderingUnavailable { order, kind }.into());
            }
        }

        diagnostics::install();
        diagnostics::clear();

        let attributes = structure.raw_attributes();
        let raw_options = options.to_raw();
        let mut status = 0;
        // Serialize the LU symbolic phase, which Accelerate cannot run concurrently; other kinds
        // are left to run in parallel. Held only across the call below.
        let lu_guard = kind.is_lu().then(|| {
            LU_SYMBOLIC_LOCK
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
        });
        // SAFETY: the structure validated its own arrays at construction, so `column_starts` has
        // `columns + 1` entries, is non-decreasing, and ends at the row-index count, and every
        // row index is inside the matrix. The shim copies both arrays, so neither has to outlive
        // this call. `attributes` and `raw_options` are live locals for its duration.
        let handle = unsafe {
            sys::accsp_symbolic_new(
                kind.to_raw(),
                structure.raw_rows(),
                structure.raw_columns(),
                structure.column_starts().as_ptr(),
                structure.row_indices().as_ptr(),
                &attributes,
                raw_options
                    .as_ref()
                    .map_or(core::ptr::null(), |options| options),
                &mut status,
            )
        };
        drop(lu_guard);

        finish(status)?;
        let handle = NonNull::new(handle).ok_or_else(|| {
            // Reaching this means the shim reported success but produced no handle, which it does
            // not do; treating it as an internal error beats unwrapping.
            Error::with_detail(Status::InternalError, diagnostics::take())
        })?;

        Ok(Self {
            handle,
            kind,
            rows: structure.raw_rows(),
            columns: structure.raw_columns(),
            block_size: structure.block_size(),
            transpose: structure.attributes().is_transposed(),
            value_count: structure.value_count(),
        })
    }

    /// The factorization this analysis was built for.
    pub fn kind(&self) -> FactorizationKind {
        self.kind
    }

    /// Rows in the stored pattern, in blocks if the block size is above one.
    ///
    /// This does not apply the pattern's transpose attribute. See
    /// [`effective_rows`](Self::effective_rows) for the matrix's scalar row count.
    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    /// Columns in the stored pattern, in blocks if the block size is above one.
    ///
    /// This does not apply the pattern's transpose attribute. See
    /// [`effective_columns`](Self::effective_columns) for the matrix's scalar column count.
    pub fn columns(&self) -> usize {
        self.columns as usize
    }

    /// Entries per block edge, as the analysed structure declared it.
    pub fn block_size(&self) -> u8 {
        self.block_size
    }

    /// Rows in the effective matrix, after applying its transpose attribute, in scalars.
    pub fn effective_rows(&self) -> usize {
        effective_scalar_dimensions(self.rows, self.columns, self.block_size, self.transpose).0
    }

    /// Columns in the effective matrix, after applying its transpose attribute, in scalars.
    pub fn effective_columns(&self) -> usize {
        effective_scalar_dimensions(self.rows, self.columns, self.block_size, self.transpose).1
    }

    /// Scalar rows a right-hand side for this analysis must have.
    ///
    /// This is [`effective_rows`](Self::effective_rows) for [`FactorizationKind::Qr`] and
    /// [`effective_columns`](Self::effective_columns) for every other kind. For
    /// [`FactorizationKind::CholeskyAtA`] the operand this counts is the reduced right-hand side
    /// `Aᵀb`; the count cannot enforce that when the matrix is square — see that kind.
    pub fn right_hand_side_rows(&self) -> usize {
        required_right_hand_side_rows(self.kind, self.effective_rows(), self.effective_columns())
    }

    /// Scalar rows a solution produced from this analysis will have.
    ///
    /// This is always [`effective_columns`](Self::effective_columns).
    pub fn solution_rows(&self) -> usize {
        self.effective_columns()
    }

    /// Scalar rows an in-place solve operand built from this analysis must have.
    ///
    /// This is the larger of [`effective_rows`](Self::effective_rows) and
    /// [`effective_columns`](Self::effective_columns). It can exceed both
    /// [`right_hand_side_rows`](Self::right_hand_side_rows) and
    /// [`solution_rows`](Self::solution_rows): Accelerate requires the physical carrier for every
    /// rectangular factorization to span the larger matrix dimension.
    pub fn in_place_operand_rows(&self) -> usize {
        self.effective_rows().max(self.effective_columns())
    }

    /// Entries a values slice must carry for this pattern.
    pub fn value_count(&self) -> usize {
        self.value_count
    }

    /// Bytes Accelerate reports it will need for a numeric factorization built from this
    /// analysis.
    pub fn factor_size<T: Scalar>(&self) -> usize {
        // SAFETY: the handle is live for the lifetime of `self`.
        unsafe { T::factor_size(self.handle.as_ptr()) }
    }

    /// Numerically factors `values` against this analysis.
    ///
    /// # Errors
    ///
    /// As [`Factorization::new`]. In particular, returns [`InputError::ValuesLength`] when
    /// `values` does not match [`value_count`](Self::value_count).
    pub fn factorize<T: Scalar>(&self, values: &[T]) -> Result<Factorization<T>, Error> {
        Factorization::new(self, values)
    }
}

impl Drop for SymbolicFactorization {
    fn drop(&mut self) {
        // SAFETY: the handle came from `accsp_symbolic_new` and is released exactly once.
        // Factorizations built from it stay valid: Accelerate retains the analysis inside each,
        // and the shim gave each its own copy of the pattern.
        unsafe { sys::accsp_symbolic_free(self.handle.as_ptr()) }
    }
}

impl core::fmt::Debug for SymbolicFactorization {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SymbolicFactorization")
            .field("kind", &self.kind)
            .field("rows", &self.rows)
            .field("columns", &self.columns)
            .field("block_size", &self.block_size)
            .field("transpose", &self.transpose)
            .field("value_count", &self.value_count)
            .finish_non_exhaustive()
    }
}

/// Reports counts of a symmetric matrix's eigenvalues by sign.
///
/// **Observed:** The scalar counts sum to the scalar matrix dimension for any block size. They count
/// pivots, whose signs match the eigenvalues by Sylvester's law of inertia except near zero; see
/// [`Factorization::inertia`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Inertia {
    /// Pivots taken as positive, within the factorization's tolerances.
    pub positive: usize,
    /// Pivots taken as zero, within the factorization's tolerances.
    pub zero: usize,
    /// Pivots taken as negative, within the factorization's tolerances.
    pub negative: usize,
}

/// Stores a numeric factorization ready to solve.
///
/// Independent of the [`SymbolicFactorization`] it came from: Accelerate retains the analysis
/// internally and the shim keeps its own copy of the pattern, so this can outlive its analysis.
///
/// # Thread safety
///
/// [`Sync`] but not [`Send`]. Shared references can solve concurrently because Accelerate allocates
/// workspace per call, and subfactor reference-count changes are serialized. The owning value stays
/// on one thread because the framework's reference count is not documented as atomic.
///
/// A `compile_fail` test pins the absence of `Send`:
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<accelerate_sparse::Factorization<f64>>();
/// ```
pub struct Factorization<T: Scalar> {
    handle: NonNull<c_void>,
    kind: FactorizationKind,
    rows: i32,
    columns: i32,
    block_size: u8,
    transpose: bool,
    value_count: usize,
    factored: bool,
    _scalar: PhantomData<T>,
    // Withholds `Send`. `Sync` is restored below.
    _not_send: PhantomData<*const ()>,
}

// SAFETY: every entry point a shared reference can reach takes the factorization by value, so the
// worker each dispatches to addresses a copy. For `SparseSolve` the workspace is allocated and
// freed within the call from sizes held in read-only fields; `SparseGetInertia` writes only through
// out-parameters the caller owns. Neither mutates the factorization. `subfactor` is the exception:
// extracting a piece retains this factorization, which is shared mutation, so it takes
// `REFCOUNT_LOCK` — as does the matching release. Any further method that mutates shared state
// through `&self` must do the same.
unsafe impl<T: Scalar> Sync for Factorization<T> {}

impl<T: Scalar> Factorization<T> {
    /// Numerically factors `values` against `symbolic`.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::ValuesLength`] if `values` does not match the analysis's value count.
    /// Otherwise returns [`Status::FactorizationFailed`] when the matrix cannot be factored by the
    /// chosen method — for Cholesky, when it is not positive definite.
    pub fn new(symbolic: &SymbolicFactorization, values: &[T]) -> Result<Self, Error> {
        Self::with_options(symbolic, values, NumericOptions::new())
    }

    /// Numerically factors `values` with explicit options.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new).
    pub fn with_options(
        symbolic: &SymbolicFactorization,
        values: &[T],
        options: NumericOptions,
    ) -> Result<Self, Error> {
        if values.len() != symbolic.value_count() {
            return Err(InputError::ValuesLength {
                expected: symbolic.value_count(),
                actual: values.len(),
            }
            .into());
        }

        diagnostics::install();
        diagnostics::clear();

        let raw_options = options.to_raw::<T>();
        let mut status = 0;
        // SAFETY: the analysis handle is live for the duration of the borrow, `values` has the
        // length its pattern requires as checked above, and it outlives the call.
        let handle = unsafe {
            T::numeric_new(
                symbolic.handle.as_ptr(),
                values.as_ptr(),
                raw_options
                    .as_ref()
                    .map_or(core::ptr::null(), |options| options),
                &mut status,
            )
        };

        finish(status)?;
        let handle = NonNull::new(handle)
            .ok_or_else(|| Error::with_detail(Status::InternalError, diagnostics::take()))?;

        Ok(Self {
            handle,
            kind: symbolic.kind,
            rows: symbolic.rows,
            columns: symbolic.columns,
            block_size: symbolic.block_size(),
            transpose: symbolic.transpose,
            value_count: symbolic.value_count(),
            factored: true,
            _scalar: PhantomData,
            _not_send: PhantomData,
        })
    }

    /// Re-forms this factorization from new values for the same pattern.
    ///
    /// Reuses the analysis and allocated factor memory.
    ///
    /// On a numeric failure the factorization is left *unfactored* rather than invalid. Solving
    /// returns [`Status::NotFactored`] until a later call succeeds; retrying with different
    /// values is the intended recovery when a matrix cannot be factored by the chosen method. A
    /// local input error leaves the existing factorization unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::ValuesLength`] if `values` does not match
    /// [`value_count`](Self::value_count). Otherwise as [`new`](Self::new).
    pub fn refactor(&mut self, values: &[T]) -> Result<(), Error> {
        self.refactor_with_options(values, NumericOptions::new())
    }

    /// Re-forms this factorization with explicit options.
    ///
    /// # Errors
    ///
    /// As [`refactor`](Self::refactor).
    pub fn refactor_with_options(
        &mut self,
        values: &[T],
        options: NumericOptions,
    ) -> Result<(), Error> {
        if values.len() != self.value_count {
            return Err(InputError::ValuesLength {
                expected: self.value_count,
                actual: values.len(),
            }
            .into());
        }

        diagnostics::install();
        diagnostics::clear();

        let raw_options = options.to_raw::<T>();
        // SAFETY: the handle is live and exclusively borrowed here, and `values` has the length
        // the pattern requires as checked above.
        let status = unsafe {
            T::numeric_refactor(
                self.handle.as_ptr(),
                values.as_ptr(),
                raw_options
                    .as_ref()
                    .map_or(core::ptr::null(), |options| options),
            )
        };

        let result = finish(status);
        self.factored = result.is_ok();
        result
    }

    /// Whether this currently holds a completed factorization.
    pub fn is_factored(&self) -> bool {
        self.factored
    }

    /// The factorization this was built as.
    pub fn kind(&self) -> FactorizationKind {
        self.kind
    }

    /// Rows in the stored pattern, in blocks if the block size is above one.
    ///
    /// This does not apply the pattern's transpose attribute. See
    /// [`effective_rows`](Self::effective_rows) for the matrix's scalar row count.
    pub fn rows(&self) -> usize {
        self.rows as usize
    }

    /// Columns in the stored pattern, in blocks if the block size is above one.
    ///
    /// This does not apply the pattern's transpose attribute. See
    /// [`effective_columns`](Self::effective_columns) for the matrix's scalar column count.
    pub fn columns(&self) -> usize {
        self.columns as usize
    }

    /// Entries per block edge, as the factored structure declared it.
    pub fn block_size(&self) -> u8 {
        self.block_size
    }

    /// Rows in the effective matrix, after applying its transpose attribute, in scalars.
    pub fn effective_rows(&self) -> usize {
        effective_scalar_dimensions(self.rows, self.columns, self.block_size, self.transpose).0
    }

    /// Columns in the effective matrix, after applying its transpose attribute, in scalars.
    pub fn effective_columns(&self) -> usize {
        effective_scalar_dimensions(self.rows, self.columns, self.block_size, self.transpose).1
    }

    /// Scalar rows a right-hand side must have.
    ///
    /// This is [`effective_rows`](Self::effective_rows) for [`FactorizationKind::Qr`] and
    /// [`effective_columns`](Self::effective_columns) for every other kind. For
    /// [`FactorizationKind::CholeskyAtA`] the operand this counts is the reduced right-hand side
    /// `Aᵀb`; the count cannot enforce that when the matrix is square — see that kind.
    pub fn right_hand_side_rows(&self) -> usize {
        required_right_hand_side_rows(self.kind, self.effective_rows(), self.effective_columns())
    }

    /// Scalar rows a solution will have.
    ///
    /// This is always [`effective_columns`](Self::effective_columns).
    pub fn solution_rows(&self) -> usize {
        self.effective_columns()
    }

    /// Scalar rows an in-place solve operand must have.
    ///
    /// This is the larger of [`effective_rows`](Self::effective_rows) and
    /// [`effective_columns`](Self::effective_columns). It can exceed both
    /// [`right_hand_side_rows`](Self::right_hand_side_rows) and
    /// [`solution_rows`](Self::solution_rows): Accelerate requires the physical carrier for every
    /// rectangular factorization to span the larger matrix dimension.
    pub fn in_place_operand_rows(&self) -> usize {
        self.effective_rows().max(self.effective_columns())
    }

    /// Entries a values slice must carry.
    pub fn value_count(&self) -> usize {
        self.value_count
    }

    /// Solves `A x = b` for a single right-hand side, returning the solution.
    ///
    /// For multiple right-hand sides, or to write into storage the caller already holds, use
    /// [`solve_into`](Self::solve_into) or [`solve_in_place`](Self::solve_in_place).
    ///
    /// # Errors
    ///
    /// Returns [`InputError::DenseZeroDimension`](crate::error::InputError::DenseZeroDimension) when `b`
    /// is empty, or [`InputError::OperandRows`] when its length differs from
    /// [`right_hand_side_rows`](Self::right_hand_side_rows). Otherwise returns
    /// [`Status::NotFactored`] if the last factorization attempt failed.
    pub fn solve_vec(&self, b: &[T]) -> Result<Vec<T>, Error>
    where
        T: Default,
    {
        let mut x = vec![T::default(); self.solution_rows()];
        self.solve_into(DenseRef::from_vector(b)?, DenseMut::from_vector(&mut x)?)?;
        Ok(x)
    }

    /// Solves `A x = b`, writing the solution into `x`.
    ///
    /// Supports multiple right-hand sides and caller-owned output storage. Column strides permit
    /// views into larger column-major matrices without a copy.
    ///
    /// Takes `&self`, so one factorization can drive concurrent solves for different right-hand
    /// sides.
    ///
    /// The operands require [`right_hand_side_rows`](Self::right_hand_side_rows) and
    /// [`solution_rows`](Self::solution_rows), respectively. The two sizes coincide for every
    /// square kind. Operands are always counted in scalars, never in blocks.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::OperandRows`] for an operand with the wrong row count, or
    /// [`InputError::OperandColumns`] when the operands carry different numbers of columns.
    /// Otherwise as [`solve_vec`](Self::solve_vec).
    pub fn solve_into(&self, b: DenseRef<'_, T>, mut x: DenseMut<'_, T>) -> Result<(), Error> {
        self.check_operand(
            b.rows(),
            self.right_hand_side_rows(),
            OperandRole::RightHandSide,
        )?;
        self.check_operand(x.rows(), self.solution_rows(), OperandRole::Solution)?;
        if b.columns() != x.columns() {
            return Err(InputError::OperandColumns {
                first: OperandRole::RightHandSide,
                first_columns: b.columns(),
                second: OperandRole::Solution,
                second_columns: x.columns(),
            }
            .into());
        }

        diagnostics::clear();
        let raw_b = b.raw();
        let raw_x = x.raw_mut();
        // SAFETY: the handle is live; both views validated their shape against their storage at
        // construction, and `x` is exclusively borrowed for the call.
        let status = unsafe { T::solve(self.handle.as_ptr(), &raw_b, &raw_x) };
        finish(status)
    }

    /// Solves with the right-hand side overwritten by the solution, in a single buffer.
    ///
    /// The physical buffer has [`in_place_operand_rows`](Self::in_place_operand_rows) scalar rows.
    /// Its first [`right_hand_side_rows`](Self::right_hand_side_rows) rows carry the input, and its
    /// first [`solution_rows`](Self::solution_rows) rows carry the output. Each column of a
    /// multiple-right-hand-side operand follows the same layout.
    ///
    /// Those three sizes can differ. In particular, a tall
    /// [`FactorizationKind::CholeskyAtA`] has an `n`-row reduced right-hand side and an `n`-row
    /// solution in an `m`-row carrier. The trailing `m - n` rows carry neither logical vector, but
    /// must exist: giving Accelerate only `n` rows is observed to report success without writing a
    /// solution.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::OperandRows`] if the operand does not have
    /// [`in_place_operand_rows`](Self::in_place_operand_rows) scalar rows. Otherwise as
    /// [`solve_vec`](Self::solve_vec).
    pub fn solve_in_place(&self, mut xb: DenseMut<'_, T>) -> Result<(), Error> {
        self.check_operand(
            xb.rows(),
            self.in_place_operand_rows(),
            OperandRole::InPlace,
        )?;

        diagnostics::clear();
        let raw = xb.raw_mut();
        // SAFETY: as in `solve_into`, with the single view both read and written.
        let status = unsafe { T::solve_in_place(self.handle.as_ptr(), &raw) };
        finish(status)
    }

    /// Counts the factored matrix's eigenvalues by sign.
    ///
    /// The counts come from the factorization's pivots, whose signs match the eigenvalues' by
    /// Sylvester's law of inertia; no eigenvalue computation is involved.
    ///
    /// Accepted only for [`FactorizationKind::LdltTpp`], a framework restriction.
    ///
    /// Near zero, [`NumericOptions::zero_tolerance`] can make pivot counts differ from the true
    /// eigenvalue signs. For a near-singular matrix, the counts describe the factorization.
    ///
    /// [`NumericOptions::zero_tolerance`]: crate::options::NumericOptions::zero_tolerance
    ///
    /// # Errors
    ///
    /// Returns [`Status::NotFactored`] if the last factorization attempt failed. Otherwise returns
    /// [`Status::ParameterError`] for any kind other than [`FactorizationKind::LdltTpp`], carrying
    /// Accelerate's own explanation in
    /// [`Error::detail`] (*observed*, since Apple documents no message for it), or
    /// [`Status::UnsupportedOs`] below macOS 13.0 or when the building SDK did not provide the
    /// underlying function.
    ///
    /// [`Error::detail`]: crate::error::Error::detail
    pub fn inertia(&self) -> Result<Inertia, Error> {
        diagnostics::clear();

        let (mut positive, mut zero, mut negative) = (0, 0, 0);
        // SAFETY: the handle is live for as long as `self` is, and the three out-parameters are
        // writable locals that outlive the call.
        let status = unsafe {
            T::get_inertia(
                self.handle.as_ptr(),
                &mut positive,
                &mut zero,
                &mut negative,
            )
        };
        finish(status)?;

        // A negative count would be Accelerate misreporting a success; treat it like the absent
        // handle on a reported success rather than wrapping it into a huge count.
        let count = |value: c_int| {
            usize::try_from(value)
                .map_err(|_| Error::with_detail(Status::InternalError, diagnostics::take()))
        };
        Ok(Inertia {
            positive: count(positive)?,
            zero: count(zero)?,
            negative: count(negative)?,
        })
    }

    /// Extracts one piece of this factorization.
    ///
    /// Returns an operator handle, not a matrix that can be read entry by entry.
    ///
    /// The returned [`Subfactor`] borrows this factorization, so [`refactor`](Self::refactor) cannot
    /// run while it is alive.
    ///
    /// # Errors
    ///
    /// Returns [`Status::NotFactored`] if the last factorization attempt failed, and
    /// [`InputError::SubfactorUnavailable`] if this kind cannot supply the requested piece — see
    /// [`SubfactorKind::applies_to`], which is checked here before Accelerate is reached.
    ///
    /// The [`Status::NotFactored`] check prevents a process abort: asking Accelerate for a piece
    /// of a failed factorization traps, and the error callback does not intercept it — *observed*,
    /// with and without a handler installed.
    pub fn subfactor(&self, kind: SubfactorKind) -> Result<Subfactor<'_, T>, Error> {
        if !kind.applies_to(self.kind) {
            return Err(InputError::SubfactorUnavailable {
                subfactor: kind,
                factorization: self.kind,
            }
            .into());
        }

        // Accelerate sizes a subfactor from the stored matrix dimensions. The current rule is
        // symmetric in them, so using the stored pair preserves the input orientation without
        // changing the result.
        let block = usize::from(self.block_size);
        let (rows, columns) = kind.scalar_shape(
            self.kind,
            self.rows as usize * block,
            self.columns as usize * block,
        );

        diagnostics::clear();
        let mut status = 0;
        // SAFETY: the handle is live for as long as `self` is, the selector is one of the shim's
        // own constants, and `status` is a writable local. The lock covers the retain this performs
        // on the factorization.
        let handle = with_refcount_lock(|| unsafe {
            T::subfactor_new(self.handle.as_ptr(), kind.to_raw(), &mut status)
        });
        finish(status)?;
        let handle = NonNull::new(handle)
            .ok_or_else(|| Error::with_detail(Status::InternalError, diagnostics::take()))?;

        Ok(Subfactor {
            handle,
            kind,
            rows,
            columns,
            transposed: false,
            _parent: PhantomData,
            _scalar: PhantomData,
            _not_send: PhantomData,
        })
    }

    /// Checks a dense operand's row count against what the factorization expects.
    ///
    /// On a shape mismatch Accelerate reports through the error callback and returns with the
    /// solution untouched, so an unchecked wrong shape would read as a successful solve of
    /// whatever the buffer already held. The callback message is best-effort — Accelerate may
    /// record it on a worker thread whose thread-local this layer never drains — so the shape is
    /// checked here instead.
    fn check_operand(
        &self,
        rows: usize,
        expected: usize,
        operand: OperandRole,
    ) -> Result<(), InputError> {
        if rows != expected {
            return Err(InputError::OperandRows {
                operand,
                expected,
                actual: rows,
            });
        }
        Ok(())
    }
}

impl<T: Scalar> Drop for Factorization<T> {
    fn drop(&mut self) {
        // SAFETY: the handle came from the numeric constructor for this element type and is
        // released exactly once.
        unsafe { T::numeric_free(self.handle.as_ptr()) }
    }
}

impl<T: Scalar> core::fmt::Debug for Factorization<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Factorization")
            .field("kind", &self.kind)
            .field("rows", &self.rows)
            .field("columns", &self.columns)
            .field("block_size", &self.block_size)
            .field("transpose", &self.transpose)
            .field("value_count", &self.value_count)
            .field("factored", &self.factored)
            .finish_non_exhaustive()
    }
}

/// Represents one piece of a [`Factorization`]: its `L`, its `Q`, a permutation, or a scaling.
///
/// A subfactor is an operator that can be applied but not read entry by entry. Obtain one from
/// [`Factorization::subfactor`].
///
/// A subfactor applies its parent's live state. Borrowing the parent prevents refactoring while the
/// subfactor exists:
///
/// ```compile_fail
/// # use accelerate_sparse::*;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let starts = [0i64, 2, 4, 5];
/// # let rows = [0i32, 1, 1, 2, 2];
/// # let structure =
/// #     SparseStructure::from_csc(3, 3, &starts, &rows, Attributes::symmetric(Triangle::Lower))?;
/// # let symbolic = SymbolicFactorization::new(FactorizationKind::Cholesky, &structure)?;
/// let mut factorization = symbolic.factorize(&[4.0, 1.0, 3.0, 1.0, 2.0])?;
/// let l = factorization.subfactor(SubfactorKind::L)?;
/// factorization.refactor(&[8.0, 2.0, 6.0, 2.0, 4.0])?;
/// let _ = l.multiply_vec(&[1.0, 0.0, 0.0])?;
/// # Ok(())
/// # }
/// ```
///
/// Not [`Send`], as with the factorization itself:
///
/// ```compile_fail
/// fn assert_send<T: Send>() {}
/// assert_send::<accelerate_sparse::Subfactor<'static, f64>>();
/// ```
///
/// Subfactor [`rows`](Self::rows) and [`columns`](Self::columns) count scalars and equal the required
/// dense-operand lengths. [`Factorization::rows`] and [`Factorization::columns`] count blocks when
/// the block size is above one.
///
/// **Observed:** Every piece is square with the smaller parent dimension, except QR's `Q`, which is
/// larger by smaller. A parent's [`Attributes`](crate::Attributes) transpose does not change these
/// shapes.
///
/// # Thread safety
///
/// As [`Factorization`]: [`Sync`] but not [`Send`].
pub struct Subfactor<'a, T: Scalar> {
    handle: NonNull<c_void>,
    kind: SubfactorKind,
    rows: usize,
    columns: usize,
    transposed: bool,
    // Borrows the factorization, so it cannot be refactored or dropped while this is alive.
    _parent: PhantomData<&'a Factorization<T>>,
    _scalar: PhantomData<T>,
    // Withholds `Send`. `Sync` is restored below.
    _not_send: PhantomData<*const ()>,
}

// SAFETY: Accelerate's subfactor entry points take the object by value, so each dispatches on a
// copy, and the scratch space they need is allocated and freed within the call. Two of the methods
// reachable from a shared reference do mutate shared state — `transpose` retains, and `Drop`
// releases — so both take `REFCOUNT_LOCK`; the rest touch nothing shared.
unsafe impl<T: Scalar> Sync for Subfactor<'_, T> {}

impl<T: Scalar> Subfactor<'_, T> {
    /// Which piece this is.
    pub fn kind(&self) -> SubfactorKind {
        self.kind
    }

    /// Rows, in scalars, with any transposition applied.
    pub fn rows(&self) -> usize {
        if self.transposed {
            self.columns
        } else {
            self.rows
        }
    }

    /// Columns, in scalars, with any transposition applied.
    pub fn columns(&self) -> usize {
        if self.transposed {
            self.rows
        } else {
            self.columns
        }
    }

    /// Whether this is applied transposed.
    pub fn is_transposed(&self) -> bool {
        self.transposed
    }

    /// The transpose, as a separate handle borrowing the same factorization.
    ///
    /// The original remains usable alongside it. For a Cholesky factorization, `P L Lᵀ Pᵀ` applied
    /// to a vector reproduces `A` times that vector, which needs both a piece and its transpose at
    /// once.
    ///
    /// # Errors
    ///
    /// Returns whatever [`Status`] Accelerate reports. No failure of this call has been observed;
    /// the result is fallible because the handle it produces is allocated like any other.
    pub fn transpose(&self) -> Result<Self, Error> {
        diagnostics::clear();
        let mut status = 0;
        // SAFETY: the handle is live for the lifetime of `self`, and `status` is a writable local.
        // The lock covers the retain this performs.
        let handle = with_refcount_lock(|| unsafe {
            T::subfactor_transpose(self.handle.as_ptr(), &mut status)
        });
        finish(status)?;
        let handle = NonNull::new(handle)
            .ok_or_else(|| Error::with_detail(Status::InternalError, diagnostics::take()))?;

        Ok(Self {
            handle,
            kind: self.kind,
            rows: self.rows,
            columns: self.columns,
            transposed: !self.transposed,
            _parent: PhantomData,
            _scalar: PhantomData,
            _not_send: PhantomData,
        })
    }

    /// Scratch bytes an operation on this subfactor allocates, for `columns` right-hand sides.
    ///
    /// Operations allocate this on every call, and the amount varies by piece.
    ///
    /// Saturates at [`usize::MAX`] on arithmetic overflow.
    pub fn workspace_required(&self, columns: usize) -> usize {
        let (mut static_bytes, mut per_rhs) = (0, 0);
        // SAFETY: the handle is live and both out-parameters are writable locals.
        unsafe {
            T::subfactor_workspace(self.handle.as_ptr(), &mut static_bytes, &mut per_rhs);
        }
        static_bytes.saturating_add(columns.saturating_mul(per_rhs))
    }

    /// Solves `subfactor · x = b`, writing the solution into `x`.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::OperandRows`] for an operand with the wrong row count, or
    /// [`InputError::OperandColumns`] when the operands carry different numbers of columns.
    /// Otherwise returns the [`Status`] Accelerate reported.
    pub fn solve_into(&self, b: DenseRef<'_, T>, mut x: DenseMut<'_, T>) -> Result<(), Error> {
        self.check_operand(b.rows(), self.rows(), OperandRole::RightHandSide)?;
        self.check_operand(x.rows(), self.columns(), OperandRole::Solution)?;
        if b.columns() != x.columns() {
            return Err(InputError::OperandColumns {
                first: OperandRole::RightHandSide,
                first_columns: b.columns(),
                second: OperandRole::Solution,
                second_columns: x.columns(),
            }
            .into());
        }

        diagnostics::clear();
        let raw_b = b.raw();
        let raw_x = x.raw_mut();
        // SAFETY: the handle is live, both views validated their shape against their storage at
        // construction, their row counts were just checked against this subfactor, and `x` is
        // exclusively borrowed for the call.
        let status = unsafe { T::subfactor_solve(self.handle.as_ptr(), &raw_b, &raw_x) };
        finish_application(status)
    }

    /// Solves with the right-hand side overwritten by the solution.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::OperandRows`] unless the operand has `max(rows, columns)` scalar
    /// rows. Otherwise as [`solve_into`](Self::solve_into).
    pub fn solve_in_place(&self, mut xb: DenseMut<'_, T>) -> Result<(), Error> {
        let expected = self.rows().max(self.columns());
        self.check_operand(xb.rows(), expected, OperandRole::InPlace)?;

        diagnostics::clear();
        let raw = xb.raw_mut();
        // SAFETY: as in `solve_into`, with the single view both read and written.
        let status = unsafe { T::subfactor_solve_in_place(self.handle.as_ptr(), &raw) };
        finish_application(status)
    }

    /// Multiplies `y = subfactor · x`.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::MultiplyUnsupported`] for a piece that cannot be multiplied — see
    /// [`SubfactorKind::supports_multiply`]. Returns [`InputError::OperandRows`] for an operand
    /// with the wrong row count, or [`InputError::OperandColumns`] when the operands carry
    /// different numbers of columns. Otherwise returns the [`Status`] Accelerate reported.
    pub fn multiply_into(&self, x: DenseRef<'_, T>, mut y: DenseMut<'_, T>) -> Result<(), Error> {
        if !self.kind.supports_multiply() {
            return Err(InputError::MultiplyUnsupported {
                subfactor: self.kind,
            }
            .into());
        }
        self.check_operand(x.rows(), self.columns(), OperandRole::Multiplicand)?;
        self.check_operand(y.rows(), self.rows(), OperandRole::Product)?;
        if x.columns() != y.columns() {
            return Err(InputError::OperandColumns {
                first: OperandRole::Multiplicand,
                first_columns: x.columns(),
                second: OperandRole::Product,
                second_columns: y.columns(),
            }
            .into());
        }

        diagnostics::clear();
        let raw_x = x.raw();
        let raw_y = y.raw_mut();
        // SAFETY: as in `solve_into`, with the roles of the two operands exchanged.
        let status = unsafe { T::subfactor_multiply(self.handle.as_ptr(), &raw_x, &raw_y) };
        finish_application(status)
    }

    /// Multiplies with the operand overwritten by the product.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::MultiplyUnsupported`] for a piece that cannot be multiplied, or
    /// [`InputError::OperandRows`] unless the operand has `max(rows, columns)` scalar rows.
    /// Otherwise as [`multiply_into`](Self::multiply_into).
    pub fn multiply_in_place(&self, mut xy: DenseMut<'_, T>) -> Result<(), Error> {
        if !self.kind.supports_multiply() {
            return Err(InputError::MultiplyUnsupported {
                subfactor: self.kind,
            }
            .into());
        }
        let expected = self.rows().max(self.columns());
        self.check_operand(xy.rows(), expected, OperandRole::InPlace)?;

        diagnostics::clear();
        let raw = xy.raw_mut();
        // SAFETY: as in `multiply_into`, with the single view both read and written.
        let status = unsafe { T::subfactor_multiply_in_place(self.handle.as_ptr(), &raw) };
        finish_application(status)
    }

    /// Applies this subfactor to a single vector, returning the result.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::DenseZeroDimension`](crate::error::InputError::DenseZeroDimension) when `x`
    /// is empty. Otherwise as [`multiply_into`](Self::multiply_into).
    pub fn multiply_vec(&self, x: &[T]) -> Result<Vec<T>, Error>
    where
        T: Default,
    {
        let mut y = vec![T::default(); self.rows()];
        self.multiply_into(DenseRef::from_vector(x)?, DenseMut::from_vector(&mut y)?)?;
        Ok(y)
    }

    /// Solves against this subfactor for a single right-hand side, returning the solution.
    ///
    /// # Errors
    ///
    /// Returns [`InputError::DenseZeroDimension`](crate::error::InputError::DenseZeroDimension) when `b`
    /// is empty. Otherwise as [`solve_into`](Self::solve_into).
    pub fn solve_vec(&self, b: &[T]) -> Result<Vec<T>, Error>
    where
        T: Default,
    {
        let mut x = vec![T::default(); self.columns()];
        self.solve_into(DenseRef::from_vector(b)?, DenseMut::from_vector(&mut x)?)?;
        Ok(x)
    }

    /// Checks a dense operand's row count.
    ///
    /// Accelerate reports a mismatch through the error callback and leaves the operand untouched,
    /// so an unchecked wrong shape would read as a successful application that returned whatever
    /// the buffer already held — and the callback is best-effort, since Accelerate may record it
    /// on a thread whose thread-local this layer never drains.
    fn check_operand(
        &self,
        rows: usize,
        expected: usize,
        operand: OperandRole,
    ) -> Result<(), InputError> {
        if rows != expected {
            return Err(InputError::OperandRows {
                operand,
                expected,
                actual: rows,
            });
        }
        Ok(())
    }
}

impl<T: Scalar> Drop for Subfactor<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the handle came from `subfactor_new` or `subfactor_transpose` for this element
        // type and is released exactly once. Releasing it decrements the factorization's reference
        // count, which this still borrows, so the lock covers it as the retains are covered.
        with_refcount_lock(|| unsafe { T::subfactor_free(self.handle.as_ptr()) })
    }
}

impl<T: Scalar> core::fmt::Debug for Subfactor<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Subfactor")
            .field("kind", &self.kind)
            .field("rows", &self.rows())
            .field("columns", &self.columns())
            .field("transposed", &self.transposed)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A subfactor application reports success through the shim whatever Accelerate thought of it,
    /// so the recorded message is what distinguishes a rejected operand from a real result. Nothing
    /// reachable through the public API can produce that message — the operand check refuses every
    /// shape Accelerate would reject — so the branch is exercised here instead. Without it, dropping
    /// the check and returning `finish(status)` passes the whole suite.
    #[test]
    fn a_recorded_message_turns_a_reported_success_into_an_error() {
        diagnostics::clear();
        assert!(
            finish_application(sys::ACCSP_STATUS_OK).is_ok(),
            "an application with nothing recorded is a success"
        );

        diagnostics::plant(
            "X (size 2x1) does not match dimensions of subfactor dimension (3 x 3).",
        );
        let error = finish_application(sys::ACCSP_STATUS_OK)
            .expect_err("a recorded message means Accelerate refused the operands");
        assert_eq!(error.status(), Some(Status::ParameterError));
        assert_eq!(error.input(), None);
        assert!(
            error.detail().is_some_and(|d| d.contains("does not match")),
            "the message Accelerate recorded should reach the caller"
        );

        // Draining leaves the slot clean for the next call.
        assert!(finish_application(sys::ACCSP_STATUS_OK).is_ok());
    }

    #[test]
    fn local_input_errors_are_not_framework_parameter_errors() {
        let local: Error = InputError::OperandRows {
            operand: OperandRole::RightHandSide,
            expected: 3,
            actual: 2,
        }
        .into();
        assert_eq!(local.status(), None);
        assert!(matches!(
            local.input(),
            Some(InputError::OperandRows {
                operand: OperandRole::RightHandSide,
                expected: 3,
                actual: 2,
            })
        ));

        diagnostics::clear();
        diagnostics::plant("Accelerate rejected the operand");
        let framework = finish_application(sys::ACCSP_STATUS_OK).unwrap_err();
        assert_eq!(framework.status(), Some(Status::ParameterError));
        assert_eq!(framework.input(), None);
    }

    /// A genuine failure keeps its own status rather than being relabelled a parameter error.
    #[test]
    fn a_failing_status_is_not_masked_by_the_message_check() {
        diagnostics::clear();
        diagnostics::plant("something Accelerate said");
        let error = finish_application(sys::ACCSP_STATUS_NOT_FACTORED).unwrap_err();
        assert_eq!(error.status(), Some(Status::NotFactored));
    }
}
