use std::ffi::{c_double, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Computes the linear combination of two double-precision vectors `x` and `y` as:
    /// `y = alpha * x + beta * y`.
    /// This modifies the `y` vector in-place.
    ///
    /// # Precision
    /// This function operates on `f64` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `x` and `y`.
    /// - `alpha`: The scalar that scales the `x` vector.
    /// - `x`: A pointer to the first input vector of `f64` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `beta`: The scalar that scales the `y` vector.
    /// - `y`: A pointer to the second input vector, which stores the result in-place.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function, and it is the caller's responsibility to ensure that
    /// the memory regions accessed by `x` and `y` (up to `n * inc_x` and `n * inc_y` respectively)
    /// are valid and within bounds.
    #[link_name = "catlas_daxpby"]
    pub fn lin_comb_catlas(
        n: c_int,
        alpha: c_double,
        x: *const c_double,
        inc_x: c_int,
        beta: c_double,
        y: *mut c_double,
        inc_y: c_int,
    );

    /// Sets all elements in the double-precision vector `x` to the given value `alpha`.
    ///
    /// # Precision
    /// This function operates on `f64` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `alpha`: The scalar value to set all elements of `x`.
    /// - `x`: A pointer to the vector to be modified in-place.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function, and it is the caller's responsibility to ensure that
    /// the memory region accessed by `x` (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "catlas_dset"]
    pub fn set(n: c_int, alpha: c_double, x: *mut c_double, inc_x: c_int);

    /// Computes the dot product of two double-precision vectors `X` and `Y`.
    ///
    /// # Precision
    /// This function operates on `f64` (double-precision) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the first input vector of `f64` numbers.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the second input vector of `f64` numbers.
    /// - `inc_y`: The increment (stride) between elements in `y`. For example, if `inc_y = 7`, every 7th element is used.
    ///
    /// # Returns
    /// - The dot product of `x` and `y` as a `f64`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds.
    #[link_name = "cblas_ddot"]
    pub fn dot(
        n: c_int,
        x: *const c_double,
        inc_x: c_int,
        y: *const c_double,
        inc_y: c_int,
    ) -> c_double;

    /// Computes the sum of the absolute values of elements in a double-precision vector.
    ///
    /// # Precision
    /// This function operates on double-precision (`f64`) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// Returns the sum of the absolute values of the elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x` elements) is valid and within bounds.
    #[link_name = "cblas_dasum"]
    pub fn norm1(n: c_int, x: *const c_double, inc_x: c_int) -> c_double;

    /// Computes `y = alpha * x + y` where `x` and `y` are vectors.
    ///
    /// # Precision
    /// This function operates on double-precision (`f64`) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `x` and `y`.
    /// - `alpha`: The scalar factor.
    /// - `x`: A pointer to vector `x`.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to vector `y`, which is modified in place.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions for `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y` elements, respectively) are valid and within bounds.
    #[link_name = "cblas_daxpy"]
    pub fn lin_comb(
        n: c_int,
        alpha: c_double,
        x: *const c_double,
        inc_x: c_int,
        y: *mut c_double,
        inc_y: c_int,
    );

    /// Copies vector `x` to vector `y`.
    ///
    /// # Precision
    /// This function operates on double-precision (`f64`) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `x` and `y`.
    /// - `x`: A pointer to vector `x`.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to vector `y`, which will store the result.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions for `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y` elements, respectively) are valid and within bounds.
    #[link_name = "cblas_dcopy"]
    pub fn copy(n: c_int, x: *const c_double, inc_x: c_int, y: *mut c_double, inc_y: c_int);

    /// Finds the index of the element with the largest absolute value in the double-precision vector `x`.
    ///
    /// # Precision
    /// This function operates on `f64` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `x`: A pointer to the input vector of `f64` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// - The index of the element with the largest absolute value.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_idamax"]
    pub fn argmax_mod(n: c_int, x: *const c_double, inc_x: c_int) -> c_int;

}
