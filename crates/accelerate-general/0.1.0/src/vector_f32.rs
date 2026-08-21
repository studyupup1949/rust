use std::ffi::{c_double, c_float, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Computes the linear combination of two single-precision vectors `x` and `y` as:
    /// `y = alpha * x + beta * y`.
    /// This modifies the `y` vector in-place.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `x` and `y`.
    /// - `alpha`: The scalar that scales the `x` vector.
    /// - `x`: A pointer to the first input vector of `f32` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `beta`: The scalar that scales the `y` vector.
    /// - `y`: A pointer to the second input vector, which stores the result in-place.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function, and it is the caller's responsibility to ensure that
    /// the memory regions accessed by `x` and `y` (up to `n * inc_x` and `n * inc_y` respectively)
    /// are valid and within bounds.
    #[link_name = "catlas_saxpby"]
    pub fn lin_comb_catlas(
        n: c_int,
        alpha: c_float,
        x: *const c_float,
        inc_x: c_int,
        beta: c_float,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Sets all elements in the single-precision vector `x` to the given value `alpha`.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
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
    #[link_name = "catlas_sset"]
    pub fn set(n: c_int, alpha: c_float, x: *mut c_float, inc_x: c_int);

    /// Computes the dot product of two single-precision vectors.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `x` and `y`.
    /// - `x`: A pointer to the first input vector of `f32` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the second input vector of `f32` numbers.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Returns
    /// - The dot product of `x` and `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds.
    #[link_name = "cblas_sdot"]
    pub fn dot(
        n: c_int,
        x: *const c_float,
        inc_x: c_int,
        y: *const c_float,
        inc_y: c_int,
    ) -> c_float;

    /// Computes the dot product of two single-precision vectors, with an added scalar.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `x` and `y`.
    /// - `sb`: A scalar value to be added to the result of the dot product.
    /// - `x`: A pointer to the first input vector of `f32` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the second input vector of `f32` numbers.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Returns
    /// - The scalar `sb` added to the dot product of `x` and `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds.
    #[link_name = "cblas_sdsdot"]
    pub fn dot_plus(
        n: c_int,
        sb: c_float,
        x: *const c_float,
        inc_x: c_int,
        y: *const c_float,
        inc_y: c_int,
    ) -> c_float;

    /// Computes the dot product of two single-precision vectors `X` and `Y`, and returns the result as a double-precision number.
    ///
    /// # Precision
    /// This function operates on `f32` (single-precision) vectors and returns an `f64` (double-precision) result.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the first input vector of `f32` numbers.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the second input vector of `f32` numbers.
    /// - `inc_y`: The increment (stride) between elements in `y`.
    ///
    /// # Returns
    /// - The dot product of `x` and `y` as a `f64`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds.
    #[link_name = "cblas_dsdot"]
    pub fn dot_as_f64(
        n: c_int,
        x: *const c_float,
        inc_x: c_int,
        y: *const c_float,
        inc_y: c_int,
    ) -> c_double;

    /// Computes the sum of the absolute values of the elements in the single-precision vector `x`.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `x`: A pointer to the input vector of `f32` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// - The sum of the absolute values of the elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_sasum"]
    pub fn norm1(n: c_int, x: *const c_float, inc_x: c_int) -> c_float;

    /// Performs the operation y = alpha * x + y for single-precision vectors.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors.
    /// - `alpha`: The scalar that scales the vector `x`.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the input/output vector `y`, which stores the result in-place.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y` are valid and within bounds.
    #[link_name = "cblas_saxpy"]
    pub fn scale_plus(
        n: c_int,
        alpha: c_float,
        x: *const c_float,
        inc_x: c_int,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Multiplies each element of a vector by a constant.
    ///
    /// This function performs the operation `x[i] = alpha * x[i]` for each element in the vector `x`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `alpha`: The constant to multiply each element of `x` by.
    /// - `x`: A pointer to the vector `x`. The result is stored in-place.
    /// - `inc_x`: Stride within `x`. For example, if `inc_x` is 7, every 7th element is scaled by `alpha`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointer passed to `x` is valid and that accessing `x` up to `n * inc_x` is safe.
    #[link_name = "cblas_sscal"]
    pub fn scale(n: c_int, alpha: c_float, x: *mut c_float, inc_x: c_int);

    /// Exchanges the elements of two single-precision vectors `x` and `y`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `x` and `y`.
    /// - `x`: A pointer to the first vector `x`. On return, contains elements copied from vector `y`.
    /// - `inc_x`: The increment between elements in vector `x`.
    /// - `y`: A pointer to the second vector `y`. On return, contains elements copied from vector `x`.
    /// - `inc_y`: The increment between elements in vector `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `x` and `y` are valid, and that accessing `x` and `y` up to `n * inc_x` and `n * inc_y` is safe.
    #[link_name = "cblas_sswap"]
    pub fn swap(n: c_int, x: *mut f32, inc_x: c_int, y: *mut f32, inc_y: c_int);

    /// Computes the L2 norm (Euclidean length) of a single-precision vector `x`.
    ///
    /// # Precision
    /// This function operates on `f32` numbers (single-precision).
    ///
    /// # Parameters
    /// - `n`: Length of vector `x`.
    /// - `x`: A pointer to the vector `x`.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Returns
    /// - The L2 norm (Euclidean length) of vector `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x` (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_snrm2"]
    pub fn norm2(n: c_int, x: *const c_float, inc_x: c_int) -> c_float;

    /// Copies the contents of vector `x` into vector `y` for single-precision vectors.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the output vector `y`.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y` are valid and within bounds.
    #[link_name = "cblas_scopy"]
    pub fn copy(n: c_int, x: *const c_float, inc_x: c_int, y: *mut c_float, inc_y: c_int);

    /// Finds the index of the element with the largest absolute value in the single-precision vector `x`.
    ///
    /// # Precision
    /// This function operates on `f32` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `x`: A pointer to the input vector of `f32` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// - The index of the element with the largest absolute value.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_isamax"]
    pub fn argmax_mod(n: c_int, x: *const c_float, inc_x: c_int) -> c_int;
}
