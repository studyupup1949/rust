//! QR Decomposition via Householder Transformations
//!
//! QR decomposition factors a matrix A into the product of an orthogonal matrix Q
//! and an upper triangular matrix R. This decomposition is more numerically stable
//! than Gaussian elimination and is used for:
//! - Solving linear least squares problems
//! - Finding eigenvalues (via QR algorithm)
//! - Computing matrix inverses
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::numerical::qr_decomposition::QRDecomposition;
//!
//! let matrix = vec![
//!     vec![12.0, -51.0, 4.0],
//!     vec![6.0, 167.0, -68.0],
//!     vec![-4.0, 24.0, -41.0],
//! ];
//!
//! let qr = QRDecomposition::decompose(&matrix).unwrap();
//! let q = qr.q();
//! let r = qr.r();
//! ```

use crate::{AlgorithmError, Result};

/// QR Decomposition result
///
/// Contains the Q (orthogonal) and R (upper triangular) matrices
pub struct QRDecomposition {
    q: Vec<Vec<f64>>,
    r: Vec<Vec<f64>>,
}

impl QRDecomposition {
    /// Performs QR decomposition using Householder transformations
    ///
    /// # Arguments
    ///
    /// * `matrix` - An m×n matrix where m >= n
    ///
    /// # Returns
    ///
    /// A QRDecomposition containing Q and R matrices
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Matrix is empty
    /// - Matrix rows have inconsistent lengths
    /// - m < n (more columns than rows)
    pub fn decompose(matrix: &[Vec<f64>]) -> Result<Self> {
        if matrix.is_empty() || matrix[0].is_empty() {
            return Err(AlgorithmError::InvalidInput(
                "Matrix cannot be empty".to_string()
            ));
        }
        
        let m = matrix.len();
        let n = matrix[0].len();
        
        // Validate matrix dimensions
        for row in matrix.iter() {
            if row.len() != n {
                return Err(AlgorithmError::InvalidInput(
                    "All rows must have the same length".to_string()
                ));
            }
        }
        
        if m < n {
            return Err(AlgorithmError::InvalidInput(
                "Matrix must have at least as many rows as columns".to_string()
            ));
        }
        
        // Initialize R with a copy of the input matrix
        let mut r = matrix.to_vec();
        
        // Initialize Q as identity matrix
        let mut q = identity_matrix(m);
        
        // Perform Householder transformations
        for k in 0..n.min(m - 1) {
            let h = householder_matrix(&r, k, m)?;
            r = matrix_multiply(&h, &r)?;
            q = matrix_multiply(&h, &q)?;
        }
        
        // Transpose Q to get the final result
        q = transpose(&q);
        
        Ok(QRDecomposition { q, r })
    }
    
    /// Returns the Q matrix (orthogonal matrix)
    pub fn q(&self) -> &[Vec<f64>] {
        &self.q
    }
    
    /// Returns the R matrix (upper triangular matrix)
    pub fn r(&self) -> &[Vec<f64>] {
        &self.r
    }
    
    /// Solves the system Ax = b using QR decomposition
    ///
    /// # Arguments
    ///
    /// * `b` - Right-hand side vector
    ///
    /// # Returns
    ///
    /// Solution vector x
    pub fn solve(&self, b: &[f64]) -> Result<Vec<f64>> {
        if b.len() != self.q.len() {
            return Err(AlgorithmError::DimensionMismatch(
                format!("Vector length {} doesn't match matrix dimension {}", 
                        b.len(), self.q.len())
            ));
        }
        
        // Compute Q^T * b
        let qt_b = matrix_vector_multiply(&transpose(&self.q), b)?;
        
        // Solve Rx = Q^T*b by back substitution
        back_substitution(&self.r, &qt_b)
    }
}

