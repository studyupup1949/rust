//! Extended Euclidean Algorithm
//!
//! The Extended Euclidean Algorithm computes not only the greatest common divisor (GCD)
//! of two integers, but also the coefficients of Bézout's identity, which are integers
//! x and y such that ax + by = gcd(a, b).
//!
//! This is essential for:
//! - Finding modular multiplicative inverses (crucial for RSA encryption)
//! - Solving linear Diophantine equations
//! - Chinese Remainder Theorem computations
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::number_theory::extended_euclidean;
//!
//! // Find GCD of 30 and 18
//! let result = extended_euclidean::extended_gcd(30, 18);
//! assert_eq!(result.gcd, 6);
//! // Verify: 30 * x + 18 * y = 6
//! assert_eq!(30 * result.x + 18 * result.y, 6);
//!
//! // Find modular inverse of 3 mod 11
//! let inv = extended_euclidean::mod_inverse(3, 11).unwrap();
//! assert_eq!((3 * inv) % 11, 1);
//! ```

use crate::{AlgorithmError, Result};

/// Result of the Extended Euclidean Algorithm
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedGcdResult {
    /// The greatest common divisor
    pub gcd: i64,
    /// Coefficient x in ax + by = gcd(a, b)
    pub x: i64,
    /// Coefficient y in ax + by = gcd(a, b)
    pub y: i64,
}

/// Computes the Extended Euclidean Algorithm
///
/// Given integers a and b, finds gcd(a, b) and integers x, y such that:
/// ax + by = gcd(a, b)
///
/// # Arguments
///
/// * `a` - First integer
/// * `b` - Second integer
///
/// # Returns
///
/// ExtendedGcdResult containing gcd, x, and y
///
/// # Examples
///
/// ```
/// use advanced_algorithms::number_theory::extended_euclidean::extended_gcd;
///
/// let result = extended_gcd(240, 46);
/// assert_eq!(result.gcd, 2);
/// assert_eq!(240 * result.x + 46 * result.y, 2);
/// ```
pub fn extended_gcd(a: i64, b: i64) -> ExtendedGcdResult {
    if b == 0 {
        return ExtendedGcdResult {
            gcd: a.abs(),
            x: if a >= 0 { 1 } else { -1 },
            y: 0,
        };
    }
    
    let mut old_r = a;
    let mut r = b;
    let mut old_s = 1i64;
    let mut s = 0i64;
    let mut old_t = 0i64;
    let mut t = 1i64;
    
    while r != 0 {
        let quotient = old_r / r;
        
        let temp_r = r;
        r = old_r - quotient * r;
        old_r = temp_r;
        
        let temp_s = s;
        s = old_s - quotient * s;
        old_s = temp_s;
        
        let temp_t = t;
        t = old_t - quotient * t;
        old_t = temp_t;
    }
    
    ExtendedGcdResult {
        gcd: old_r.abs(),
        x: if old_r >= 0 { old_s } else { -old_s },
        y: if old_r >= 0 { old_t } else { -old_t },
    }
}

/// Computes the greatest common divisor using the standard Euclidean algorithm
///
/// # Arguments
///
/// * `a` - First integer
/// * `b` - Second integer
///
/// # Returns
///
/// The GCD of a and b
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    
    a
}

/// Computes the least common multiple
///
/// # Arguments
///
/// * `a` - First integer
/// * `b` - Second integer
///
/// # Returns
///
/// The LCM of a and b
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    
    (a.abs() / gcd(a, b)) * b.abs()
}

