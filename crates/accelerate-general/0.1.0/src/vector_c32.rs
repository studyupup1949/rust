use num_complex::Complex;
use std::ffi::{c_float, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Sets all elements in the single-precision complex vector `x` to the given complex value `alpha`.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `alpha`: A pointer to the complex value to set all elements of `x`.
    /// - `x`: A pointer to the vector to be modified in-place.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function, and it is the caller's responsibility to ensure that
    /// the memory region accessed by `x` (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "catlas_cset"]
    pub fn set(n: c_int, alpha: *const Complex<c_float>, x: *mut Complex<c_float>, inc_x: c_int);

    /// Computes the dot product of the complex conjugate of the first single-precision complex vector `X`
    /// with the second single-precision complex vector `Y`.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors.
    /// - `x`: A pointer to the first input vector of complex numbers (complex conjugate).
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the second input vector of complex numbers.
    /// - `inc_y`: The increment between elements in `y`.
    /// - `dotc`: A pointer to store the result of the dot product (complex conjugate).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds, and that the memory pointed to by `dotc` is valid.
    #[link_name = "cblas_cdotc_sub"]
    pub fn dot_conj_plus(
        n: c_int,
        x: *const Complex<c_float>,
        inc_x: c_int,
        y: *const Complex<c_float>,
        inc_y: c_int,
        dotc: *mut Complex<c_float>,
    );

    /// Computes the dot product of two single-precision complex vectors `X` and `Y`, without conjugating the first vector.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the first input vector of `Complex<f32>` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `y`: A pointer to the second input vector of `Complex<f32>` numbers.
    /// - `inc_y`: The increment between elements in `y`.
    /// - `dotu`: A pointer to store the result of the dot product (`Complex<f32>`).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds, and that the memory pointed to by `dotu` is valid.
    #[link_name = "cblas_cdotu_sub"]
    pub fn dot_unconj_plus(
        n: c_int,
        x: *const Complex<c_float>,
        inc_x: c_int,
        y: *const Complex<c_float>,
        inc_y: c_int,
        dotu: *mut Complex<c_float>,
    );

    /// Computes a constant times a vector plus a vector (single-precision complex).
    ///
    /// `Y = alpha * X + Y`
    ///
    /// This function modifies the vector `Y` in-place.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `alpha`: A pointer to the complex scalar that scales vector `X`.
    /// - `x`: A pointer to the input vector `X` (of complex numbers).
    /// - `inc_x`: The stride between elements in vector `X`.
    /// - `y`: A pointer to the input/output vector `Y` (of complex numbers).
    /// - `inc_y`: The stride between elements in vector `Y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions pointed to by `x` and `y` are valid
    /// and that accessing the data up to `n * inc_x` and `n * inc_y` is safe.
    #[link_name = "cblas_caxpy"]
    pub fn scaled_plus(
        n: c_int,                       // Number of elements in vectors
        alpha: *const Complex<c_float>, // Scaling factor for X (complex scalar)
        x: *const Complex<c_float>,     // Input vector X
        inc_x: c_int,                   // Stride within X
        y: *mut Complex<c_float>,       // Input/output vector Y
        inc_y: c_int,                   // Stride within Y
    );

    /// Copies a vector to another vector (single-precision complex).
    ///
    /// Copies the elements from the vector `X` to the vector `Y`.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the source vector `X` (complex numbers).
    /// - `inc_x`: The stride between elements in vector `X`.
    /// - `y`: A pointer to the destination vector `Y` (complex numbers).
    /// - `inc_y`: The stride between elements in vector `Y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions pointed to by `x` and `y` are valid
    /// and that accessing the data up to `n * inc_x` and `n * inc_y` is safe.
    #[link_name = "cblas_ccopy"]
    pub fn copy(
        n: c_int,                   // Number of elements in vectors
        x: *const Complex<c_float>, // Source vector X
        inc_x: c_int,               // Stride within X
        y: *mut Complex<c_float>,   // Destination vector Y
        inc_y: c_int,               // Stride within Y
    );

    /// Multiplies each element of a complex vector by a constant scaling factor.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `X`.
    /// - `alpha`: A pointer to the constant scaling factor.
    /// - `x`: A pointer to the input/output vector `X`. The result is stored in-place.
    /// - `inc_x`: The stride between elements in `X`. For example, if `inc_x` is 7, every 7th element of `X` is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` are valid and that the stride `inc_x` is correctly set.
    ///
    /// # Discussion
    /// The function scales each element in the vector `X` by a constant value `alpha`, modifying `X` in place.
    #[link_name = "cblas_cscal"]
    pub fn scale_by_c32(
        n: c_int,
        alpha: *const Complex<c_float>,
        x: *mut Complex<c_float>,
        inc_x: c_int,
    );

    /// Multiplies each element of a complex vector `X` by a constant scalar `alpha`.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in vector `X`.
    /// - `alpha`: The constant scalar that scales each element of `X`.
    /// - `x`: A pointer to the complex vector `X`, which is modified in place.
    /// - `inc_x`: The increment (stride) between elements in `X`. For example, if `inc_x` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `X` are valid.
    ///
    /// # Discussion
    /// This function scales each element of the complex vector `X` by the constant `alpha`.
    #[link_name = "cblas_csscal"]
    pub fn scale_by_f32(n: c_int, alpha: c_float, x: *mut Complex<c_float>, inc_x: c_int);

    /// Exchanges the elements of two complex vectors `X` and `Y`.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `X` and `Y`.
    /// - `x`: A pointer to the first complex vector `X`. On return, contains elements copied from vector `Y`.
    /// - `inc_x`: The increment (stride) between elements in `X`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `y`: A pointer to the second complex vector `Y`. On return, contains elements copied from vector `X`.
    /// - `inc_y`: The increment (stride) between elements in `Y`. For example, if `inc_y` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `X` and `Y` are valid and within bounds.
    ///
    /// # Discussion
    /// This function swaps the elements between two complex vectors `X` and `Y` in place.
    #[link_name = "cblas_cswap"]
    pub fn swap(
        n: c_int,
        x: *mut Complex<c_float>,
        inc_x: c_int,
        y: *mut Complex<c_float>,
        inc_y: c_int,
    );

    /// Computes the sum of the absolute values of real and imaginary parts of elements in a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `X`.
    /// - `x`: A pointer to the source vector `X` (stored as single-precision complex numbers).
    /// - `inc_x`: The stride between elements in `X`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Return Value
    /// Returns a single floating-point value containing the sum of the absolute values of both the real and imaginary parts of the vector.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` are valid.
    #[link_name = "cblas_scasum"]
    pub fn norm1(
        n: c_int,                   // Number of elements in the vector
        x: *const Complex<c_float>, // Pointer to source vector X
        inc_x: c_int,               // Stride within vector X
    ) -> c_float;

    /// Computes the unitary norm (Euclidean norm or 2-norm) of a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The length of the vector `X`.
    /// - `x`: A pointer to the vector `X` (stored as single-precision complex numbers).
    /// - `inc_x`: The stride between elements in `X`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Return Value
    /// Returns the unitary norm (Euclidean norm) of the vector.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` are valid.
    #[link_name = "cblas_scnrm2"]
    pub fn norm2(
        n: c_int,                   // Length of vector X
        x: *const Complex<c_float>, // Pointer to vector X
        inc_x: c_int,               // Stride within vector X
    ) -> c_float;

    /// Finds the index of the element with the largest absolute value in the single-precision complex vector `x`.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `x`: A pointer to the input vector of `Complex<f32>` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// - The index of the element with the largest absolute value.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_icamax"]
    pub fn argmax_mod(n: c_int, x: *const Complex<c_float>, inc_x: c_int) -> c_int;

    /// Computes the linear combination of two single-precision complex vectors `x` and `y` as:
    /// `y = alpha * x + beta * y`.
    /// This modifies the `y` vector in-place.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `x` and `y`.
    /// - `alpha`: A pointer to the scalar that scales the `x` vector.
    /// - `x`: A pointer to the first input vector of complex numbers.
    /// - `inc_x`: The increment between elements in `x`.
    /// - `beta`: A pointer to the scalar that scales the `y` vector.
    /// - `y`: A pointer to the second input vector, which stores the result in-place.
    /// - `inc_y`: The increment between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function, and it is the caller's responsibility to ensure that
    /// the memory regions accessed by `x` and `y` (up to `n * inc_x` and `n * inc_y` respectively)
    /// are valid and within bounds.
    #[link_name = "catlas_caxpby"]
    pub fn lin_comb_c32_catlas(
        n: c_int,
        alpha: *const Complex<c_float>,
        x: *const Complex<c_float>,
        inc_x: c_int,
        beta: *const Complex<c_float>,
        y: *mut Complex<c_float>,
        inc_y: c_int,
    );

}
