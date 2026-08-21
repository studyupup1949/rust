use num_complex::Complex;
use std::ffi::{c_double, c_float, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Applies a Givens rotation matrix to a pair of vectors `x` and `y`.
    ///
    /// It is applied to each pair of elements from `x` and `y`.
    ///
    /// # Precision
    /// This function operates on `f32` numbers (single-precision).
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `x` and `y`.
    /// - `x`: A pointer to the vector `x`, modified on return.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the vector `y`, modified on return.
    /// - `inc_y`: The increment (stride) between elements in `y`. For example, if `inc_y = 7`, every 7th element is used.
    /// - `c`: The value `cos(θ)` in the Givens rotation matrix.
    /// - `s`: The value `sin(θ)` in the Givens rotation matrix.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `y` (up to `n * inc_x` and `n * inc_y`) are valid and within bounds.
    #[link_name = "cblas_srot"]
    pub fn givens_rot_f32(
        n: c_int,
        x: *mut c_float,
        inc_x: c_int,
        y: *mut c_float,
        inc_y: c_int,
        c: c_float,
        s: c_float,
    );

    /// Constructs a Givens rotation matrix that zeroes the lower value (`b`) in a vertical matrix containing `a` and `b`.
    ///
    /// A Givens rotation is used to introduce zeros into vectors or matrices, which is useful in algorithms like QR decomposition.
    ///
    /// Given two numbers `a` and `b`, this function computes the values of `cos(θ)` (`c`) and `sin(θ)` (`s`) such that the resulting vector after the Givens rotation will have a zero in the second position (`b` becomes `0`).
    ///
    /// # Formula
    /// Givens Rotation Matrix, this matrix rotates a 2D vector such that the second component becomes zero.
    /// Given values `a` and `b`, the function computes the cosine (c = cos(θ)) and sine (s = sin(θ)) that zero out the second component `b`.
    ///
    /// The resulting values are:
    ///
    ///    a' = sqrt(a² + b²),   b' = 0
    ///
    /// where a' is the updated value of `a`, and b' is set to zero.
    ///
    /// # Precision
    /// This function operates on `f32` values (single-precision).
    ///
    /// # Parameters
    /// - `a`: Single-precision value `a`. Overwritten on return with result `r`, the magnitude of the Givens rotation.
    /// - `b`: Single-precision value `b`. Overwritten on return with result `z` (zero).
    /// - `c`: Overwritten on return with the value `cos(θ)`, the cosine of the Givens rotation.
    /// - `s`: Overwritten on return with the value `sin(θ)`, the sine of the Givens rotation.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `a`, `b`, `c`, and `s` are valid memory locations for the results.
    #[link_name = "cblas_srotg"]
    pub fn givens_gen_f32(a: *mut c_float, b: *mut c_float, c: *mut c_float, s: *mut c_float);

    /// Applies a modified Givens transformation to two single-precision vectors `X` and `Y`.
    ///
    /// # Precision
    /// This function operates on `f32` values (single-precision).
    ///
    /// # Parameters
    /// - `n`: The number of elements in the vectors `X` and `Y`.
    /// - `x`: Pointer to the vector `X`, which is modified on return.
    /// - `inc_x`: The increment between elements in `X`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: Pointer to the vector `Y`, which is modified on return.
    /// - `inc_y`: The increment between elements in `Y`. For example, if `inc_y = 7`, every 7th element is used.
    /// - `p`: Pointer to a 5-element vector, where:
    ///   - `p[0]`: Flag that defines the form of matrix `H`. Possible values are:
    ///     - `-2.0`: Matrix `H` is the identity matrix.
    ///     - `-1.0`: Matrix `H` is the matrix `SH` (defined by the remaining values in `p`).
    ///     - `0.0`: Matrix `H[1,2]` and `H[2,1]` are derived from `SH`. The remaining values are `1.0`.
    ///     - `1.0`: Matrix `H[1,1]` and `H[2,2]` are derived from `SH`. `H[1,2]` is `1.0` and `H[2,1]` is `-1.0`.
    ///   - `p[1]`: Value for `SH[1,1]`.
    ///   - `p[2]`: Value for `SH[2,1]`.
    ///   - `p[3]`: Value for `SH[1,2]`.
    ///   - `p[4]`: Value for `SH[2,2]`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions pointed to by `x`, `y`, and `p` are valid.
    #[link_name = "cblas_srotm"]
    pub fn givens_mod_rot_f32(
        n: c_int,
        x: *mut c_float,
        inc_x: c_int,
        y: *mut c_float,
        inc_y: c_int,
        p: *const c_float,
    );

    /// Generates a modified Givens rotation matrix that zeroes the second component of the vector (`sqrt(D1) * B1`, `sqrt(D2) * B2`).
    ///
    /// # Precision
    /// This function operates on `f32` values (single-precision).
    ///
    /// # Parameters
    /// - `d1`: Scaling factor `D1`, overwritten with an updated value on return.
    /// - `d2`: Scaling factor `D2`, overwritten with an updated value on return.
    /// - `b1`: Scaling factor `B1`, overwritten with an updated value on return.
    /// - `b2`: Scaling factor `B2`, used as input.
    /// - `p`: A 5-element vector for storing the resulting modified Givens rotation matrix:
    ///   - `p[0]`: Flag value that defines the form of matrix `H`:
    ///     - `-2.0`: Identity matrix.
    ///     - `-1.0`: Matrix `H` is identical to `SH`.
    ///     - `0.0`: `H[1,2]` and `H[2,1]` are derived from `SH`; other values are `1.0`.
    ///     - `1.0`: `H[1,1]` and `H[2,2]` are derived from `SH`; `H[1,2] = 1.0`, `H[2,1] = -1.0`.
    ///   - `p[1]`: Value for `SH[1,1]`.
    ///   - `p[2]`: Value for `SH[2,1]`.
    ///   - `p[3]`: Value for `SH[1,2]`.
    ///   - `p[4]`: Value for `SH[2,2]`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers to `d1`, `d2`, `b1`, and `p` are valid.
    #[link_name = "cblas_srotmg"]
    pub fn givens_mod_gen_f32(
        d1: *mut c_float,
        d2: *mut c_float,
        b1: *mut c_float,
        b2: c_float,
        p: *mut c_float,
    );

    /// Constructs a complex Givens rotation that zeroes the second element of a 2-element complex vector.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `a`: Pointer to complex value `a`. Overwritten on return with the upper value `r`.
    /// - `b`: Pointer to complex value `b`. Overwritten on return.
    /// - `c`: Pointer to real value `c`. Overwritten on return with the value `cos(θ)`.
    /// - `s`: Pointer to complex value `s`. Overwritten on return with the value `sin(θ)`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `a`, `b`, `c`, and `s` are valid and properly allocated.
    ///
    /// # Discussion
    /// Given complex numbers `a` and `b`, this function computes a Givens rotation such that the second value `b` is zeroed.
    /// It returns the Givens rotation parameters, `cos(θ)` (stored in `c`) and `sin(θ)` (stored in `s`), while updating `a` with the resulting magnitude `r`.
    #[link_name = "cblas_crotg"]
    pub fn givens_gen_c32(
        a: *mut Complex<c_float>,
        b: *mut Complex<c_float>,
        c: *mut c_float,
        s: *mut Complex<c_float>,
    );

    /// Applies a Givens rotation matrix to a pair of complex vectors `X` and `Y`.
    ///
    /// The Givens rotation is applied to each corresponding element from `X` and `Y`.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `X` and `Y`.
    /// - `x`: A pointer to the complex vector `X`, modified on return.
    /// - `inc_x`: The increment (stride) between elements in `X`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `y`: A pointer to the complex vector `Y`, modified on return.
    /// - `inc_y`: The increment (stride) between elements in `Y`. For example, if `inc_y` is 7, every 7th element is used.
    /// - `c`: The value `cos(θ)` in the Givens rotation matrix.
    /// - `s`: The value `sin(θ)` in the Givens rotation matrix.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `X` and `Y` are valid.
    ///
    /// # Discussion
    /// This function applies a Givens rotation matrix to two complex vectors `X` and `Y`. It computes the rotated values and stores the results back in `X` and `Y` in place.
    #[link_name = "cblas_csrot"]
    pub fn givens_rot_c32(
        n: c_int,
        x: *mut Complex<c_float>,
        inc_x: c_int,
        y: *mut Complex<c_float>,
        inc_y: c_int,
        c: c_float,
        s: c_float,
    );

    /// Constructs a complex Givens rotation that zeroes the second element of a 2-element complex vector.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f64>`).
    ///
    /// # Parameters
    /// - `a`: Pointer to complex value `a`. Overwritten on return with the upper value `r`.
    /// - `b`: Pointer to complex value `b`. Overwritten on return.
    /// - `c`: Pointer to real value `c`. Overwritten on return with the value `cos(θ)`.
    /// - `s`: Pointer to complex value `s`. Overwritten on return with the value `sin(θ)`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `a`, `b`, `c`, and `s` are valid and properly allocated.
    ///
    /// # Discussion
    /// Given complex numbers `a` and `b`, this function computes a Givens rotation such that the second value `b` is zeroed.
    /// It returns the Givens rotation parameters, `cos(θ)` (stored in `c`) and `sin(θ)` (stored in `s`), while updating `a` with the resulting magnitude `r`.
    #[link_name = "cblas_zrotg"]
    pub fn givens_gen_c64(
        a: *mut Complex<c_double>,
        b: *mut Complex<c_double>,
        c: *mut c_double,
        s: *mut Complex<c_double>,
    );

    /// Applies a Givens rotation matrix to a pair of complex vectors `X` and `Y`.
    ///
    /// The Givens rotation is applied to each corresponding element from `X` and `Y`.
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f64>`).
    ///
    /// # Parameters
    /// - `n`: The number of elements in vectors `X` and `Y`.
    /// - `x`: A pointer to the complex vector `X`, modified on return.
    /// - `inc_x`: The increment (stride) between elements in `X`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `y`: A pointer to the complex vector `Y`, modified on return.
    /// - `inc_y`: The increment (stride) between elements in `Y`. For example, if `inc_y` is 7, every 7th element is used.
    /// - `c`: The value `cos(θ)` in the Givens rotation matrix.
    /// - `s`: The value `sin(θ)` in the Givens rotation matrix.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `X` and `Y` are valid.
    ///
    /// # Discussion
    /// This function applies a Givens rotation matrix to two complex vectors `X` and `Y`. It computes the rotated values and stores the results back in `X` and `Y` in place.
    #[link_name = "cblas_zsrot"]
    pub fn givens_rot_c64(
        n: c_int,
        x: *mut Complex<c_double>,
        inc_x: c_int,
        y: *mut Complex<c_double>,
        inc_y: c_int,
        c: c_double,
        s: c_double,
    );

}
