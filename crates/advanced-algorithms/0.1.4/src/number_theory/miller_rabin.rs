//! Miller-Rabin Primality Test
//!
//! A probabilistic algorithm for testing whether a number is prime.
//! This is crucial for cryptographic applications, especially RSA key generation.
//!
//! The algorithm provides a probabilistic guarantee: if it says a number is composite,
//! it's definitely composite. If it says a number is probably prime, the probability
//! of error decreases exponentially with the number of iterations.
//!
//! # Examples
//!
//! ```
//! use advanced_algorithms::number_theory::miller_rabin;
//!
//! // Test if 17 is prime with 10 iterations
//! assert!(miller_rabin::is_prime(17, 10));
//!
//! // Test if 15 is prime
//! assert!(!miller_rabin::is_prime(15, 10));
//!
//! // Test a large prime
//! let large_prime = 1_000_000_007u64;
//! assert!(miller_rabin::is_prime(large_prime, 20));
//! ```

use rand::Rng;

/// Tests if a number is probably prime using the Miller-Rabin algorithm
///
/// # Arguments
///
/// * `n` - The number to test
/// * `iterations` - Number of iterations (higher = more accurate, typical: 10-40)
///
/// # Returns
///
/// `true` if the number is probably prime, `false` if definitely composite
///
/// # Accuracy
///
/// The probability of a composite number passing k iterations is at most 4^(-k).
/// With 20 iterations, the error probability is less than 1 in a trillion.
///
/// # Panics
///
/// Panics if n < 2
pub fn is_prime(n: u64, iterations: usize) -> bool {
    // Handle small cases
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    
    // Write n-1 as 2^r * d
    let (r, d) = factor_power_of_two(n - 1);
    
    let mut rng = rand::thread_rng();
    
    'witness_loop: for _ in 0..iterations {
        // Pick a random witness a in [2, n-2]
        let a = rng.gen_range(2..n - 1);
        
        // Compute x = a^d mod n
        let mut x = mod_pow(a, d, n);
        
        if x == 1 || x == n - 1 {
            continue 'witness_loop;
        }
        
        // Repeat r-1 times
        for _ in 0..r - 1 {
            x = mod_mul(x, x, n);
            
            if x == n - 1 {
                continue 'witness_loop;
            }
        }
        
        // n is composite
        return false;
    }
    
    // n is probably prime
    true
}

/// Deterministic Miller-Rabin for 64-bit integers
///
/// Uses a predetermined set of witnesses that work for all 64-bit integers
pub fn is_prime_deterministic(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n.is_multiple_of(2) {
        return false;
    }
    
    // Small primes to check
    let small_primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    
    for &p in &small_primes {
        if n == p {
            return true;
        }
        if n.is_multiple_of(p) {
            return false;
        }
    }
    
    // Witnesses that work for all 64-bit integers
    let witnesses = if n < 2_047 {
        vec![2]
    } else if n < 1_373_653 {
        vec![2, 3]
    } else if n < 9_080_191 {
        vec![31, 73]
    } else if n < 25_326_001 {
        vec![2, 3, 5]
    } else if n < 3_215_031_751 {
        vec![2, 3, 5, 7]
    } else if n < 4_759_123_141 {
        vec![2, 7, 61]
    } else if n < 1_122_004_669_633 {
        vec![2, 13, 23, 1662803]
    } else if n < 2_152_302_898_747 {
        vec![2, 3, 5, 7, 11]
    } else if n < 3_474_749_660_383 {
        vec![2, 3, 5, 7, 11, 13]
    } else if n < 341_550_071_728_321 {
        vec![2, 3, 5, 7, 11, 13, 17]
    } else {
        vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    };
    
    let (r, d) = factor_power_of_two(n - 1);
    
    'witness_loop: for a in witnesses {
        if a >= n {
            continue;
        }
        
        let mut x = mod_pow(a, d, n);
        
        if x == 1 || x == n - 1 {
            continue 'witness_loop;
        }
        
        for _ in 0..r - 1 {
            x = mod_mul(x, x, n);
            
            if x == n - 1 {
                continue 'witness_loop;
            }
        }
        
        return false;
    }
    
    true
}

/// Generates a random prime number with approximately the given bit length
///
/// # Arguments
///
/// * `bits` - Approximate bit length of the prime (must be > 2)
/// * `iterations` - Miller-Rabin iterations for each candidate
///
/// # Returns
///
/// A random prime number
pub fn generate_prime(bits: u32, iterations: usize) -> u64 {
    assert!(bits > 2 && bits <= 63, "Bit length must be between 3 and 63");
    
    let mut rng = rand::thread_rng();
    
    loop {
        // Generate a random odd number with the specified bit length
        let mut candidate = rng.gen_range(1u64 << (bits - 1)..(1u64 << bits));
        
        // Make it odd
        candidate |= 1;
        
        // Ensure it has the correct bit length
        candidate |= 1u64 << (bits - 1);
        
        if is_prime(candidate, iterations) {
            return candidate;
        }
    }
}

/// Factors out powers of 2 from n
///
/// Returns (r, d) where n = 2^r * d and d is odd
fn factor_power_of_two(mut n: u64) -> (u64, u64) {
    let mut r = 0;
    
    while n.is_multiple_of(2) {
        n /= 2;
        r += 1;
    }
    
    (r, n)
}

/// Computes (base^exp) mod m using binary exponentiation
fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut result = 1u64;
    base %= m;
    
    while exp > 0 {
        if exp % 2 == 1 {
            result = mod_mul(result, base, m);
        }
        base = mod_mul(base, base, m);
        exp /= 2;
    }
    
    result
}

/// Computes (a * b) mod m avoiding overflow
fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
    // Use u128 to avoid overflow
    ((a as u128 * b as u128) % m as u128) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_small_primes() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31];
        
        for p in primes {
            assert!(is_prime(p, 10), "{} should be prime", p);
            assert!(is_prime_deterministic(p), "{} should be prime (deterministic)", p);
        }
    }
    
    #[test]
    fn test_composites() {
        let composites = vec![4, 6, 8, 9, 10, 12, 14, 15, 16, 18, 20];
        
        for c in composites {
            assert!(!is_prime(c, 10), "{} should be composite", c);
            assert!(!is_prime_deterministic(c), "{} should be composite (deterministic)", c);
        }
    }
    
    #[test]
    fn test_large_prime() {
        let large_prime = 1_000_000_007u64;
        assert!(is_prime(large_prime, 20));
        assert!(is_prime_deterministic(large_prime));
    }
    
    #[test]
    fn test_carmichael_number() {
        // 561 is a Carmichael number (pseudoprime to many bases)
        assert!(!is_prime(561, 20));
        assert!(!is_prime_deterministic(561));
    }
    
    #[test]
    fn test_generate_prime() {
        let prime = generate_prime(16, 20);
        assert!(is_prime_deterministic(prime));
        assert!(prime >= 1 << 15 && prime < 1 << 16);
    }
}
