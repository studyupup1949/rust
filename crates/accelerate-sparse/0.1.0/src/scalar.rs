//! The [`Scalar`] trait identifies supported factorization element types.

use accelerate_sparse_sys as sys;
use core::ffi::{c_int, c_void};

mod private {
    pub trait Sealed {}
    impl Sealed for f64 {}
    impl Sealed for f32 {}
}

/// Identifies an element type Accelerate can factor.
///
/// Implemented for [`f64`] and [`f32`]. Symbolic analysis is independent of the element type, which
/// is chosen when values are first factored.
///
/// Sealed to Accelerate's element types. Its members dispatch to type-specific entry points and are
/// not part of the stable surface.
pub trait Scalar: private::Sealed + Copy + PartialEq + core::fmt::Debug + 'static {
    /// The dense-operand descriptor this element type is passed through.
    #[doc(hidden)]
    type Dense: Copy;

    /// Builds a dense-operand descriptor.
    #[doc(hidden)]
    fn dense(rows: c_int, columns: c_int, column_stride: c_int, data: *mut Self) -> Self::Dense;

    /// Fills `out` with the default numeric-phase options for this element type.
    ///
    /// # Safety
    ///
    /// `out` must be writable.
    #[doc(hidden)]
    unsafe fn default_numeric_options(out: *mut sys::accsp_numeric_options);

    /// Bytes a numeric factorization of this element type will occupy.
    ///
    /// # Safety
    ///
    /// `symbolic` must be a live symbolic handle.
    #[doc(hidden)]
    unsafe fn factor_size(symbolic: *const sys::accsp_symbolic_t) -> usize;

    /// Builds a numeric factorization, returning an erased handle.
    ///
    /// # Safety
    ///
    /// `symbolic` must be live, `values` must have one entry per stored non-zero of its pattern,
    /// and `out_status` must be writable.
    #[doc(hidden)]
    unsafe fn numeric_new(
        symbolic: *const sys::accsp_symbolic_t,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
        out_status: *mut c_int,
    ) -> *mut c_void;

    /// Re-forms a numeric factorization from new values.
    ///
    /// # Safety
    ///
    /// `handle` must be a live handle of this element type, and `values` must have one entry per
    /// stored non-zero of its pattern.
    #[doc(hidden)]
    unsafe fn numeric_refactor(
        handle: *mut c_void,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
    ) -> c_int;

    /// Releases a numeric factorization.
    ///
    /// # Safety
    ///
    /// `handle` must be a live handle of this element type, not yet released.
    #[doc(hidden)]
    unsafe fn numeric_free(handle: *mut c_void);

    /// Solves into a separate solution operand.
    ///
    /// # Safety
    ///
    /// `handle` must be live and both operands must describe storage they own.
    #[doc(hidden)]
    unsafe fn solve(handle: *const c_void, b: *const Self::Dense, x: *const Self::Dense) -> c_int;

    /// Solves with the right-hand side overwritten.
    ///
    /// # Safety
    ///
    /// `handle` must be live and the operand must describe storage it owns.
    #[doc(hidden)]
    unsafe fn solve_in_place(handle: *const c_void, xb: *const Self::Dense) -> c_int;

    /// Counts the factorization's pivots by sign.
    ///
    /// # Safety
    ///
    /// `handle` must be a live handle of this element type, and each out-parameter must be
    /// writable and suitably aligned.
    #[doc(hidden)]
    unsafe fn get_inertia(
        handle: *const c_void,
        positive: *mut c_int,
        zero: *mut c_int,
        negative: *mut c_int,
    ) -> c_int;

    /// Extracts a subfactor, returning an erased handle.
    ///
    /// # Safety
    ///
    /// `handle` must be a live factorization handle of this element type and `out_status` must be
    /// writable. A non-null result must be released with [`subfactor_free`](Self::subfactor_free).
    #[doc(hidden)]
    unsafe fn subfactor_new(
        handle: *const c_void,
        selector: u8,
        out_status: *mut c_int,
    ) -> *mut c_void;

    /// Takes the transpose of a subfactor as a separate handle.
    ///
    /// # Safety
    ///
    /// As [`subfactor_new`](Self::subfactor_new), with a live subfactor handle.
    #[doc(hidden)]
    unsafe fn subfactor_transpose(subfactor: *const c_void, out_status: *mut c_int) -> *mut c_void;

    /// Releases a subfactor handle.
    ///
    /// # Safety
    ///
    /// `subfactor` must be a live subfactor handle of this element type, not yet released.
    #[doc(hidden)]
    unsafe fn subfactor_free(subfactor: *mut c_void);

    /// Reports the scratch space a subfactor operation needs.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live and both out-parameters writable.
    #[doc(hidden)]
    unsafe fn subfactor_workspace(
        subfactor: *const c_void,
        static_bytes: *mut usize,
        per_rhs_bytes: *mut usize,
    );

    /// Solves against a subfactor.
    ///
    /// # Safety
    ///
    /// `subfactor` must be live and both operands must describe storage they own, shaped as the
    /// subfactor requires.
    #[doc(hidden)]
    unsafe fn subfactor_solve(
        subfactor: *const c_void,
        b: *const Self::Dense,
        x: *const Self::Dense,
    ) -> c_int;

    /// Solves against a subfactor with the operand overwritten.
    ///
    /// # Safety
    ///
    /// As [`subfactor_solve`](Self::subfactor_solve), with one operand read and written.
    #[doc(hidden)]
    unsafe fn subfactor_solve_in_place(subfactor: *const c_void, xb: *const Self::Dense) -> c_int;

    /// Multiplies by a subfactor.
    ///
    /// # Safety
    ///
    /// As [`subfactor_solve`](Self::subfactor_solve).
    #[doc(hidden)]
    unsafe fn subfactor_multiply(
        subfactor: *const c_void,
        x: *const Self::Dense,
        y: *const Self::Dense,
    ) -> c_int;

    /// Multiplies by a subfactor with the operand overwritten.
    ///
    /// # Safety
    ///
    /// As [`subfactor_solve`](Self::subfactor_solve), with one operand read and written.
    #[doc(hidden)]
    unsafe fn subfactor_multiply_in_place(
        subfactor: *const c_void,
        xy: *const Self::Dense,
    ) -> c_int;
}

