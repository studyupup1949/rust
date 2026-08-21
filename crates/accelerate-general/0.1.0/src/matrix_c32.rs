use crate::constants::{IsDiagUnit, MultiplyOrder, RowColMajor, TransposeMode, UpOrLowTriangle};
use num_complex::Complex;
use std::ffi::{c_float, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Scales a general band matrix, then multiplies it by a vector, and adds another vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (`Complex<f32>`).
    ///
    /// Computes the operation `alpha * A * x + beta * y` or `alpha * A^T * x + beta * y`, where `A` is a band matrix.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `trans_a`: Specifies whether to use matrix `A`, its transpose, or conjugate transpose.
    /// - `m`: The number of rows in matrix `A`.
    /// - `n`: The number of columns in matrix `A`.
    /// - `kl`: The number of sub-diagonals in matrix `A`.
    /// - `ku`: The number of super-diagonals in matrix `A`.
    /// - `alpha`: Scaling factor applied to matrix `A`.
    /// - `a`: A pointer to the band matrix `A` (complex values).
    /// - `lda`: The leading dimension of matrix `A`, must be at least `kl + ku + 1`.
    /// - `x`: A pointer to the input vector `X` (complex values).
    /// - `inc_x`: The stride within vector `X`.
    /// - `beta`: Scaling factor applied to vector `Y`.
    /// - `y`: A pointer to the output vector `Y` (complex values).
    /// - `inc_y`: The stride within vector `Y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions pointed to by `a`, `x`, and `y` are valid and
    /// that accessing the data up to `m * n`, `n * inc_x`, and `m * inc_y` elements is safe.
    #[link_name = "cblas_cgbmv"]
    pub fn band_mat_vec_mul(
        major: RowColMajor,             // Row-major or column-major
        trans_a: TransposeMode,         // Transpose mode for matrix A
        m: c_int,                       // Number of rows in matrix A
        n: c_int,                       // Number of columns in matrix A
        kl: c_int,                      // Number of sub-diagonals in matrix A
        ku: c_int,                      // Number of super-diagonals in matrix A
        alpha: *const Complex<c_float>, // Scaling factor alpha
        a: *const Complex<c_float>,     // Pointer to band matrix A
        lda: c_int,                     // Leading dimension of matrix A
        x: *const Complex<c_float>,     // Pointer to input vector X
        inc_x: c_int,                   // Stride within vector X
        beta: *const Complex<c_float>,  // Scaling factor beta
        y: *mut Complex<c_float>,       // Pointer to output vector Y
        inc_y: c_int,                   // Stride within vector Y
    );

    /// Multiplies two matrices (single-precision complex) and optionally adds a scaled matrix.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (`Complex<f32>`).
    ///
    /// Computes the matrix-matrix product and adds the result to a scaled matrix `C`:
    /// `C = alpha * A * B + beta * C` or `C = alpha * B * A + beta * C`.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `trans_a`: Specifies whether to transpose matrix `A`.
    /// - `trans_b`: Specifies whether to transpose matrix `B`.
    /// - `m`: The number of rows in matrices `A` and `C`.
    /// - `n`: The number of columns in matrices `B` and `C`.
    /// - `k`: The number of columns in matrix `A` and the number of rows in matrix `B`.
    /// - `alpha`: The scaling factor applied to the product of `A` and `B`.
    /// - `a`: A pointer to matrix `A` (complex values).
    /// - `lda`: The leading dimension of matrix `A` (usually the number of rows if column-major, or columns if row-major).
    /// - `b`: A pointer to matrix `B` (complex values).
    /// - `ldb`: The leading dimension of matrix `B` (same as `lda`, typically).
    /// - `beta`: The scaling factor applied to matrix `C`.
    /// - `c`: A pointer to matrix `C` (complex values), where the result will be stored.
    /// - `ldc`: The leading dimension of matrix `C` (same as `lda`, typically).
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions for `a`, `b`, and `c` are valid and that the dimensions and strides provided match the matrix layout.
    #[link_name = "cblas_cgemm"]
    pub fn mat_mul_add(
        major: RowColMajor,             // Row-major or column-major
        trans_a: TransposeMode,         // Transpose mode for matrix A
        trans_b: TransposeMode,         // Transpose mode for matrix B
        m: c_int,                       // Number of rows in matrices A and C
        n: c_int,                       // Number of columns in matrices B and C
        k: c_int,                       // Number of columns in A and rows in B
        alpha: *const Complex<c_float>, // Scaling factor for the product of A and B
        a: *const Complex<c_float>,     // Pointer to matrix A
        lda: c_int,                     // Leading dimension of matrix A
        b: *const Complex<c_float>,     // Pointer to matrix B
        ldb: c_int,                     // Leading dimension of matrix B
        beta: *const Complex<c_float>,  // Scaling factor for matrix C
        c: *mut Complex<c_float>,       // Pointer to matrix C (output)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// General matrix-vector multiplication (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (`Complex<c_float>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `trans`: Specifies whether to transpose matrix `A`.
    /// - `m`: Number of rows in matrix `A`.
    /// - `n`: Number of columns in matrix `A`.
    /// - `alpha`: The scalar factor applied to matrix `A`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `x`: A pointer to vector `x`.
    /// - `inc_x`: The increment between elements in vector `x`.
    /// - `beta`: The scalar factor applied to vector `y`.
    /// - `y`: A pointer to vector `y`, which stores the result in-place.
    /// - `inc_y`: The increment between elements in vector `y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions accessed by `a`, `x`, and `y` are valid.
    #[link_name = "cblas_cgemv"]
    pub fn mat_vec_mul(
        major: RowColMajor,
        trans: TransposeMode,
        m: c_int,
        n: c_int,
        alpha: *const Complex<c_float>,
        a: *const Complex<c_float>,
        lda: c_int,
        x: *const Complex<c_float>,
        inc_x: c_int,
        beta: *const Complex<c_float>,
        y: *mut Complex<c_float>,
        inc_y: c_int,
    );

    /// Performs a rank-1 update of a matrix using the conjugate transpose of vector `Y`:
    /// `A = alpha * x * conjg(y') + A` (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (`Complex<c_float>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `m`: The number of rows in matrix `A`.
    /// - `n`: The number of columns in matrix `A`.
    /// - `alpha`: The scalar factor applied to vector `X`.
    /// - `x`: A pointer to the input vector `X`.
    /// - `inc_x`: The increment between elements in vector `X`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the input vector `Y`.
    /// - `inc_y`: The increment between elements in vector `Y`. For example, if `inc_y = 7`, every 7th element is used.
    /// - `a`: A pointer to the matrix `A`, which is updated in-place.
    /// - `lda`: The leading dimension of matrix `A`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions accessed by `x`, `y`, and `A` are valid.
    #[link_name = "cblas_cgerc"]
    pub fn mat_rank1_conj_update(
        major: RowColMajor,
        m: c_int,
        n: c_int,
        alpha: *const Complex<c_float>,
        x: *const Complex<c_float>,
        inc_x: c_int,
        y: *const Complex<c_float>,
        inc_y: c_int,
        a: *mut Complex<c_float>,
        lda: c_int,
    );

    /// Performs a rank-1 update: `A = alpha * x * y^T + A`
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `m`: The number of rows in matrix `A`.
    /// - `n`: The number of columns in matrix `A`.
    /// - `alpha`: The scaling factor for vector `x`.
    /// - `x`: A pointer to the input vector `x` (single-precision complex).
    /// - `inc_x`: The increment (stride) between elements in vector `x`.
    /// - `y`: A pointer to the input vector `y` (single-precision complex).
    /// - `inc_y`: The increment (stride) between elements in vector `y`.
    /// - `a`: A pointer to the matrix `A` (single-precision complex) which will be updated in place.
    /// - `lda`: The leading dimension of matrix `A`.
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_cgeru"]
    pub fn rank1_update_unconj(
        major: RowColMajor,             // Row or Column Major
        m: c_int,                       // Number of rows in matrix A
        n: c_int,                       // Number of columns in matrix A
        alpha: *const Complex<c_float>, // Scaling factor for vector x (complex)
        x: *const Complex<c_float>,     // Pointer to vector x (complex)
        inc_x: c_int,                   // Increment for vector x
        y: *const Complex<c_float>,     // Pointer to vector y (complex)
        inc_y: c_int,                   // Increment for vector y
        a: *mut Complex<c_float>,       // Pointer to matrix A (complex, output)
        lda: c_int,                     // Leading dimension of matrix A
    );

    /// Scales a Hermitian band matrix, then multiplies by a vector, then adds a vector.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Valid values are 'U' (Upper) or 'L' (Lower).
    /// - `n`: The order of the Hermitian matrix `A` (i.e., the number of rows/columns).
    /// - `k`: Half-bandwidth of the Hermitian band matrix `A`.
    /// - `alpha`: The scaling factor to multiply the Hermitian band matrix `A` by.
    /// - `a`: A pointer to the Hermitian band matrix `A` (complex).
    /// - `lda`: The leading dimension of the matrix `A`.
    /// - `x`: A pointer to the input vector `X` (complex).
    /// - `inc_x`: The increment (stride) between elements in vector `X`.
    /// - `beta`: The scaling factor to multiply the vector `Y` by.
    /// - `y`: A pointer to the output vector `Y` (complex), which will be replaced with the result.
    /// - `inc_y`: The increment (stride) between elements in vector `Y`.
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_chbmv"]
    pub fn herm_band_mat_vec_mul(
        major: RowColMajor,             // Row or Column Major
        tri: UpOrLowTriangle,           // Upper or Lower triangle
        n: c_int,                       // Order of the matrix A
        k: c_int,                       // Half-bandwidth of the Hermitian band matrix A
        alpha: *const Complex<c_float>, // Scaling factor alpha (complex)
        a: *const Complex<c_float>,     // Pointer to the Hermitian band matrix A (complex)
        lda: c_int,                     // Leading dimension of matrix A
        x: *const Complex<c_float>,     // Pointer to vector X (complex)
        inc_x: c_int,                   // Stride within vector X
        beta: *const Complex<c_float>,  // Scaling factor beta (complex)
        y: *mut Complex<c_float>,       // Pointer to vector Y (complex, output)
        inc_y: c_int,                   // Stride within vector Y
    );

    /// Multiplies two Hermitian matrices, then adds a third matrix with scaling.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `side`: Determines the order in which the matrices should be multiplied.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the Hermitian matrix. Valid values are 'U' (Upper) or 'L' (Lower).
    /// - `m`: The number of rows in matrices `A` and `C`.
    /// - `n`: The number of columns in matrices `B` and `C`.
    /// - `alpha`: The scaling factor applied to the product of matrices `A` and `B` (complex).
    /// - `a`: A pointer to the Hermitian matrix `A` (complex).
    /// - `lda`: The leading dimension of matrix `A` (typically the number of rows).
    /// - `b`: A pointer to matrix `B` (complex).
    /// - `ldb`: The leading dimension of matrix `B` (typically the number of rows).
    /// - `beta`: The scaling factor applied to matrix `C` (complex).
    /// - `c`: A pointer to matrix `C`, where the result is stored (complex).
    /// - `ldc`: The leading dimension of matrix `C` (typically the number of rows).
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_chemm"]
    pub fn herm_mat_mul_add(
        major: RowColMajor,             // Row or Column Major
        side: MultiplyOrder,            // Left or Right multiplication
        tri: UpOrLowTriangle,           // Upper or Lower triangle
        m: c_int,                       // Number of rows in A and C
        n: c_int,                       // Number of columns in B and C
        alpha: *const Complex<c_float>, // Scaling factor alpha (complex)
        a: *const Complex<c_float>,     // Pointer to Hermitian matrix A (complex)
        lda: c_int,                     // Leading dimension of matrix A
        b: *const Complex<c_float>,     // Pointer to matrix B (complex)
        ldb: c_int,                     // Leading dimension of matrix B
        beta: *const Complex<c_float>,  // Scaling factor beta (complex)
        c: *mut Complex<c_float>,       // Pointer to matrix C (complex, output)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// Scales and multiplies a Hermitian matrix by a vector, then adds a second (scaled) vector.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Valid values are 'U' (Upper) or 'L' (Lower).
    /// - `n`: The order (number of rows and columns) of the Hermitian matrix `A`.
    /// - `alpha`: The scaling factor applied to the matrix `A` (complex).
    /// - `a`: A pointer to the Hermitian matrix `A` (complex).
    /// - `lda`: The leading dimension of matrix `A` (typically the number of rows).
    /// - `x`: A pointer to vector `X` (complex).
    /// - `inc_x`: The increment between elements in `X`. For example, if `inc_x = 7`, every 7th element of `X` is used.
    /// - `beta`: The scaling factor applied to the vector `Y` (complex).
    /// - `y`: A pointer to vector `Y` (complex), which will store the result in-place.
    /// - `inc_y`: The increment between elements in `Y`. For example, if `inc_y = 7`, every 7th element of `Y` is used.
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_chemv"]
    pub fn herm_mat_vec_mul_add(
        major: RowColMajor,             // Row or Column Major
        tri: UpOrLowTriangle,           // Upper or Lower triangle of Hermitian matrix
        n: c_int,                       // Order of the matrix A
        alpha: *const Complex<c_float>, // Scaling factor alpha (complex)
        a: *const Complex<c_float>,     // Pointer to Hermitian matrix A (complex)
        lda: c_int,                     // Leading dimension of matrix A
        x: *const Complex<c_float>,     // Pointer to vector X (complex)
        inc_x: c_int,                   // Increment between elements in X
        beta: *const Complex<c_float>,  // Scaling factor beta (complex)
        y: *mut Complex<c_float>,       // Pointer to vector Y (complex, output)
        inc_y: c_int,                   // Increment between elements in Y
    );

    /// Hermitian rank 1 update: adds the product of a scaling factor, vector X, and the conjugate transpose of X to matrix A.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper ('U') or lower ('L') triangle from the matrix.
    /// - `n`: The order of matrix `A` (the number of rows and columns).
    /// - `alpha`: The scaling factor for vector `X` (real scalar).
    /// - `x`: A pointer to vector `X` (complex).
    /// - `inc_x`: The increment between elements in `X`. For example, if `inc_x = 7`, every 7th element of `X` is used.
    /// - `a`: A pointer to Hermitian matrix `A` (complex), which will be updated in-place.
    /// - `lda`: The leading dimension of matrix `A` (typically the number of rows).
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_cher"]
    pub fn herm_rank1_update(
        major: RowColMajor,         // Row or Column Major
        tri: UpOrLowTriangle,       // Upper or Lower triangle of Hermitian matrix
        n: c_int,                   // Order of matrix A
        alpha: c_float,             // Scaling factor (real scalar)
        x: *const Complex<c_float>, // Pointer to vector X (complex)
        inc_x: c_int,               // Increment between elements in X
        a: *mut Complex<c_float>,   // Pointer to Hermitian matrix A (complex)
        lda: c_int,                 // Leading dimension of matrix A
    );

    /// Hermitian rank 2 update: adds the product of a scaling factor, vector X, and the conjugate transpose of vector Y
    /// to the product of the conjugate of the scaling factor, vector Y, and the conjugate transpose of vector X, and adds the result to matrix A.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper ('U') or lower ('L') triangle of the Hermitian matrix.
    /// - `n`: The order of the matrix `A` (the number of rows and columns).
    /// - `alpha`: The scaling factor `α` (complex scalar).
    /// - `x`: A pointer to vector `X` (complex).
    /// - `inc_x`: The increment between elements in `X`. For example, if `inc_x = 7`, every 7th element of `X` is used.
    /// - `y`: A pointer to vector `Y` (complex).
    /// - `inc_y`: The increment between elements in `Y`. For example, if `inc_y = 7`, every 7th element of `Y` is used.
    /// - `a`: A pointer to Hermitian matrix `A` (complex), which will be updated in-place.
    /// - `lda`: The leading dimension of the matrix `A` (typically the number of rows).
    ///
    /// # Safety
    /// This function is `unsafe` as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_cher2"]
    pub fn herm_rank2_update(
        major: RowColMajor,             // Row or Column Major
        tri: UpOrLowTriangle,           // Upper or Lower triangle of Hermitian matrix
        n: c_int,                       // Order of matrix A
        alpha: *const Complex<c_float>, // Scaling factor (complex scalar)
        x: *const Complex<c_float>,     // Pointer to vector X (complex)
        inc_x: c_int,                   // Increment between elements in X
        y: *const Complex<c_float>,     // Pointer to vector Y (complex)
        inc_y: c_int,                   // Increment between elements in Y
        a: *mut Complex<c_float>,       // Pointer to Hermitian matrix A (complex)
        lda: c_int,                     // Leading dimension of matrix A
    );

    /// Rank-k update: multiplies a Hermitian matrix by its transpose and adds a second matrix.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper ('U') or lower ('L') triangle of the Hermitian matrix.
    /// - `trans`: Specifies whether to use matrix `A` ('N') or the conjugate transpose of `A` ('C').
    /// - `n`: The order of matrix `C` (the number of rows and columns).
    /// - `k`: The number of columns in matrix `A` (or number of rows if matrix `A` is transposed).
    /// - `alpha`: The scaling factor for matrix `A`.
    /// - `a`: A pointer to matrix `A` (complex).
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `beta`: The scaling factor for matrix `C`.
    /// - `c`: A pointer to matrix `C` (complex), which will be updated in-place.
    /// - `ldc`: The leading dimension of matrix `C`.
    ///
    /// # Safety
    /// This is an `unsafe` function as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_cherk"]
    pub fn herm_rank_k_update(
        major: RowColMajor,         // Row or Column Major
        tri: UpOrLowTriangle,       // Upper or Lower triangle of Hermitian matrix
        trans: TransposeMode,       // Transpose mode (None, Conjugate Transpose)
        n: c_int,                   // Order of matrix C
        k: c_int,                   // Number of columns of A (or rows if transposed)
        alpha: c_float,             // Scaling factor for A
        a: *const Complex<c_float>, // Pointer to matrix A (complex)
        lda: c_int,                 // Leading dimension of matrix A
        beta: c_float,              // Scaling factor for matrix C
        c: *mut Complex<c_float>,   // Pointer to matrix C (complex)
        ldc: c_int,                 // Leading dimension of matrix C
    );

    /// Performs a rank-2k update of a complex Hermitian matrix.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper ('U') or lower ('L') triangle of the Hermitian matrix.
    /// - `trans`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate transpose of `A` ('C').
    /// - `n`: The order of matrix `C` (the number of rows and columns).
    /// - `k`: The number of columns in matrices `A` and `B` (or rows if transposed).
    /// - `alpha`: The scaling factor for matrices `A` and `B`.
    /// - `a`: A pointer to matrix `A` (complex).
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `b`: A pointer to matrix `B` (complex).
    /// - `ldb`: The leading dimension of matrix `B`.
    /// - `beta`: The scaling factor for matrix `C`.
    /// - `c`: A pointer to matrix `C` (complex), which will be updated in-place.
    /// - `ldc`: The leading dimension of matrix `C`.
    ///
    /// # Safety
    /// This is an `unsafe` function as it involves raw pointers and does not perform bounds checking.
    #[link_name = "cblas_cher2k"]
    pub fn herm_rank_2k_update(
        major: RowColMajor,             // Row or Column Major
        tri: UpOrLowTriangle,           // Upper or Lower triangle of Hermitian matrix
        trans: TransposeMode,           // Transpose mode (None, Transpose, Conjugate Transpose)
        n: c_int,                       // Order of matrix C
        k: c_int,                       // Number of columns of A and B (or rows if transposed)
        alpha: *const Complex<c_float>, // Pointer to the scaling factor for A and B
        a: *const Complex<c_float>,     // Pointer to matrix A (complex)
        lda: c_int,                     // Leading dimension of matrix A
        b: *const Complex<c_float>,     // Pointer to matrix B (complex)
        ldb: c_int,                     // Leading dimension of matrix B
        beta: c_float,                  // Scaling factor for matrix C
        c: *mut Complex<c_float>,       // Pointer to matrix C (complex)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// Performs the operation y = alpha * A * x + beta * y, where A is a Hermitian matrix stored in packed format.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` complex numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper ('U') or lower ('L') triangle of the Hermitian matrix.
    /// - `n`: The order of matrix `A` and the number of elements in vectors `x` and `y`.
    /// - `alpha`: Scaling factor applied to the matrix `A`.
    /// - `ap`: A pointer to the Hermitian matrix `A` stored in packed format (complex).
    /// - `x`: A pointer to the vector `X` (complex).
    /// - `inc_x`: The increment between elements in `X`.
    /// - `beta`: Scaling factor applied to the vector `Y`.
    /// - `y`: A pointer to the vector `Y` (complex), which stores the result in-place.
    /// - `inc_y`: The increment between elements in `Y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the pointers passed to `ap`, `x`, and `y` are valid and within bounds.
    #[link_name = "cblas_chpmv"]
    pub fn pack_herm_mat_vec_mul(
        major: RowColMajor,             // RowColMajor: Row or Column major data ordering
        tri: UpOrLowTriangle, // UpOrLowTriangle: Upper or Lower triangle of Hermitian matrix
        n: c_int,             // Number of rows/columns in matrix A
        alpha: *const Complex<c_float>, // Pointer to scaling factor for A (complex)
        ap: *const Complex<c_float>, // Pointer to packed Hermitian matrix A (complex)
        x: *const Complex<c_float>, // Pointer to vector X (complex)
        inc_x: c_int,         // Stride between elements in X
        beta: *const Complex<c_float>, // Pointer to scaling factor for Y (complex)
        y: *mut Complex<c_float>, // Pointer to vector Y (complex), result is stored in-place
        inc_y: c_int,         // Stride between elements in Y
    );

    /// Scales and multiplies a vector times its conjugate transpose, then adds a packed Hermitian matrix.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers (single-precision complex).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Valid values are `'U'` or `'L'`.
    /// - `n`: Order of matrix `A` and the number of elements in vector `X`.
    /// - `alpha`: Scaling factor that vector `X` is multiplied by.
    /// - `x`: Pointer to the vector `X`.
    /// - `inc_x`: Stride within `X`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `ap`: Pointer to packed Hermitian matrix `A`, which is overwritten by the results on return.
    ///
    /// # Safety
    /// This function is unsafe as it deals with raw pointers and FFI.
    #[link_name = "cblas_chpr"]
    pub fn pack_hermitian_rank1_update(
        major: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        alpha: c_float,
        x: *const Complex<c_float>,
        inc_x: c_int,
        ap: *mut Complex<c_float>,
    );

    /// Multiplies a vector times the conjugate transpose of a second vector and vice-versa, sums the results, and adds a packed Hermitian matrix.
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers (single-precision complex).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Valid values are `'U'` or `'L'`.
    /// - `n`: Order of matrix `A` and the number of elements in vectors `X` and `Y`.
    /// - `alpha`: Scaling factor that vector `X` is multiplied by.
    /// - `x`: Pointer to the vector `X`.
    /// - `inc_x`: Stride within `X`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `y`: Pointer to the vector `Y`.
    /// - `inc_y`: Stride within `Y`. For example, if `inc_y` is 7, every 7th element is used.
    /// - `ap`: Pointer to packed Hermitian matrix `A`, which is overwritten by the results on return.
    ///
    /// # Safety
    /// This function is unsafe as it deals with raw pointers and FFI.
    #[link_name = "cblas_chpr2"]
    pub fn pack_hermitian_rank2_update(
        major: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        alpha: *const Complex<c_float>,
        x: *const Complex<c_float>,
        inc_x: c_int,
        y: *const Complex<c_float>,
        inc_y: c_int,
        ap: *mut Complex<c_float>,
    );

    /// Multiplies a matrix by a symmetric matrix (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` (single-precision complex) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `side`: Determines whether matrix `A` is on the left or right side of the multiplication (left or right of matrix `B`).
    /// - `tri`: Specifies whether to use the upper or lower triangle from the symmetric matrix `A`.
    /// - `m`: The number of rows in matrices `A` and `C`.
    /// - `n`: The number of columns in matrices `B` and `C`.
    /// - `alpha`: The scaling factor applied to the product of matrices `A` and `B`.
    /// - `a`: A pointer to the symmetric matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. It should be at least `m` (for column-major) or `n` (for row-major).
    /// - `b`: A pointer to matrix `B`.
    /// - `ldb`: The leading dimension of matrix `B`. It should be at least `m` (for column-major) or `n` (for row-major).
    /// - `beta`: The scaling factor applied to matrix `C`.
    /// - `c`: A pointer to the result matrix `C`.
    /// - `ldc`: The leading dimension of matrix `C`. It should be at least `m` (for column-major) or `n` (for row-major).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A`, `B`, and `C` are valid and within bounds.
    ///
    /// # Discussion
    /// This function computes one of the following, depending on the value of `side`:
    /// - `C ← α * A * B + β * C` (if `side` is left),
    /// - `C ← α * B * A + β * C` (if `side` is right).
    #[link_name = "cblas_csymm"]
    pub fn sym_mat_mul(
        major: RowColMajor,             // RowColMajor
        side: MultiplyOrder,            // MultiplyOrder
        tri: UpOrLowTriangle,           // UpOrLowTriangle
        m: c_int,                       // Number of rows in matrices A and C
        n: c_int,                       // Number of columns in matrices B and C
        alpha: *const Complex<c_float>, // Scaling factor for matrix A
        a: *const Complex<c_float>,     // Pointer to symmetric matrix A
        lda: c_int,                     // Leading dimension of matrix A
        b: *const Complex<c_float>,     // Pointer to matrix B
        ldb: c_int,                     // Leading dimension of matrix B
        beta: *const Complex<c_float>,  // Scaling factor for matrix C
        c: *mut Complex<c_float>,       // Pointer to matrix C (output)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// Performs a rank-2k update of a symmetric matrix (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers (single-precision complex).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Valid values are 'U' (upper) or 'L' (lower).
    /// - `trans`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate transpose of `A` ('C').
    /// - `n`: The order of matrix `C` (number of rows and columns).
    /// - `k`: The number of columns in matrices `A` and `B` if `trans = 'N'`, or the number of rows if `trans = 'T'` or `trans = 'C'`.
    /// - `alpha`: The scaling factor applied to the product of matrices `A` and `B`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `b`: A pointer to matrix `B`.
    /// - `ldb`: The leading dimension of matrix `B`.
    /// - `beta`: The scaling factor applied to matrix `C`.
    /// - `c`: A pointer to the result matrix `C`, where the result is stored in place.
    /// - `ldc`: The leading dimension of matrix `C`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A`, `B`, and `C` are valid and within bounds.
    ///
    /// # Discussion
    /// This function computes:
    /// `C ← α * A * B^T + α * B * A^T + β * C`,
    /// where `A` and `B` are matrices, and `C` is a symmetric matrix.
    #[link_name = "cblas_csyr2k"]
    pub fn sym_rank_2k_update(
        major: RowColMajor,             // RowColMajor
        tri: UpOrLowTriangle,           // UpOrLowTriangle
        trans: TransposeMode,           // TransposeMode
        n: c_int,                       // Order of matrix C
        k: c_int,                       // Number of columns (or rows) in matrices A and B
        alpha: *const Complex<c_float>, // Scaling factor for A and B
        a: *const Complex<c_float>,     // Pointer to matrix A
        lda: c_int,                     // Leading dimension of matrix A
        b: *const Complex<c_float>,     // Pointer to matrix B
        ldb: c_int,                     // Leading dimension of matrix B
        beta: *const Complex<c_float>,  // Scaling factor for matrix C
        c: *mut Complex<c_float>,       // Pointer to matrix C (output)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// Performs a rank-k update of a symmetric matrix (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `Complex<f32>` numbers (single-precision complex).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Valid values are 'U' (upper) or 'L' (lower).
    /// - `trans`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T').
    /// - `n`: The order of matrix `C` (number of rows and columns).
    /// - `k`: The number of columns in matrix `A` if `trans = 'N'`, or the number of rows if `trans = 'T'`.
    /// - `alpha`: The scaling factor applied to matrix `A`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `beta`: The scaling factor applied to matrix `C`.
    /// - `c`: A pointer to the result matrix `C`, where the result is stored in place.
    /// - `ldc`: The leading dimension of matrix `C`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A` and `C` are valid and within bounds.
    ///
    /// # Discussion
    /// This function computes:
    /// `C ← α * A * A^T + β * C` when `trans = 'N'`,
    /// or
    /// `C ← α * A^T * A + β * C` when `trans = 'T'`.
    #[link_name = "cblas_csyrk"]
    pub fn sym_rank_k_update(
        major: RowColMajor,             // RowColMajor
        tri: UpOrLowTriangle,           // UpOrLowTriangle
        trans: TransposeMode,           // TransposeMode
        n: c_int,                       // Order of matrix C
        k: c_int,                       // Number of columns (or rows) in matrix A
        alpha: *const Complex<c_float>, // Scaling factor for matrix A
        a: *const Complex<c_float>,     // Pointer to matrix A
        lda: c_int,                     // Leading dimension of matrix A
        beta: *const Complex<c_float>,  // Scaling factor for matrix C
        c: *mut Complex<c_float>,       // Pointer to matrix C (output)
        ldc: c_int,                     // Leading dimension of matrix C
    );

    /// Scales a triangular band matrix, then multiplies it by a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (single-precision).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Valid values are 'U' (upper) or 'L' (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T', 'C').
    /// - `diag`: Specifies whether the matrix is unit triangular ('U') or not ('N').
    /// - `n`: The order of matrix `A`.
    /// - `k`: The half-bandwidth of matrix `A`.
    /// - `a`: A pointer to the triangular band matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of the array containing matrix `A`. It must be at least `k + 1`.
    /// - `x`: A pointer to the vector `x`, which is modified on return.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A` and `x` are valid and within bounds.
    ///
    /// # Discussion
    /// This function computes the matrix-vector multiplication `A * x` for a triangular band matrix `A` and stores the result in vector `x`.
    #[link_name = "cblas_ctbmv"]
    pub fn tri_band_mat_vec_mul(
        major: RowColMajor,         // RowColMajor
        tri: UpOrLowTriangle,       // UpOrLowTriangle
        trans_a: TransposeMode,     // TransposeMode
        diag: IsDiagUnit,           // IsDiagUnit
        n: c_int,                   // Order of matrix A
        k: c_int,                   // Half-bandwidth of matrix A
        a: *const Complex<c_float>, // Pointer to complex matrix A
        lda: c_int,                 // Leading dimension of matrix A
        x: *mut Complex<c_float>,   // Pointer to vector x (input/output)
        inc_x: c_int,               // Stride within vector x
    );

    /// Solves a triangular banded system of equations, either `A * X = B` or `A^T * X = B`
    /// (or `A^H * X = B` for conjugate transpose), depending on the value of `trans_a`.
    ///
    /// # Precision
    /// This function operates on `f32` complex numbers (single-precision).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Valid values are 'U' (upper) or 'L' (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is ('N'), its transpose ('T'), or its conjugate transpose ('C').
    /// - `diag`: Specifies whether the matrix is unit triangular ('U') or not ('N').
    /// - `n`: The order of matrix `A` (i.e., the number of rows and columns).
    /// - `k`: The number of super-diagonals or sub-diagonals of matrix `A`, depending on whether the upper ('U') or lower ('L') triangle is used.
    /// - `a`: A pointer to the triangular band matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of matrix `A`, must be at least `k + 1`.
    /// - `x`: On entry, contains the vector `B`. On return, this vector is overwritten with the solution vector `X`.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid and within bounds.
    ///
    /// # Discussion
    /// Solves the system of linear equations `A * X = B` or `A^T * X = B`, depending on the value of `trans_a`.
    #[link_name = "cblas_ctbsv"]
    pub fn tri_band_solve(
        major: RowColMajor,         // RowColMajor
        tri: UpOrLowTriangle,       // UpOrLowTriangle
        trans_a: TransposeMode,     // TransposeMode
        diag: IsDiagUnit,           // IsDiagUnit
        n: c_int,                   // Order of matrix A
        k: c_int,                   // Number of super-diagonals/sub-diagonals in A
        a: *const Complex<c_float>, // Pointer to matrix A
        lda: c_int,                 // Leading dimension of matrix A
        x: *mut Complex<c_float>,   // Pointer to vector X (solution vector, modified in-place)
        inc_x: c_int,               // Stride within vector X
    );

    /// Multiplies a packed triangular matrix by a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Valid values are `'U'` (upper) or `'L'` (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether the matrix is unit triangular (`'U'`) or not (`'N'`).
    /// - `n`: The order of matrix `A` and the number of elements in vectors `x`.
    /// - `ap`: Pointer to the packed triangular matrix `A` (stored in single-precision complex numbers).
    /// - `x`: Pointer to the input/output vector `X`. On return, this vector contains the result.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `ap` and `x` are valid and within bounds.
    ///
    /// # Discussion
    /// Computes `A * x`, `A^T * x`, or `conjg(A^T) * x`, depending on the value of `trans_a`, and stores the result in `x`.
    #[link_name = "cblas_ctpmv"]
    pub fn pack_tri_mat_vec_mul(
        major: RowColMajor,          // RowColMajor
        tri: UpOrLowTriangle,        // UpOrLowTriangle
        trans_a: TransposeMode,      // TransposeMode
        diag: IsDiagUnit,            // IsDiagUnit
        n: c_int,                    // Order of matrix A and number of elements in x
        ap: *const Complex<c_float>, // Pointer to packed triangular matrix A
        x: *mut Complex<c_float>,    // Pointer to vector x (output)
        inc_x: c_int,                // Stride within vector x
    );

    /// Solves a packed triangular system of equations (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Valid values are `'U'` (upper) or `'L'` (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether the matrix is unit triangular (`'U'`) or not (`'N'`).
    /// - `n`: The order of matrix `A` and the number of elements in vectors `x`.
    /// - `ap`: Pointer to the packed triangular matrix `A` (stored as single-precision complex numbers).
    /// - `x`: On entry, contains vector `B`. On return, this vector contains the solution vector `X`.
    /// - `inc_x`: The increment (stride) between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `ap` and `x` are valid and within bounds.
    ///
    /// # Discussion
    /// Solves the system of equations `A * X = B` or `A' * X = B`, depending on the value of `trans_a`.
    /// The packed storage format is used for matrix `A`.
    #[link_name = "cblas_ctpsv"]
    pub fn pack_tri_solve(
        major: RowColMajor,      // RowColMajor
        tri: UpOrLowTriangle,    // UpOrLowTriangle
        trans_a: TransposeMode,  // TransposeMode
        diag: IsDiagUnit,        // IsDiagUnit
        n: c_int,                // Order of matrix A and number of elements in x
        ap: *const Complex<f32>, // Pointer to packed triangular matrix A
        x: *mut Complex<f32>,    // Pointer to vector x (input/output)
        inc_x: c_int,            // Stride within vector x
    );

    /// Scales a triangular matrix and multiplies it by a matrix (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `side`: Specifies whether matrix `A` appears on the left or right in the multiplication.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Valid values are `'U'` (upper) or `'L'` (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether matrix `A` is unit triangular (`'U'`) or not (`'N'`).
    /// - `m`: The number of rows in matrix `B`.
    /// - `n`: The number of columns in matrix `B`.
    /// - `alpha`: The scaling factor applied to matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of array containing matrix `A`.
    /// - `b`: Pointer to the matrix `B`. On return, contains the result of the matrix multiplication.
    /// - `ldb`: The leading dimension of array containing matrix `B`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `b` are valid and within bounds.
    ///
    /// # Discussion
    /// If `side` is `'L'`, the function computes `alpha * A * B` or `alpha * A' * B`, depending on `trans_a`.
    /// If `side` is `'R'`, it computes `alpha * B * A` or `alpha * B * A'`, depending on `trans_a`.
    /// In either case, the result is stored in matrix `B`.
    #[link_name = "cblas_ctrmm"]
    pub fn tri_mat_mul(
        major: RowColMajor,         // RowColMajor
        side: MultiplyOrder,        // MultiplyOrder
        tri: UpOrLowTriangle,       // UpOrLowTriangle
        trans_a: TransposeMode,     // TransposeMode
        diag: IsDiagUnit,           // IsDiagUnit
        m: c_int,                   // Number of rows in matrix B
        n: c_int,                   // Number of columns in matrix B
        alpha: *const Complex<f32>, // Scaling factor for matrix A
        a: *const Complex<f32>,     // Pointer to triangular matrix A
        lda: c_int,                 // Leading dimension of matrix A
        b: *mut Complex<f32>,       // Pointer to matrix B (output)
        ldb: c_int,                 // Leading dimension of matrix B
    );

    /// Multiplies a triangular matrix by a vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Valid values are `'U'` (upper) or `'L'` (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether matrix `A` is unit triangular (`'U'`) or not (`'N'`).
    /// - `n`: The order of matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of array containing matrix `A`.
    /// - `x`: Pointer to the vector `X`, modified in place on return.
    /// - `inc_x`: The increment (stride) between elements in `X`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid and within bounds.
    ///
    /// # Discussion
    /// Computes `A * X` or `A' * X`, depending on the value of `trans_a`. The result is stored in the input vector `X`.
    #[link_name = "cblas_ctrmv"]
    pub fn tri_mat_vec_mul(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        a: *const Complex<f32>, // Pointer to triangular matrix A
        lda: c_int,             // Leading dimension of matrix A
        x: *mut Complex<f32>,   // Pointer to vector X (input/output)
        inc_x: c_int,           // Increment between elements in X
    );

    /// Solves a triangular system of equations with multiple right-hand side vectors (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `side`: Determines whether matrix `A` is on the left or right side in the system of equations. Use `MultiplyOrder`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Valid values are `'U'` (upper) or `'L'` (lower).
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether matrix `A` is unit triangular (`'U'`) or not (`'N'`).
    /// - `m`: The number of rows in matrix `B`.
    /// - `n`: The number of columns in matrix `B`.
    /// - `alpha`: Scaling factor for matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of array containing matrix `A`.
    /// - `b`: Pointer to matrix `B` (input/output). On entry, contains matrix `B`, and on return, contains the solution matrix `X`.
    /// - `ldb`: The leading dimension of matrix `B`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `b` are valid and within bounds.
    ///
    /// # Discussion
    /// Solves the system of equations `A * X = alpha * B` (if `side` is 'L') or `X * A = alpha * B` (if `side` is 'R'), depending on the value of `trans_a`. The result is stored in matrix `B`.
    #[link_name = "cblas_ctrsm"]
    pub fn tri_solve_multiple(
        major: RowColMajor,         // RowColMajor
        side: MultiplyOrder,        // MultiplyOrder (Left or Right)
        tri: UpOrLowTriangle,       // UpOrLowTriangle
        trans_a: TransposeMode,     // TransposeMode
        diag: IsDiagUnit,           // IsDiagUnit
        m: c_int,                   // Number of rows in matrix B
        n: c_int,                   // Number of columns in matrix B
        alpha: *const Complex<f32>, // Scaling factor for matrix A
        a: *const Complex<f32>,     // Pointer to triangular matrix A
        lda: c_int,                 // Leading dimension of matrix A
        b: *mut Complex<f32>,       // Pointer to matrix B (input/output, stores the solution)
        ldb: c_int,                 // Leading dimension of matrix B
    );

    /// Solves a triangular system of equations with a single right-hand side vector (single-precision complex).
    ///
    /// # Precision
    /// This function operates on single-precision complex numbers (`Complex<f32>`).
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Use `'U'` for upper, `'L'` for lower.
    /// - `trans_a`: Specifies whether to use matrix `A` as is (`'N'`), its transpose (`'T'`), or its conjugate transpose (`'C'`).
    /// - `diag`: Specifies whether matrix `A` is unit triangular (`'U'` for unit triangular, `'N'` for non-unit).
    /// - `n`: The order of matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A` (stored as single-precision complex numbers).
    /// - `lda`: The leading dimension of matrix `A`.
    /// - `x`: Pointer to the vector `X` (input/output). On entry, contains vector `B`, and on return, contains the solution vector `X`.
    /// - `inc_x`: The stride between elements in `X`. For example, if `inc_x = 7`, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid.
    ///
    /// # Discussion
    /// Solves the system of equations `A * x = b` or `A' * x = b`, where `A` is a triangular matrix and `b` is a vector. The result is stored in vector `X`.
    #[link_name = "cblas_ctrsv"]
    pub fn tri_solve(
        major: RowColMajor,         // RowColMajor
        tri: UpOrLowTriangle,       // UpOrLowTriangle
        trans_a: TransposeMode,     // TransposeMode
        diag: IsDiagUnit,           // IsDiagUnit
        n: c_int,                   // Order of matrix A
        a: *const Complex<c_float>, // Pointer to triangular matrix A
        lda: c_int,                 // Leading dimension of matrix A
        x: *mut Complex<c_float>,   // Pointer to vector X (input/output)
        inc_x: c_int,               // Stride within vector X
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
    pub fn vec_abs_sum(
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
    pub fn vec_unitary_norm(
        n: c_int,                   // Length of vector X
        x: *const Complex<c_float>, // Pointer to vector X
        inc_x: c_int,               // Stride within vector X
    ) -> c_float;

}
