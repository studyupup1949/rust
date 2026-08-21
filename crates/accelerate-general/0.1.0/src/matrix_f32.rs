use crate::constants::{IsDiagUnit, MultiplyOrder, RowColMajor, TransposeMode, UpOrLowTriangle};
use std::ffi::{c_float, c_int};

#[link(name = "Accelerate", kind = "framework")]
extern "C" {
    /// Performs a matrix-vector operation using a general band matrix, with the form:
    /// `y = alpha * A * x + beta * y`, where `A` is a general band matrix.
    ///
    /// # Precision
    /// This function operates on single-precision `f32` numbers.
    ///
    /// # Parameters
    /// - `major`: The memory layout of the matrix `A`. Use `RowColMajor` to specify row-major or column-major order.
    /// - `trans_a`: Specifies whether to transpose matrix `A`. Use `TransposeMode` for no transpose, transpose, or conjugate transpose.
    /// - `m`: The number of rows in the matrix `A`.
    /// - `n`: The number of columns in the matrix `A`.
    /// - `kl`: The number of sub-diagonals (below the main diagonal) in the matrix `A`.
    /// - `ku`: The number of super-diagonals (above the main diagonal) in the matrix `A`.
    /// - `alpha`: The scalar factor applied to matrix `A`.
    /// - `a`: A pointer to the band matrix `A`, stored in column-major order. The matrix is stored as an array, where each column stores the elements of the band, including the main diagonal, sub-diagonals, and super-diagonals. The unused entries outside the band are not stored.
    /// - `lda`: The leading dimension of `A`, which is the number of rows in the band storage array. It must be at least `kl + ku + 1`.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in `x`. `inc_x` must ensure that accessing `x` up to `n * inc_x` is valid.
    /// - `beta`: The scalar factor applied to the vector `y`.
    /// - `y`: A pointer to the input/output vector `y`, which stores the result in-place.
    /// - `inc_y`: The increment between elements in `y`. `inc_y` must ensure that accessing `y` up to `m * inc_y` is valid.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that:
    /// - The band matrix `A` is stored in column-major order, with dimensions and increments that ensure the validity of memory accesses up to `m * n` elements.
    /// - The vectors `x` and `y` must be valid for reads/writes up to `n * inc_x` and `m * inc_y` elements respectively.
    #[link_name = "cblas_sgbmv"]
    pub fn band_mat_mul_vec(
        major: RowColMajor,
        trans: TransposeMode,
        m: c_int,
        n: c_int,
        kl: c_int,
        ku: c_int,
        alpha: c_float,
        a: *const c_float,
        lda: c_int,
        x: *const c_float,
        inc_x: c_int,
        beta: c_float,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Performs a general matrix-matrix multiplication (GEMM) operation of the form:
    /// `C = alpha * A * B + beta * C`.
    ///
    /// # Precision
    /// This function operates on single-precision floating point numbers (`f32`).
    ///
    /// # Parameters
    /// - `major`: Specifies the memory layout of the matrices. Use `RowColMajor` to specify row-major (C) or column-major (Fortran) order.
    /// - `trans_a`: Specifies whether to transpose matrix `A`. Use `TransposeMode` for options such as no transpose, transpose, or conjugate transpose.
    /// - `trans_b`: Specifies whether to transpose matrix `B`.
    /// - `m`: The number of rows in matrices `A` and `C`.
    /// - `n`: The number of columns in matrices `B` and `C`.
    /// - `k`: The number of columns in matrix `A` and the number of rows in matrix `B`.
    /// - `alpha`: The scaling factor applied to the product of matrices `A` and `B`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. This is typically the number of rows in matrix `A` (if column-major) or the number of columns (if row-major).
    /// - `b`: A pointer to matrix `B`.
    /// - `ldb`: The leading dimension of matrix `B`. This is typically the number of rows in matrix `B` (if column-major) or the number of columns (if row-major).
    /// - `beta`: The scaling factor applied to matrix `C`.
    /// - `c`: A pointer to matrix `C` where the result will be stored.
    /// - `ldc`: The leading dimension of matrix `C`. This is typically the number of rows in matrix `C` (if column-major) or the number of columns (if row-major).
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the matrices `A`, `B`, and `C` have valid memory regions and that the dimensions provided
    /// (i.e., `m`, `n`, `k`, `lda`, `ldb`, and `ldc`) match the actual matrix storage.
    #[link_name = "cblas_sgemm"]
    pub fn mat_mul(
        major: RowColMajor,
        trans_a: TransposeMode,
        trans_b: TransposeMode,
        m: c_int,
        n: c_int,
        k: c_int,
        alpha: c_float,
        a: *const c_float,
        lda: c_int,
        b: *const c_float,
        ldb: c_int,
        beta: c_float,
        c: *mut c_float,
        ldc: c_int,
    );

    /// Performs a matrix-vector operation using a general matrix, either:
    /// `y = alpha * A * x + beta * y` or `y = alpha * A^T * x + beta * y`, depending on the transpose mode.
    ///
    /// # Precision
    /// This function operates on single-precision floating point numbers (`f32`).
    ///
    /// # Parameters
    /// - `major`: Specifies the memory layout of matrix `A`. Use `RowColMajor` to specify row-major (C) or column-major (Fortran) order.
    /// - `trans_a`: Specifies whether to transpose matrix `A`. Use `TransposeMode` to specify no transpose, transpose, or conjugate transpose.
    /// - `m`: The number of rows in matrix `A`.
    /// - `n`: The number of columns in matrix `A`.
    /// - `alpha`: The scalar factor applied to the product of matrix `A` and vector `X`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. If column-major order is used, `lda` should be equal to `m`; if row-major, `lda` should be equal to `n`.
    /// - `x`: A pointer to the input vector `X`.
    /// - `inc_x`: The increment (stride) between elements in `x`.
    /// - `beta`: The scalar factor applied to vector `Y`.
    /// - `y`: A pointer to the output vector `Y`, which stores the result in-place.
    /// - `inc_y`: The increment (stride) between elements in `y`.
    ///
    /// # Safety
    /// This is an `unsafe` function. The caller must ensure that the memory regions accessed by `A`, `X`, and `Y` are valid.
    #[link_name = "cblas_sgemv"]
    pub fn mat_vec_mul(
        major: RowColMajor,
        trans_a: TransposeMode,
        m: c_int,
        n: c_int,
        alpha: c_float,
        a: *const c_float,
        lda: c_int,
        x: *const c_float,
        inc_x: c_int,
        beta: c_float,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Performs a rank-1 update of a matrix: `A = alpha * x * y' + A`.
    ///
    /// # Precision
    /// This function operates on single-precision floating point numbers (`f32`).
    ///
    /// # Parameters
    /// - `major`: Specifies the memory layout of matrix `A` (row-major or column-major).
    /// - `m`: The number of rows in matrix `A`.
    /// - `n`: The number of columns in matrix `A`.
    /// - `alpha`: The scalar factor applied to the outer product of vectors `x` and `y`.
    /// - `x`: A pointer to vector `x`.
    /// - `inc_x`: The increment (stride) between elements in `x`.
    /// - `y`: A pointer to vector `y`.
    /// - `inc_y`: The increment (stride) between elements in `y`.
    /// - `a`: A pointer to matrix `A`, which will be updated in-place.
    /// - `lda`: The leading dimension of matrix `A`. For column-major order, this is usually `m`; for row-major order, this is usually `n`.
    ///
    /// # Safety
    /// This function is unsafe as it involves raw pointers. The caller must ensure that the memory regions accessed by `x`, `y`, and `A` (according to their dimensions and strides) are valid.
    #[link_name = "cblas_sger"]
    pub fn mat_rank1_update(
        major: RowColMajor,
        m: c_int,
        n: c_int,
        alpha: c_float,
        x: *const c_float,
        inc_x: c_int,
        y: *const c_float,
        inc_y: c_int,
        a: *mut c_float,
        lda: c_int,
    );

    /// Scales a symmetric band matrix, then multiplies by a vector, and adds another vector.
    ///
    /// Computes the operation `y = alpha * A * x + beta * y`, where:
    /// - `A` is a symmetric band matrix,
    /// - `x` is a vector, and
    /// - `y` is a vector that will store the result.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies whether the matrix is stored in row-major or column-major order.
    /// - `tri`: Specifies whether the upper ('U') or lower ('L') triangle of the matrix is used.
    /// - `n`: The order (number of rows/columns) of the matrix `A`.
    /// - `k`: The number of sub-diagonals/super-diagonals in the band matrix `A`.
    /// - `alpha`: Scaling factor for matrix `A`.
    /// - `a`: Pointer to the band matrix `A`.
    /// - `lda`: Leading dimension of `A`, must be at least `k + 1`.
    /// - `x`: Pointer to the input vector `x`.
    /// - `inc_x`: Stride within `x`. For example, if `inc_x` is 7, every 7th element is used.
    /// - `beta`: Scaling factor for vector `y`.
    /// - `y`: Pointer to the input/output vector `y`. Replaced by the results on return.
    /// - `inc_y`: Stride within `y`. For example, if `inc_y` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `a`, `x`, and `y` are valid, and the matrix dimensions and strides are correct.
    #[link_name = "cblas_ssbmv"]
    pub fn sym_band_mat_vec_mul(
        major: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        k: c_int,
        alpha: c_float,
        a: *const c_float,
        lda: c_int,
        x: *const c_float,
        inc_x: c_int,
        beta: c_float,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Performs the matrix-vector operation using a packed symmetric matrix:
    /// `y = alpha * A * x + beta * y`, where `A` is a packed symmetric matrix.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies whether the matrix is in row-major or column-major order.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix.
    /// - `n`: The order of the matrix `A`, and the number of elements in the vectors `x` and `y`.
    /// - `alpha`: The scaling factor applied to the matrix `A`.
    /// - `ap`: A pointer to the packed symmetric matrix `A` (stored in packed format).
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in the vector `x`.
    /// - `beta`: The scaling factor applied to the vector `y`.
    /// - `y`: A pointer to the input/output vector `y`. The result is stored in place.
    /// - `inc_y`: The increment between elements in the vector `y`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `ap`, `x`, and `y` are valid and that accessing `ap`, `x`, and `y` up to `n * inc_x` and `n * inc_y` is safe.
    #[link_name = "cblas_sspmv"]
    pub fn pack_sym_mat_vec_mul(
        order: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        alpha: c_float,
        ap: *const c_float,
        x: *const c_float,
        inc_x: c_int,
        beta: c_float,
        y: *mut c_float,
        inc_y: c_int,
    );

    /// Performs a rank-one update on a packed symmetric matrix, adding the product of a vector `x`
    /// and its transpose, scaled by `alpha`:
    /// `A = A + alpha * x * x^T`, where `A` is a packed symmetric matrix.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies whether the matrix `A` is in row-major or column-major order.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix.
    /// - `n`: The order of the matrix `A` and the number of elements in vector `x`.
    /// - `alpha`: The scaling factor applied to the outer product of vector `x`.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The increment between elements in vector `x`.
    /// - `ap`: A pointer to the packed symmetric matrix `A` (stored in packed format). The result is stored in place.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `x` and `ap` are valid and that accessing `x` and `ap` up to `n * inc_x` is safe.
    #[link_name = "cblas_sspr"]
    pub fn pack_sym_rank1_update(
        major: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        alpha: c_float,
        x: *const c_float,
        inc_x: c_int,
        ap: *mut c_float,
    );

    /// Performs a rank-two update on a packed symmetric matrix, adding the product of two vectors `x` and `y`,
    /// scaled by `alpha`:
    /// `A = A + alpha * (x * y^T + y * x^T)`, where `A` is a packed symmetric matrix.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies whether the matrix `A` is in row-major or column-major order.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix.
    /// - `n`: The order of the matrix `A` and the number of elements in vectors `x` and `y`.
    /// - `alpha`: The scaling factor applied to the outer product of vectors `x` and `y`.
    /// - `x`: A pointer to the first input vector `x`.
    /// - `inc_x`: The increment between elements in vector `x`.
    /// - `y`: A pointer to the second input vector `y`.
    /// - `inc_y`: The increment between elements in vector `y`.
    /// - `a`: A pointer to the packed symmetric matrix `A` (stored in packed format). The result is stored in place.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the pointers passed to `x`, `y`, and `a` are valid and that accessing `x`, `y`, and `a` up to `n * inc_x` and `n * inc_y` is safe.
    #[link_name = "cblas_sspr2"]
    pub fn pack_sym_rank_2_update(
        major: RowColMajor,
        tri: UpOrLowTriangle,
        n: c_int,
        alpha: c_float,
        x: *const c_float,
        inc_x: c_int,
        y: *const c_float,
        inc_y: c_int,
        a: *mut c_float,
    );

    /// Multiplies a matrix by a symmetric matrix and updates the result in matrix `C` (single-precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering.
    /// - `side`: Determines the order in which the matrices should be multiplied. Use `MultiplyOrder`.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Use `UpOrLowTriangle`.
    /// - `m`: Number of rows in matrices `A` and `C`.
    /// - `n`: Number of columns in matrices `B` and `C`.
    /// - `alpha`: Scaling factor for the product of matrices `A` and `B`.
    /// - `a`: A pointer to the symmetric matrix `A`.
    /// - `lda`: The leading dimension of matrix `A` (should be `m` for column-major or `n` for row-major).
    /// - `b`: A pointer to the matrix `B`.
    /// - `ldb`: The leading dimension of matrix `B` (should be `m` for column-major or `n` for row-major).
    /// - `beta`: Scaling factor for matrix `C`.
    /// - `c`: A pointer to the matrix `C` (result is stored here).
    /// - `ldc`: The leading dimension of matrix `C` (should be `m` for column-major or `n` for row-major).
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A`, `B`, and `C` are valid and within bounds.
    #[link_name = "cblas_ssymm"]
    pub fn sym_mat_mul(
        major: RowColMajor,
        side: MultiplyOrder,
        tri: UpOrLowTriangle,
        m: c_int,
        n: c_int,
        alpha: c_float,
        a: *const c_float,
        lda: c_int,
        b: *const c_float,
        ldb: c_int,
        beta: c_float,
        c: *mut c_float,
        ldc: c_int,
    );

    /// Multiplies a symmetric matrix by a vector, then scales and adds another vector (single-precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `n`: The order of matrix `A` and the length of vectors `x` and `y`.
    /// - `alpha`: Scaling factor for matrix `A`.
    /// - `a`: A pointer to the symmetric matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. This value should be at least the order of the matrix.
    /// - `x`: A pointer to the vector `x`.
    /// - `inc_x`: The stride between elements in `x`. For example, if `inc_x = 7`, every 7th element of `x` is used.
    /// - `beta`: Scaling factor for vector `y`.
    /// - `y`: A pointer to the vector `y`, which contains the results on return.
    /// - `inc_y`: The stride between elements in `y`. For example, if `inc_y = 7`, every 7th element of `y` is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A`, `x`, and `y` are valid and within bounds.
    #[link_name = "cblas_ssymv"]
    pub fn sym_mat_vec_mul(
        major: RowColMajor,   // RowColMajor
        tri: UpOrLowTriangle, // UpOrLowTriangle
        n: c_int,             // Order of matrix A and length of vectors x and y
        alpha: c_float,       // Scaling factor for matrix A
        a: *const c_float,    // Pointer to matrix A
        lda: c_int,           // Leading dimension of matrix A
        x: *const c_float,    // Pointer to vector x
        inc_x: c_int,         // Stride within x
        beta: c_float,        // Scaling factor for vector y
        y: *mut c_float,      // Pointer to vector y
        inc_y: c_int,         // Stride within y
    );

    /// Rank one update: adds a symmetric matrix to the product of a scaling factor, a vector, and its transpose (single precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `n`: The order of matrix `A` and the number of elements in vector `x`.
    /// - `alpha`: The scaling factor to multiply vector `x` by.
    /// - `x`: A pointer to the input vector `x`.
    /// - `inc_x`: The stride between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `a`: A pointer to the matrix `A`, which is symmetric.
    /// - `lda`: The leading dimension of matrix `A`. It should be at least `n`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x` and `A` are valid and within bounds.
    #[link_name = "cblas_ssyr"]
    pub fn sym_rank_1_update(
        major: RowColMajor,   // RowColMajor
        tri: UpOrLowTriangle, // UpOrLowTriangle
        n: c_int,             // Order of matrix A and the number of elements in vector x
        alpha: c_float,       // Scaling factor to multiply x by
        x: *const c_float,    // Pointer to vector x
        inc_x: c_int,         // Stride within x
        a: *mut c_float,      // Pointer to matrix A
        lda: c_int,           // Leading dimension of matrix A
    );

    /// Rank two update of a symmetric matrix using two vectors (single precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `n`: The order of matrix `A` and the number of elements in vectors `x` and `y`.
    /// - `alpha`: The scaling factor to multiply vectors `x` and `y` by.
    /// - `x`: A pointer to the first input vector `x`.
    /// - `inc_x`: The stride between elements in `x`. For example, if `inc_x = 7`, every 7th element is used.
    /// - `y`: A pointer to the second input vector `y`.
    /// - `inc_y`: The stride between elements in `y`. For example, if `inc_y = 7`, every 7th element is used.
    /// - `a`: A pointer to the symmetric matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. It should be at least `n`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `x`, `y`, and `A` are valid and within bounds.
    #[link_name = "cblas_ssyr2"]
    pub fn sym_rank_2_update(
        major: RowColMajor,   // RowColMajor
        tri: UpOrLowTriangle, // UpOrLowTriangle
        n: c_int,             // Order of matrix A and the number of elements in vectors x and y
        alpha: c_float,       // Scaling factor to multiply x and y
        x: *const c_float,    // Pointer to vector x
        inc_x: c_int,         // Stride within x
        y: *const c_float,    // Pointer to vector y
        inc_y: c_int,         // Stride within y
        a: *mut c_float,      // Pointer to matrix A
        lda: c_int,           // Leading dimension of matrix A
    );

    /// Performs a rank-2k update of a symmetric matrix (single precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Use `UpOrLowTriangle`.
    /// - `trans`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate of `A` ('C'). Use `TransposeMode`.
    /// - `n`: The order of matrix `C`.
    /// - `k`: Specifies the number of columns in matrices `A` and `B` if `trans = 'N'`; or the number of rows if `trans = 'T'` or `trans = 'C'`.
    /// - `alpha`: The scaling factor for matrices `A` and `B`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. It must be at least `max(1, n)` if `trans = 'N'`; otherwise, it must be at least `max(1, k)`.
    /// - `b`: A pointer to matrix `B`.
    /// - `ldb`: The leading dimension of matrix `B`. It must be at least `max(1, n)` if `trans = 'N'`; otherwise, it must be at least `max(1, k)`.
    /// - `beta`: The scaling factor for matrix `C`.
    /// - `c`: A pointer to matrix `C`.
    /// - `ldc`: The leading dimension of matrix `C`. It must be at least `max(1, n)`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A`, `B`, and `C` are valid and within bounds.
    #[link_name = "cblas_ssyr2k"]
    pub fn sym_rank_2k_update(
        major: RowColMajor,   // RowColMajor
        tri: UpOrLowTriangle, // UpOrLowTriangle
        trans: TransposeMode, // TransposeMode
        n: c_int,             // Order of matrix C
        k: c_int,             // Number of columns of A, B (or rows if transposed)
        alpha: c_float,       // Scaling factor for A and B
        a: *const c_float,    // Pointer to matrix A
        lda: c_int,           // Leading dimension of matrix A
        b: *const c_float,    // Pointer to matrix B
        ldb: c_int,           // Leading dimension of matrix B
        beta: c_float,        // Scaling factor for matrix C
        c: *mut c_float,      // Pointer to matrix C
        ldc: c_int,           // Leading dimension of matrix C
    );

    /// Performs a rank-k update of a symmetric matrix (single precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle from the matrix. Use `UpOrLowTriangle`.
    /// - `trans`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T', 't', 'C', 'c'). Use `TransposeMode`.
    /// - `n`: The order of matrix `C`.
    /// - `k`: The number of columns in matrix `A` (or number of rows if matrix `A` is transposed).
    /// - `alpha`: The scaling factor for matrix `A`.
    /// - `a`: A pointer to matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. It must be at least `max(1, n)` if `trans = 'N'`; otherwise, it must be at least `max(1, k)`.
    /// - `beta`: The scaling factor for matrix `C`.
    /// - `c`: A pointer to matrix `C`.
    /// - `ldc`: The leading dimension of matrix `C`. It must be at least `max(1, n)`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A` and `C` are valid and within bounds.
    #[link_name = "cblas_ssyrk"]
    pub fn sym_rank_k_update(
        major: RowColMajor,   // RowColMajor
        tri: UpOrLowTriangle, // UpOrLowTriangle
        trans: TransposeMode, // TransposeMode
        n: c_int,             // Order of matrix C
        k: c_int,             // Number of columns of A (or rows if transposed)
        alpha: c_float,       // Scaling factor for A
        a: *const c_float,    // Pointer to matrix A
        lda: c_int,           // Leading dimension of matrix A
        beta: c_float,        // Scaling factor for matrix C
        c: *mut c_float,      // Pointer to matrix C
        ldc: c_int,           // Leading dimension of matrix C
    );

    /// Scales a triangular band matrix, then multiplies it by a vector (single precision).
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T', 't', 'C', or 'c'). Use `TransposeMode`.
    /// - `diag`: Specifies whether the matrix is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `n`: The order of the matrix `A`.
    /// - `k`: The half-bandwidth of the matrix `A`.
    /// - `a`: A pointer to the triangular matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`. Must be at least `max(1, n)`.
    /// - `x`: A pointer to the vector `x`, which is modified in place to store the result.
    /// - `inc_x`: The stride within `x`. For example, if `inc_x` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `A` and `x` are valid and within bounds.
    #[link_name = "cblas_stbmv"]
    pub fn tri_band_mat_vec_mul(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        k: c_int,               // Half-bandwidth of matrix A
        a: *const c_float,      // Pointer to matrix A
        lda: c_int,             // Leading dimension of matrix A
        x: *mut c_float,        // Pointer to vector x
        inc_x: c_int,           // Stride within x
    );

    /// Solves a triangular banded system of equations, either `A * X = B` or `A^T * X = B`, depending on the value of `transa`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or its transpose ('T', 'C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether the matrix is unit triangular (`U`) or not (`N`). Use `IsDiagUnit`.
    /// - `n`: The order of the matrix `A` (number of rows and columns).
    /// - `k`: The number of superdiagonals or subdiagonals of matrix `A`, depending on the value of `uplo`.
    /// - `a`: A pointer to the triangular band matrix `A`.
    /// - `lda`: The leading dimension of matrix `A`, must be at least `k + 1`.
    /// - `x`: On entry, contains vector `B`. On return, overwritten with vector `X` (solution).
    /// - `inc_x`: The stride within `x`. If `inc_x` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid and within bounds.
    #[link_name = "cblas_stbsv"]
    pub fn tri_band_solve(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        k: c_int,               // Number of super/subdiagonals
        a: *const c_float,      // Pointer to matrix A
        lda: c_int,             // Leading dimension of A
        x: *mut c_float,        // Pointer to vector X (input/output)
        inc_x: c_int,           // Stride within X
    );

    /// Multiplies a triangular matrix `A` by a vector `X` and stores the result in `X`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate transpose of `A` ('C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether the matrix is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `n`: The order of the matrix `A` and the number of elements in vectors `X` and `Y`.
    /// - `ap`: A pointer to the triangular matrix `A` in packed storage format.
    /// - `x`: A pointer to the input/output vector `X`. On return, it contains the result of the matrix-vector multiplication.
    /// - `inc_x`: The increment between elements in `X`. If `inc_x` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `ap` and `x` are valid and within bounds.
    #[link_name = "cblas_stpmv"]
    pub fn pack_tri_mat_vec_mul(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        ap: *const c_float,     // Pointer to packed triangular matrix A
        x: *mut c_float,        // Pointer to vector X (input/output)
        inc_x: c_int,           // Stride within X
    );

    /// Solves a system of linear equations `A * X = B` or `A' * X = B` where `A` is a packed triangular matrix.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of the matrix `A`. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate transpose of `A` ('C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether the matrix is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `n`: The order of matrix `A` (i.e., the number of rows and columns).
    /// - `ap`: A pointer to the triangular matrix `A` in packed storage format.
    /// - `x`: A pointer to the input vector `B` (on entry) and the solution vector `X` (on return).
    /// - `inc_x`: The increment between elements in `X`. If `inc_x` is 7, every 7th element is used.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `ap` and `x` are valid and within bounds.
    #[link_name = "cblas_stpsv"]
    pub fn pack_tri_solve(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        ap: *const c_float,     // Pointer to packed triangular matrix A
        x: *mut c_float,        // Pointer to vector X (input/output)
        inc_x: c_int,           // Stride within X
    );

    /// Scales a triangular matrix `A` and multiplies it by another matrix `B`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `side`: Specifies whether matrix `A` appears on the left or right of matrix `B`. Use `MultiplyOrder`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N'), the transpose of `A` ('T'), or the conjugate transpose of `A`. Use `TransposeMode`.
    /// - `diag`: Specifies whether matrix `A` is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `m`: Number of rows in matrix `B`.
    /// - `n`: Number of columns in matrix `B`.
    /// - `alpha`: Scaling factor for matrix `A`.
    /// - `a`: A pointer to the triangular matrix `A`.
    /// - `lda`: Leading dimension of matrix `A`.
    /// - `b`: A pointer to the matrix `B` (overwritten by results on return).
    /// - `ldb`: Leading dimension of matrix `B`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `b` are valid and within bounds.
    #[link_name = "cblas_strmm"]
    pub fn tri_mat_mul(
        major: RowColMajor,     // RowColMajor
        side: MultiplyOrder,    // MultiplyOrder (Left or Right)
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        m: c_int,               // Number of rows in matrix B
        n: c_int,               // Number of columns in matrix B
        alpha: c_float,         // Scaling factor for matrix A
        a: *const c_float,      // Pointer to triangular matrix A
        lda: c_int,             // Leading dimension of matrix A
        b: *mut c_float,        // Pointer to matrix B (output)
        ldb: c_int,             // Leading dimension of matrix B
    );

    /// Multiplies a triangular matrix `A` by a vector `X`.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T' or 'C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether matrix `A` is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `n`: Order of matrix `A`.
    /// - `a`: A pointer to the triangular matrix `A`.
    /// - `lda`: Leading dimension of matrix `A`.
    /// - `x`: A pointer to the vector `X` (overwritten by results on return).
    /// - `inc_x`: The increment between elements in `X`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid and within bounds.
    #[link_name = "cblas_strmv"]
    pub fn tri_mat_vec_mul(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        a: *const c_float,      // Pointer to triangular matrix A
        lda: c_int,             // Leading dimension of matrix A
        x: *mut c_float,        // Pointer to vector X (output)
        inc_x: c_int,           // Increment between elements in X
    );

    /// Solves a triangular system of equations with multiple right-hand sides.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `side`: Determines whether matrix `A` appears on the left or right side in the matrix equation. Use `MultiplyOrder`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T' or 'C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether matrix `A` is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `m`: Number of rows in matrix `B`.
    /// - `n`: Number of columns in matrix `B`.
    /// - `alpha`: Scalar applied to matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A`.
    /// - `lda`: Leading dimension of matrix `A`.
    /// - `b`: Pointer to matrix `B`, overwritten with the solution matrix `X`.
    /// - `ldb`: Leading dimension of matrix `B`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `b` are valid and within bounds.
    #[link_name = "cblas_strsm"]
    pub fn tri_solve_multiple(
        major: RowColMajor,     // RowColMajor
        side: MultiplyOrder,    // MultiplyOrder
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        m: c_int,               // Number of rows in matrix B
        n: c_int,               // Number of columns in matrix B
        alpha: c_float,         // Scaling factor for matrix A
        a: *const c_float,      // Pointer to triangular matrix A
        lda: c_int,             // Leading dimension of matrix A
        b: *mut c_float,        // Pointer to matrix B (overwritten with solution X)
        ldb: c_int,             // Leading dimension of matrix B
    );

    /// Solves a triangular system of equations with a single right-hand side.
    ///
    /// # Precision
    /// This function operates on single-precision (`f32`) numbers.
    ///
    /// # Parameters
    /// - `major`: Specifies row-major (C) or column-major (Fortran) data ordering. Use `RowColMajor`.
    /// - `tri`: Specifies whether to use the upper or lower triangle of matrix `A`. Use `UpOrLowTriangle`.
    /// - `trans_a`: Specifies whether to use matrix `A` ('N') or the transpose of `A` ('T' or 'C'). Use `TransposeMode`.
    /// - `diag`: Specifies whether matrix `A` is unit triangular ('U') or not ('N'). Use `IsDiagUnit`.
    /// - `n`: Order of matrix `A`.
    /// - `a`: Pointer to the triangular matrix `A`.
    /// - `lda`: Leading dimension of matrix `A`.
    /// - `x`: Pointer to the vector `X`, which stores the solution `x` or `b` depending on the equation solved.
    /// - `inc_x`: The increment between elements in `x`.
    ///
    /// # Safety
    /// This is an `unsafe` C function. The caller must ensure that the memory regions accessed by `a` and `x` are valid and within bounds.
    #[link_name = "cblas_strsv"]
    pub fn tri_solve(
        major: RowColMajor,     // RowColMajor
        tri: UpOrLowTriangle,   // UpOrLowTriangle
        trans_a: TransposeMode, // TransposeMode
        diag: IsDiagUnit,       // IsDiagUnit
        n: c_int,               // Order of matrix A
        a: *const c_float,      // Pointer to triangular matrix A
        lda: c_int,             // Leading dimension of matrix A
        x: *mut c_float,        // Pointer to vector X
        inc_x: c_int,           // Increment for vector X
    );
}
