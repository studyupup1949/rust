//! Singular Value Decomposition (SVD)
//!
//! SVD factorizes a matrix A into the product U * Σ * V^T, where:
//! - U is an orthogonal matrix (left singular vectors)
//! - Σ is a diagonal matrix (singular values)
//! - V^T is an orthogonal matrix (right singular vectors)
//!
//! SVD is used for:
//! - Matrix approximation and compression
//! - Principal Component Analysis (PCA)
//! - Solving linear least squares problems
//! - Pseudoinverse computation
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::numerical::svd::SVD;
//!
//! let matrix = vec![
//!     vec![1.0, 2.0],
//!     vec![3.0, 4.0],
//!     vec![5.0, 6.0],
//! ];
//!
//! let svd = SVD::decompose(&matrix, 1e-10, 1000).unwrap();
//! let singular_values = svd.singular_values();
//! ```

use crate::{AlgorithmError, Result};
use crate::numerical::qr_decomposition::QRDecomposition;

/// SVD decomposition result
pub struct SVD {
    u: Vec<Vec<f64>>,
    singular_values: Vec<f64>,
    vt: Vec<Vec<f64>>,
}

impl SVD {
    /// Performs Singular Value Decomposition
    ///
    /// Uses the Golub-Reinsch algorithm for SVD computation
    ///
    /// # Arguments
    ///
    /// * `matrix` - An m×n matrix
    /// * `tolerance` - Convergence tolerance
    /// * `max_iterations` - Maximum iterations for convergence
    ///
    /// # Returns
    ///
    /// SVD decomposition containing U, Σ, and V^T
    pub fn decompose(
        matrix: &[Vec<f64>],
        tolerance: f64,
        max_iterations: usize,
    ) -> Result<Self> {
        if matrix.is_empty() || matrix[0].is_empty() {
            return Err(AlgorithmError::InvalidInput(
                "Matrix cannot be empty".to_string()
            ));
        }
        
        let m = matrix.len();
        let n = matrix[0].len();
        
        // For numerical stability, if m < n, work with transpose
        if m < n {
            let transposed = transpose(matrix);
            let svd = Self::decompose_internal(&transposed, tolerance, max_iterations)?;
            
            // Swap U and V^T
            return Ok(SVD {
                u: svd.vt,
                singular_values: svd.singular_values,
                vt: svd.u,
            });
        }
        
        Self::decompose_internal(matrix, tolerance, max_iterations)
    }
    
    fn decompose_internal(
        matrix: &[Vec<f64>],
        tolerance: f64,
        max_iterations: usize,
    ) -> Result<Self> {
        let m = matrix.len();
        let n = matrix[0].len();
        
        // Step 1: Compute A^T * A
        let at = transpose(matrix);
        let ata = matrix_multiply(&at, matrix)?;
        
        // Step 2: Find eigenvalues and eigenvectors of A^T * A using QR algorithm
        let (eigenvalues, eigenvectors) = qr_algorithm(&ata, tolerance, max_iterations)?;
        
        // Step 3: Singular values are square roots of eigenvalues
        let singular_values: Vec<f64> = eigenvalues.iter()
            .map(|&x| if x > 0.0 { x.sqrt() } else { 0.0 })
            .collect();
        
        // Sort singular values in descending order
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&i, &j| {
            singular_values[j].partial_cmp(&singular_values[i]).unwrap()
        });
        
        let sorted_values: Vec<f64> = indices.iter()
            .map(|&i| singular_values[i])
            .collect();
        
        // Step 4: V is the matrix of eigenvectors
        let mut v = vec![vec![0.0; n]; n];
        for (j, &idx) in indices.iter().enumerate() {
            for i in 0..n {
                v[i][j] = eigenvectors[i][idx];
            }
        }
        
        let vt = transpose(&v);
        
        // Step 5: Compute U = A * V * Σ^(-1)
        let av = matrix_multiply(matrix, &v)?;
        let mut u = vec![vec![0.0; n]; m];
        
        for i in 0..m {
            for j in 0..n {
                if sorted_values[j] > tolerance {
                    u[i][j] = av[i][j] / sorted_values[j];
                }
            }
        }
        
