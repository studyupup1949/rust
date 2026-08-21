
//! ## Example
//!
//! ```rust
//! use adlo::*; 
//! use nalgebra::{DVector, DMatrix};
//!
//! fn basis_2d_test() {
//!        let mut basis_2d = vec![
//!            AdloVector::new(DVector::from_vec(vec![5, 3])),
//!            AdloVector::new(DVector::from_vec(vec![2, 2]))
//!         ];
//!        adaptive_lll(&mut basis_2d);
//!        let expected = AdloVector::new(DVector::from_vec(vec![1, -1]));
//!        assert_eq!(expected, basis_2d[basis_2d.len()-1]);
//!    } 
//!```

use nalgebra::{DVector, DMatrix};

// Used for calculating adaptive lovaz factor;
const PSI: f64 = 0.618;
const S_PSI: f64 = 1.0 - PSI;


/// The Vector struct represents a distinction in n-dimensional space.  
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct AdloVector {
    coords: DVector<i64>,
    f64_coords: DVector<f64>,
    norm_sq: i128,
}

impl AdloVector {
    /// Create a new Adaptive LLL Vector
    pub fn new(coords: DVector<i64>) -> Self {
        let f64_coords = coords.clone().cast::<f64>();
        let norm_sq = Self::calculate_norm_sq(&coords);
        AdloVector { coords, f64_coords, norm_sq }
    }
    fn calculate_norm_sq(coords: &DVector<i64>) -> i128 {
        let mut sum: i128 = 0;
        for &coord in coords.iter() {
            let coord_i128 = coord as i128;
            sum += coord_i128 * coord_i128;
        }
        sum
    }
    fn norm_squared(&self) -> i128 {
        self.norm_sq
    }
    fn norm(&self) -> f64 {
        (self.norm_squared() as f64).sqrt()
    }
    fn set_f64_coords(&mut self) {
        self.f64_coords = self.coords.clone().cast::<f64>();
    }
}

fn calculate_local_density(basis: &[AdloVector], k: usize) -> f64 {
    let n = basis.len();
    let mut count = 0;
    // TODO: Experiment with search radius
    let radius = basis[k].norm() * 1.5;

    for i in 0..n {
        if i != k {
            let dist = basis[k].norm() - basis[i].norm();
            if dist <= radius {
                count += 1;
            }
        }
    }
    // A simple density measure: number of nearby vectors
    (count as f64) / (n as f64 - 1.0)
}

fn calculate_lovasz_factor(density: f64) -> f64 {
    // Higher density -> factor closer to 1
    // TODO: Experiment with different functions to map density to the factor
    PSI + S_PSI * density.min(1.0) // Linear interpolation (example)
}

/// The LLL (Lenstra–Lenstra–Lovász) algorithm is a lattice basis reduction algorithm that finds a basis with short,
///
/// nearly orthogonal vectors for any given lattice. It is particularly useful for solving problems in number theory and cryptography,
///
/// such as finding integer relations and breaking certain types of encryption schemes.
///
/// The Gram-Schmidt process is a method used in linear algebra to transform a set of linearly independent vectors into
///
/// an orthogonal set of vectors that span the same space. This process can further be used to create an orthonormal set
///
/// of vectors by normalizing each vector in the orthogonal set to have a unit length
///
/// The lovasz_factor is calculated via `PSI + 1 - PSI * density.min(1.0)` where density is the
/// 
/// measure of nearby vectors.
pub fn adaptive_lll(basis: &mut [AdloVector]) {
    let n = basis.len();
    let mut b_star: Vec<AdloVector> = Vec::with_capacity(n);
    let mut mu: DMatrix<f64> = DMatrix::from_element(n, n, 0.0);

    // Gram-Schmidt
    for i in 0..n {
        b_star.push(basis[i].clone());
        for j in 0..i {
            mu[(i, j)] = b_star[i].f64_coords.dot(&b_star[j].f64_coords) / b_star[j].norm().powi(2);
            // Calculate new_coords_f64 using i128 for intermediate values:
            let mut new_coords_f64 = DVector::<f64>::zeros(n);
            for dim in 0..n {
                let mu_val = mu[(i, j)];
                let b_star_j_coord = b_star[j].f64_coords[dim];
                let intermediate_val = (mu_val * b_star_j_coord) as f64; // Cast to f64 to prevent overflow
                new_coords_f64[dim] = intermediate_val;
            }
            b_star[i].f64_coords -= new_coords_f64.clone();
            b_star[i].coords = new_coords_f64.map(|x| x.round() as i64);
        }
    }

    let mut k = 1;
    while k < n {
        // Size reduction
        for i in (0..k).rev() {
            let r_f64 = mu[(k, i)];
            let r_i128 = r_f64.round() as i128; // Use i128 here
            let mut change_basis = DVector::<i64>::zeros(n);

            for dim in 0..n {
                change_basis[dim] = (r_i128 * basis[i].coords[dim] as i128).try_into().unwrap(); // Convert i128 back to i64
            }

            basis[k].coords -= change_basis;
            basis[k].set_f64_coords();

            for l in 0..=i {
                mu[(k, l)] -= r_f64 * mu[(i, l)];
            }
        }
        // Adaptive Lovász condition
        let local_density = calculate_local_density(basis, k);
        let lovasz_factor = calculate_lovasz_factor(local_density);
        // Lovász condition
        if b_star[k].norm().powi(2) + (mu[(k, k - 1)].round() as f64 * b_star[k - 1].norm().powi(2)) < lovasz_factor * b_star[k - 1].norm().powi(2) {
            basis.swap(k - 1, k);
            // Recompute mu and b_star after swap
            b_star.clear();
            mu = DMatrix::zeros(n, n);
            for i in 0..n {
                b_star.push(basis[i].clone());
                for j in 0..i {
                    mu[(i, j)] = b_star[i].f64_coords.dot(&b_star[j].f64_coords) / b_star[j].norm().powi(2);
                    let new_coords_f64 = mu[(i, j)] * &b_star[j].f64_coords;
                    b_star[i].f64_coords -= new_coords_f64.clone();
                    b_star[i].coords = new_coords_f64.map(|x| x.round() as i64);
                }
            }
            k = 1;
        } else {
            k += 1;
        }
    }
}

