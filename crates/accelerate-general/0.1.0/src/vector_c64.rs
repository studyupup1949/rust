use num_complex::Complex;
use std::ffi::{c_double, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Computes the linear combination of two double-precision complex vectors `x` and `y` as:
    /// `y = alpha * x + beta * y`.
    /// This modifies the `y` vector in-place.
    ///
    /// # Precision
    /// This function operates on `Complex<f64>` complex numbers.
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
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by
    /// `x` and `y` (up to `n * inc_x` and `n * inc_y` respectively) are valid and within bounds.
    #[link_name = "catlas_zaxpby"]
    pub fn lin_comb_catlas(
        n: c_int,
        alpha: *const Complex<c_double>,
        x: *const Complex<c_double>,
        inc_x: c_int,
        beta: *const Complex<c_double>,
        y: *mut Complex<c_double>,
        inc_y: c_int,
    );

    /// Sets all elements in the double-precision complex vector `x` to the given complex value `alpha`.
    ///
    /// # Precision
    /// This function operates on `Complex<f64>` complex numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `alpha`: A pointer to the complex value to set all elements of `x`.
    /// - `x`: A pointer to the vector to be modified in-place.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "catlas_zset"]
    pub fn set(n: c_int, alpha: *const Complex<c_double>, x: *mut Complex<c_double>, inc_x: c_int);

    /// Computes the dot product of the complex conjugate of the first double-precision complex vector `X`
    /// with the second double-precision complex vector `Y`, and stores the result in `DOTC`.
    ///
    /// # Precision
    /// This function operates on `Complex<f64>` numbers (double-precision complex numbers).
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the first input vector of `Complex<f64>` numbers. The elements of `x` are conjugated before the dot product.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the second input vector of `Complex<f64>` numbers.
    /// - `inc_y`: The increment (stride) between elements in `y`.
    /// - `dotc`: A pointer to store the result of the conjugated dot product (`Complex<f64>`).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds, and that the memory pointed to by `dotc` is valid.
    #[link_name = "cblas_zdotc_sub"]
    pub fn dot_conj(
        n: c_int,
        x: *const Complex<c_double>,
        inc_x: c_int,
        y: *const Complex<c_double>,
        inc_y: c_int,
        dotc: *mut Complex<c_double>,
    );

    /// Computes the dot product of two double-precision complex vectors `X` and `Y`, and stores the result in `DOTU`.
    ///
    /// # Precision
    /// This function operates on `Complex<f64>` numbers (double-precision complex numbers).
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: A pointer to the first input vector of `Complex<f64>` numbers.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the second input vector of `Complex<f64>` numbers.
    /// - `inc_y`: The increment (stride) between elements in `y`.
    /// - `dotu`: A pointer to store the result of the dot product (`Complex<f64>`).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y`
    /// (up to `n * inc_x` and `n * inc_y`) are valid and within bounds, and that the memory pointed to by `dotu` is valid.
    #[link_name = "cblas_zdotu_sub"]
    pub fn dot_unconj(
        n: c_int,
        x: *const Complex<c_double>,
        inc_x: c_int,
        y: *const Complex<c_double>,
        inc_y: c_int,
        dotu: *mut Complex<c_double>,
    );

    /// Computes the sum of the absolute values of real and imaginary parts of elements in a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f64>`).
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
    #[link_name = "cblas_dzasum"]
    pub fn norm1(
        n: c_int,                    // Number of elements in the vector
        x: *const Complex<c_double>, // Pointer to source vector X
        inc_x: c_int,                // Stride within vector X
    ) -> c_double;

    /// Computes the unitary norm (Euclidean norm or 2-norm) of a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f64>`).
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
    #[link_name = "cblas_dznrm2"]
    pub fn norm2(
        n: c_int,                    // Length of vector X
        x: *const Complex<c_double>, // Pointer to vector X
        inc_x: c_int,                // Stride within vector X
    ) -> c_double;

    /// Finds the index of the element with the largest absolute value in the double-precision complex vector `x`.
    ///
    /// # Precision
    /// This function operates on `Complex<f64>` numbers.
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vector `x`.
    /// - `x`: A pointer to the input vector of `Complex<f64>` numbers.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Returns
    /// - The index of the element with the largest absolute value.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory region accessed by `x`
    /// (up to `n * inc_x`) is valid and within bounds.
    #[link_name = "cblas_izamax"]
    pub fn argmax_mod(n: c_int, x: *const Complex<c_double>, inc_x: c_int) -> c_int;
}