impl Scalar for f64 {
    type Dense = sys::accsp_dense_d;

    fn dense(rows: c_int, columns: c_int, column_stride: c_int, data: *mut Self) -> Self::Dense {
        sys::accsp_dense_d {
            row_count: rows,
            column_count: columns,
            column_stride,
            data,
        }
    }

    unsafe fn default_numeric_options(out: *mut sys::accsp_numeric_options) {
        // SAFETY: the caller guarantees `out` is writable; the shim only writes through it.
        unsafe { sys::accsp_default_numeric_options_d(out) }
    }

    unsafe fn factor_size(symbolic: *const sys::accsp_symbolic_t) -> usize {
        // SAFETY: the caller guarantees `symbolic` is live.
        unsafe { sys::accsp_symbolic_factor_size_d(symbolic) }
    }

    unsafe fn numeric_new(
        symbolic: *const sys::accsp_symbolic_t,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
        out_status: *mut c_int,
    ) -> *mut c_void {
        // SAFETY: forwarded from this function's own contract, which repeats the shim's.
        unsafe { sys::accsp_numeric_new_d(symbolic, values, options, out_status) }.cast()
    }

    unsafe fn numeric_refactor(
        handle: *mut c_void,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
    ) -> c_int {
        // SAFETY: `handle` came from `numeric_new` for this element type, so the cast restores
        // the type it was erased from.
        unsafe { sys::accsp_numeric_refactor_d(handle.cast(), values, options) }
    }

    unsafe fn numeric_free(handle: *mut c_void) {
        // SAFETY: as above.
        unsafe { sys::accsp_numeric_free_d(handle.cast()) }
    }

    unsafe fn solve(handle: *const c_void, b: *const Self::Dense, x: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_solve_d(handle.cast(), b, x) }
    }

    unsafe fn solve_in_place(handle: *const c_void, xb: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_solve_in_place_d(handle.cast(), xb) }
    }

    unsafe fn get_inertia(
        handle: *const c_void,
        positive: *mut c_int,
        zero: *mut c_int,
        negative: *mut c_int,
    ) -> c_int {
        // SAFETY: as above, with three writable out-parameters supplied by the caller.
        unsafe { sys::accsp_get_inertia_d(handle.cast(), positive, zero, negative) }
    }

    unsafe fn subfactor_new(
        handle: *const c_void,
        selector: u8,
        out_status: *mut c_int,
    ) -> *mut c_void {
        // SAFETY: as above; the selector is an opaque value to the shim, which reports an
        // unsupported one rather than acting on it.
        unsafe { sys::accsp_subfactor_new_d(handle.cast(), selector, out_status) }.cast()
    }

    unsafe fn subfactor_transpose(subfactor: *const c_void, out_status: *mut c_int) -> *mut c_void {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_transpose_d(subfactor.cast(), out_status) }.cast()
    }

    unsafe fn subfactor_free(subfactor: *mut c_void) {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_free_d(subfactor.cast()) }
    }

    unsafe fn subfactor_workspace(
        subfactor: *const c_void,
        static_bytes: *mut usize,
        per_rhs_bytes: *mut usize,
    ) {
        // SAFETY: as above, with two writable out-parameters supplied by the caller.
        unsafe { sys::accsp_subfactor_workspace_d(subfactor.cast(), static_bytes, per_rhs_bytes) }
    }

    unsafe fn subfactor_solve(
        subfactor: *const c_void,
        b: *const Self::Dense,
        x: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_solve_d(subfactor.cast(), b, x) }
    }

    unsafe fn subfactor_solve_in_place(subfactor: *const c_void, xb: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_solve_in_place_d(subfactor.cast(), xb) }
    }

    unsafe fn subfactor_multiply(
        subfactor: *const c_void,
        x: *const Self::Dense,
        y: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_multiply_d(subfactor.cast(), x, y) }
    }

    unsafe fn subfactor_multiply_in_place(
        subfactor: *const c_void,
        xy: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_multiply_in_place_d(subfactor.cast(), xy) }
    }
}