fn create_rotation_matrix(n: usize, i: usize, j: usize, theta: f64) -> DMatrix<f64> {
    let mut matrix = DMatrix::identity(n, n);
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();

    matrix[(i, i)] = cos_theta;
    matrix[(j, j)] = cos_theta;
    matrix[(i, j)] = -sin_theta;
    matrix[(j, i)] = sin_theta;

    matrix
}

fn rotate_vector(v: &DVector<f64>, i: usize, j: usize, theta: f64) -> DVector<f64> {
    let n = v.len();
    let rotation_matrix = create_rotation_matrix(n, i, j, theta);
    rotation_matrix * v
}

/// Multi-frame search rotates the basis by 45 degrees and
///
/// calculates best basis via `adaptive_lll` on each frame.
pub fn multi_frame_search(basis: &mut Vec<AdloVector>) {
    let mut best_basis = basis.to_vec();
    let mut best_norm = best_basis[0].norm();
    let n = basis.len();
    // Generate multiple frames
    for angle in (0..360).step_by(45) {
        let angle_rad = (angle as f64).to_radians();
        // Iterate over all pairs of dimensions
        for i in 0..n {
            for j in i + 1..n { // Avoid rotating a dimension with itself and avoid duplicates
                let mut rotated_basis = basis.to_vec();
                for vec in &mut rotated_basis {
                    // Rotate in the i-j plane
                    let rotated_f64_coords = rotate_vector(&vec.f64_coords, i, j, angle_rad);
                    let rotated_i64_coords = rotated_f64_coords.map(|x| x.round() as i64);
                    vec.f64_coords = rotated_f64_coords;
                    vec.coords = rotated_i64_coords;
                }
                adaptive_lll(&mut rotated_basis);
                if rotated_basis[0].norm() < best_norm {
                    best_norm = rotated_basis[0].norm();
                    best_basis = rotated_basis;
                }
            }
        }
    }
    *basis = best_basis;
}

// Tests
//-------------------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn basis_2d_test() {
        let mut basis_2d = vec![
            AdloVector::new(DVector::from_vec(vec![5, 3])),
            AdloVector::new(DVector::from_vec(vec![2, 2]))
        ];
        multi_frame_search(&mut basis_2d);
        let mut expected = AdloVector::new(DVector::from_vec(vec![1, -1]));
        expected.norm_sq = 34;
        assert_eq!(expected, basis_2d[basis_2d.len()-1]);
    }

    #[test]
    fn basis_5d_test() {
        let mut basis_5d = vec![
            AdloVector::new(DVector::from_vec(vec![1, 1, 1])),
            AdloVector::new(DVector::from_vec(vec![-1, 0, 2])),
            AdloVector::new(DVector::from_vec(vec![3, 5, 6])),
        ];
        multi_frame_search(&mut basis_5d);
        let mut expected = AdloVector::new(DVector::from_vec(vec![-1, 0, 2]));
        expected.norm_sq = 5; 
        assert_eq!(expected, basis_5d[basis_5d.len()-2]);
    }
}