/// Finds the modular multiplicative inverse of a modulo m
///
/// Returns x such that (a * x) ≡ 1 (mod m)
///
/// # Arguments
///
/// * `a` - The number to find the inverse of
/// * `m` - The modulus
///
/// # Returns
///
/// The modular inverse if it exists
///
/// # Errors
///
/// Returns error if gcd(a, m) ≠ 1 (inverse doesn't exist)
///
/// # Examples
///
/// ```
/// use advanced_algorithms::number_theory::extended_euclidean::mod_inverse;
///
/// let inv = mod_inverse(3, 11).unwrap();
/// assert_eq!((3 * inv) % 11, 1);
/// ```
pub fn mod_inverse(a: i64, m: i64) -> Result<i64> {
    if m <= 1 {
        return Err(AlgorithmError::InvalidInput(
            "Modulus must be greater than 1".to_string()
        ));
    }
    
    let result = extended_gcd(a, m);
    
    if result.gcd != 1 {
        return Err(AlgorithmError::InvalidInput(
            format!("Modular inverse doesn't exist: gcd({}, {}) = {}", a, m, result.gcd)
        ));
    }
    
    // Ensure the result is positive
    let inv = ((result.x % m) + m) % m;
    
    Ok(inv)
}

/// Solves a linear congruence ax ≡ b (mod m)
///
/// # Arguments
///
/// * `a` - Coefficient
/// * `b` - Target value
/// * `m` - Modulus
///
/// # Returns
///
/// All solutions modulo m, or empty vector if no solution exists
pub fn solve_linear_congruence(a: i64, b: i64, m: i64) -> Vec<i64> {
    let g = gcd(a, m);
    
    if b % g != 0 {
        // No solution exists
        return Vec::new();
    }
    
    // Reduce the equation
    let a_reduced = a / g;
    let b_reduced = b / g;
    let m_reduced = m / g;
    
    // Find one solution
    if let Ok(a_inv) = mod_inverse(a_reduced, m_reduced) {
        let x0 = ((a_inv * b_reduced) % m_reduced + m_reduced) % m_reduced;
        
        // Generate all solutions
        let mut solutions = Vec::new();
        for i in 0..g {
            solutions.push((x0 + i * m_reduced) % m);
        }
        
        solutions
    } else {
        Vec::new()
    }
}

/// Computes GCD for multiple numbers
pub fn gcd_multiple(numbers: &[i64]) -> i64 {
    if numbers.is_empty() {
        return 0;
    }
    
    numbers.iter().fold(numbers[0], |acc, &x| gcd(acc, x))
}

/// Computes LCM for multiple numbers
pub fn lcm_multiple(numbers: &[i64]) -> i64 {
    if numbers.is_empty() {
        return 0;
    }
    
    numbers.iter().fold(numbers[0], |acc, &x| lcm(acc, x))
}

/// Checks if two numbers are coprime (gcd = 1)
pub fn are_coprime(a: i64, b: i64) -> bool {
    gcd(a, b) == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extended_gcd() {
        let result = extended_gcd(240, 46);
        assert_eq!(result.gcd, 2);
        assert_eq!(240 * result.x + 46 * result.y, 2);
    }
    
    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(100, 35), 5);
        assert_eq!(gcd(17, 19), 1);
    }
    
    #[test]
    fn test_lcm() {
        assert_eq!(lcm(12, 18), 36);
        assert_eq!(lcm(21, 6), 42);
    }
    
    #[test]
    fn test_mod_inverse() {
        let inv = mod_inverse(3, 11).unwrap();
        assert_eq!((3 * inv) % 11, 1);
        
        let inv2 = mod_inverse(7, 26).unwrap();
        assert_eq!((7 * inv2) % 26, 1);
    }
    
    #[test]
    fn test_mod_inverse_no_solution() {
        assert!(mod_inverse(6, 9).is_err());
    }
    
    #[test]
    fn test_coprime() {
        assert!(are_coprime(17, 13));
        assert!(!are_coprime(12, 18));
    }
    
    #[test]
    fn test_linear_congruence() {
        let solutions = solve_linear_congruence(3, 6, 9);
        assert!(!solutions.is_empty());
        
        // Verify each solution
        for &x in &solutions {
            assert_eq!((3 * x) % 9, 6 % 9);
        }
    }
}