impl Scalar for f32 {
    type Dense = sys::accsp_dense_f;

    fn dense(rows: c_int, columns: c_int, column_stride: c_int, data: *mut Self) -> Self::Dense {
        sys::accsp_dense_f {
            row_count: rows,
            column_count: columns,
            column_stride,
            data,
        }
    }

    unsafe fn default_numeric_options(out: *mut sys::accsp_numeric_options) {
        // SAFETY: the caller guarantees `out` is writable; the shim only writes through it.
        unsafe { sys::accsp_default_numeric_options_f(out) }
    }

    unsafe fn factor_size(symbolic: *const sys::accsp_symbolic_t) -> usize {
        // SAFETY: the caller guarantees `symbolic` is live.
        unsafe { sys::accsp_symbolic_factor_size_f(symbolic) }
    }

    unsafe fn numeric_new(
        symbolic: *const sys::accsp_symbolic_t,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
        out_status: *mut c_int,
    ) -> *mut c_void {
        // SAFETY: forwarded from this function's own contract, which repeats the shim's.
        unsafe { sys::accsp_numeric_new_f(symbolic, values, options, out_status) }.cast()
    }

    unsafe fn numeric_refactor(
        handle: *mut c_void,
        values: *const Self,
        options: *const sys::accsp_numeric_options,
    ) -> c_int {
        // SAFETY: `handle` came from `numeric_new` for this element type, so the cast restores
        // the type it was erased from.
        unsafe { sys::accsp_numeric_refactor_f(handle.cast(), values, options) }
    }

    unsafe fn numeric_free(handle: *mut c_void) {
        // SAFETY: as above.
        unsafe { sys::accsp_numeric_free_f(handle.cast()) }
    }

    unsafe fn solve(handle: *const c_void, b: *const Self::Dense, x: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_solve_f(handle.cast(), b, x) }
    }

    unsafe fn solve_in_place(handle: *const c_void, xb: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_solve_in_place_f(handle.cast(), xb) }
    }

    unsafe fn get_inertia(
        handle: *const c_void,
        positive: *mut c_int,
        zero: *mut c_int,
        negative: *mut c_int,
    ) -> c_int {
        // SAFETY: as above, with three writable out-parameters supplied by the caller.
        unsafe { sys::accsp_get_inertia_f(handle.cast(), positive, zero, negative) }
    }

    unsafe fn subfactor_new(
        handle: *const c_void,
        selector: u8,
        out_status: *mut c_int,
    ) -> *mut c_void {
        // SAFETY: as above; the selector is an opaque value to the shim, which reports an
        // unsupported one rather than acting on it.
        unsafe { sys::accsp_subfactor_new_f(handle.cast(), selector, out_status) }.cast()
    }

    unsafe fn subfactor_transpose(subfactor: *const c_void, out_status: *mut c_int) -> *mut c_void {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_transpose_f(subfactor.cast(), out_status) }.cast()
    }

    unsafe fn subfactor_free(subfactor: *mut c_void) {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_free_f(subfactor.cast()) }
    }

    unsafe fn subfactor_workspace(
        subfactor: *const c_void,
        static_bytes: *mut usize,
        per_rhs_bytes: *mut usize,
    ) {
        // SAFETY: as above, with two writable out-parameters supplied by the caller.
        unsafe { sys::accsp_subfactor_workspace_f(subfactor.cast(), static_bytes, per_rhs_bytes) }
    }

    unsafe fn subfactor_solve(
        subfactor: *const c_void,
        b: *const Self::Dense,
        x: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_solve_f(subfactor.cast(), b, x) }
    }

    unsafe fn subfactor_solve_in_place(subfactor: *const c_void, xb: *const Self::Dense) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_solve_in_place_f(subfactor.cast(), xb) }
    }

    unsafe fn subfactor_multiply(
        subfactor: *const c_void,
        x: *const Self::Dense,
        y: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_multiply_f(subfactor.cast(), x, y) }
    }

    unsafe fn subfactor_multiply_in_place(
        subfactor: *const c_void,
        xy: *const Self::Dense,
    ) -> c_int {
        // SAFETY: as above.
        unsafe { sys::accsp_subfactor_multiply_in_place_f(subfactor.cast(), xy) }
    }
}