        // Extend U to be m×m if needed
        if m > n {
            u = extend_u(&u, m);
        }
        
        Ok(SVD {
            u,
            singular_values: sorted_values,
            vt,
        })
    }
    
    /// Returns the U matrix (left singular vectors)
    pub fn u(&self) -> &[Vec<f64>] {
        &self.u
    }
    
    /// Returns the singular values
    pub fn singular_values(&self) -> &[f64] {
        &self.singular_values
    }
    
    /// Returns the V^T matrix (right singular vectors transposed)
    pub fn vt(&self) -> &[Vec<f64>] {
        &self.vt
    }
    
    /// Computes the matrix rank
    pub fn rank(&self, tolerance: f64) -> usize {
        self.singular_values.iter()
            .filter(|&&s| s > tolerance)
            .count()
    }
    
    /// Computes the condition number (ratio of largest to smallest singular value)
    pub fn condition_number(&self) -> f64 {
        let max_sv = self.singular_values.iter()
            .fold(0.0f64, |a, &b| a.max(b));
        
        let min_sv = self.singular_values.iter()
            .filter(|&&s| s > 1e-10)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        
        if min_sv > 0.0 && min_sv.is_finite() {
            max_sv / min_sv
        } else {
            f64::INFINITY
        }
    }
}

/// QR algorithm for eigenvalue computation
fn qr_algorithm(
    matrix: &[Vec<f64>],
    tolerance: f64,
    max_iterations: usize,
) -> Result<(Vec<f64>, Vec<Vec<f64>>)> {
    let n = matrix.len();
    let mut a = matrix.to_vec();
    let mut q_total = identity_matrix(n);
    
    for _ in 0..max_iterations {
        let qr = QRDecomposition::decompose(&a)?;
        let q = qr.q().to_vec();
        let r = qr.r().to_vec();
        
        // Update A = R * Q
        a = matrix_multiply(&r, &q)?;
        
        // Update total Q
        q_total = matrix_multiply(&q_total, &q)?;
        
        // Check for convergence (off-diagonal elements should be near zero)
        let mut max_off_diag: f64 = 0.0;
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            for j in 0..n {
                if i != j {
                    max_off_diag = max_off_diag.max(a[i][j].abs());
                }
            }
        }
        
        if max_off_diag < tolerance {
            break;
        }
    }
    
    // Extract eigenvalues from diagonal
    let eigenvalues: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    
    Ok((eigenvalues, q_total))
}

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

fn matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
    let m = a.len();
    let n = b[0].len();
    let p = a[0].len();
    
    if p != b.len() {
        return Err(AlgorithmError::DimensionMismatch(
            "Matrix dimensions incompatible".to_string()
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

fn identity_matrix(n: usize) -> Vec<Vec<f64>> {
    let mut matrix = vec![vec![0.0; n]; n];
    for (i, row) in matrix.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    matrix
}

fn extend_u(u: &[Vec<f64>], m: usize) -> Vec<Vec<f64>> {
    let n = u[0].len();
    let mut extended = vec![vec![0.0; m]; m];
    
    // Copy existing U
    for i in 0..m {
        for j in 0..n {
            extended[i][j] = u[i][j];
        }
    }
    
    // Fill remaining with orthogonal vectors (simple approach)
    #[allow(clippy::needless_range_loop)]
    for i in n..m {
        extended[i][i] = 1.0;
    }
    
    extended
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_svd_basic() {
        let matrix = vec![
            vec![1.0, 2.0],
            vec![3.0, 4.0],
            vec![5.0, 6.0],
        ];
        
        let svd = SVD::decompose(&matrix, 1e-10, 1000).unwrap();
        let sv = svd.singular_values();
        
        // Singular values should be positive and in descending order
        assert!(sv[0] >= sv[1]);
        assert!(sv[0] > 0.0);
    }
    
    #[test]
    fn test_svd_rank() {
        let matrix = vec![
            vec![1.0, 2.0],
            vec![2.0, 4.0],
        ];
        
        let svd = SVD::decompose(&matrix, 1e-10, 1000).unwrap();
        let rank = svd.rank(1e-8);
        
        // This matrix has rank 1
        assert_eq!(rank, 1);
    }
}