/// Creates a Householder matrix for the k-th column
fn householder_matrix(a: &[Vec<f64>], k: usize, m: usize) -> Result<Vec<Vec<f64>>> {
    let _n = a[0].len();
    
    // Extract the k-th column from row k onwards
    let mut x: Vec<f64> = (k..m).map(|i| a[i][k]).collect();
    
    // Compute the norm of x
    let norm_x: f64 = x.iter().map(|&val| val * val).sum::<f64>().sqrt();
    
    if norm_x < 1e-10 {
        // Column is already zeros below diagonal
        return Ok(identity_matrix(m));
    }
    
    // Compute the Householder vector
    x[0] += if x[0] >= 0.0 { norm_x } else { -norm_x };
    let norm_v: f64 = x.iter().map(|&val| val * val).sum::<f64>().sqrt();
    
    if norm_v < 1e-10 {
        return Ok(identity_matrix(m));
    }
    
    // Normalize
    for val in x.iter_mut() {
        *val /= norm_v;
    }
    
    // Construct H = I - 2vv^T
    let mut h = identity_matrix(m);
    
    for i in 0..(m - k) {
        for j in 0..(m - k) {
            h[i + k][j + k] -= 2.0 * x[i] * x[j];
        }
    }
    
    Ok(h)
}

/// Creates an identity matrix of size n×n
fn identity_matrix(n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    matrix
}

/// Multiplies two matrices
fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
    let m = a.len();
    let n = b[0].len();
    let p = a[0].len();
    
    if p != b.len() {
        return Err(AlgorithmError::DimensionMismatch(
            "Matrix dimensions incompatible for multiplication".to_string()
        ));
    }
    
    let mut result = vec![vec![0.0; n]; m];
    
    for i in 0..m {
        for j in 0..n {
            for k in 0..p {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    
    Ok(result)
}

/// Multiplies a matrix by a vector
fn matrix_vector_multiply(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
    let m = a.len();
    let n = a[0].len();
    
    if n != b.len() {
        return Err(AlgorithmError::DimensionMismatch(
            "Matrix and vector dimensions incompatible".to_string()
        ));
    }
    
    let mut result = vec![0.0; m];
    
    for i in 0..m {
        for j in 0..n {
            result[i] += a[i][j] * b[j];
        }
    }
    
    Ok(result)
}

/// Transposes a matrix
fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let m = matrix.len();
    let n = matrix[0].len();
    
    let mut result = vec![vec![0.0; m]; n];
    
    for i in 0..m {
        for j in 0..n {
            result[j][i] = matrix[i][j];
        }
    }
    
    result
}

/// Solves an upper triangular system using back substitution
fn back_substitution(r: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>> {
    let n = r[0].len();
    let mut x = vec![0.0; n];
    
    for i in (0..n).rev() {
        if r[i][i].abs() < 1e-10 {
            return Err(AlgorithmError::NumericalInstability(
                "Matrix is singular or nearly singular".to_string()
            ));
        }
        
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= r[i][j] * x[j];
        }
        x[i] = sum / r[i][i];
    }
    
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qr_decomposition() {
        let matrix = vec![
            vec![12.0, -51.0, 4.0],
            vec![6.0, 167.0, -68.0],
            vec![-4.0, 24.0, -41.0],
        ];
        
        let qr = QRDecomposition::decompose(&matrix).unwrap();
        let q = qr.q();
        let r = qr.r();
        
        // Verify Q is orthogonal (Q^T * Q = I)
        let qt = transpose(q);
        let qtq = matrix_multiply(&qt, q).unwrap();
        
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((qtq[i][j] - expected).abs() < 1e-10);
            }
        }
        
        // Verify R is upper triangular
        for i in 0..3 {
            for j in 0..i {
                assert!(r[i][j].abs() < 1e-10);
            }
        }
    }
    
    #[test]
    fn test_solve() {
        let matrix = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
        ];
        
        let b = vec![1.0, 2.0, 3.0];
        
        let qr = QRDecomposition::decompose(&matrix).unwrap();
        let x = qr.solve(&b).unwrap();
        
        // Solution should minimize ||Ax - b||
        assert_eq!(x.len(), 2);
    }
}
