//! Number theory primitives — primality, factorization, combinatorics.
//!
//! Pure functions for integer number theory, suitable for use in expression
//! evaluation and as building blocks for higher-level math operations.
//!
//! # Quick Start
//!
//! ```rust
//! use abaco::ntheory;
//!
//! assert!(ntheory::is_prime(104729));
//! assert_eq!(ntheory::next_prime(100), 101);
//! assert_eq!(ntheory::factor(360), vec![2u64, 2, 2, 3, 3, 5]);
//! assert_eq!(ntheory::totient(12), 4);
//! assert_eq!(ntheory::fibonacci(10), 55);
//! assert_eq!(ntheory::binomial(10, 3), 120);
//! ```

// ── Primality testing ───────────────────────────────────────────────────────

/// Modular exponentiation: `(base^exp) mod modulus` without overflow.
#[inline]
fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let modulus = modulus as u128;
    let mut result: u128 = 1;
    base %= modulus as u64;
    let mut base = base as u128;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result as u64
}

/// Deterministic Miller-Rabin primality test.
///
/// Correct for all `n < 2^64` using the witnesses from Sorenson & Webster (2015).
/// Returns `true` if `n` is prime.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::is_prime;
/// assert!(is_prime(2));
/// assert!(is_prime(104729));
/// assert!(!is_prime(100));
/// assert!(!is_prime(1));
/// ```
#[must_use]
#[inline]
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n < 4 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }
    // Small primes fast path
    if n < 25 {
        return matches!(n, 5 | 7 | 11 | 13 | 17 | 19 | 23);
    }

    // Write n-1 as 2^r * d
    let mut d = n - 1;
    let mut r = 0u32;
    while d.is_multiple_of(2) {
        d /= 2;
        r += 1;
    }

    // Deterministic witnesses for n < 2^64 (Sorenson & Webster, 2015)
    let witnesses: &[u64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    'witness: for &a in witnesses {
        if a >= n {
            continue;
        }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = mod_pow(x, 2, n);
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

/// Return the smallest prime greater than `n`.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::next_prime;
/// assert_eq!(next_prime(100), 101);
/// assert_eq!(next_prime(0), 2);
/// assert_eq!(next_prime(2), 3);
/// ```
#[must_use]
#[inline]
pub fn next_prime(n: u64) -> u64 {
    if n < 2 {
        return 2;
    }
    let mut candidate = if n.is_multiple_of(2) { n + 1 } else { n + 2 };
    while !is_prime(candidate) {
        candidate = match candidate.checked_add(2) {
            Some(c) => c,
            None => return candidate, // overflow — no prime found in u64 range
        };
    }
    candidate
}

/// Return the largest prime less than `n`, or `None` if `n <= 2`.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::prev_prime;
/// assert_eq!(prev_prime(100), Some(97));
/// assert_eq!(prev_prime(3), Some(2));
/// assert_eq!(prev_prime(2), None);
/// ```
#[must_use]
#[inline]
pub fn prev_prime(n: u64) -> Option<u64> {
    if n <= 2 {
        return None;
    }
    if n == 3 {
        return Some(2);
    }
    let mut candidate = if n.is_multiple_of(2) { n - 1 } else { n - 2 };
    while candidate >= 2 && !is_prime(candidate) {
        candidate = candidate.checked_sub(2)?;
    }
    if candidate >= 2 {
        Some(candidate)
    } else {
        None
    }
}

// ── Factorization ───────────────────────────────────────────────────────────

/// Return the prime factorization of `n` as a sorted vector.
///
/// Returns an empty vector for `n < 2`.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::factor;
/// assert_eq!(factor(360), vec![2u64, 2, 2, 3, 3, 5]);
/// assert_eq!(factor(17), vec![17u64]);
/// assert_eq!(factor(1), Vec::<u64>::new());
/// ```
#[must_use]
#[inline]
pub fn factor(mut n: u64) -> Vec<u64> {
    if n < 2 {
        return Vec::new();
    }
    let mut factors = Vec::new();

    // Trial division for small factors
    while n.is_multiple_of(2) {
        factors.push(2);
        n /= 2;
    }
    let mut d = 3u64;
    while d * d <= n {
        while n.is_multiple_of(d) {
            factors.push(d);
            n /= d;
        }
        d += 2;
    }
    if n > 1 {
        factors.push(n);
    }
    factors
}

// ── Arithmetic functions ────────────────────────────────────────────────────

/// Euler's totient function — count of integers in `[1, n]` coprime to `n`.
///
/// Returns 0 for `n == 0`.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::totient;
/// assert_eq!(totient(1), 1);
/// assert_eq!(totient(12), 4);
/// assert_eq!(totient(97), 96); // prime
/// ```
#[must_use]
#[inline]
pub fn totient(mut n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut result = n;
    let mut p = 2u64;
    while p * p <= n {
        if n.is_multiple_of(p) {
            while n.is_multiple_of(p) {
                n /= p;
            }
            result -= result / p;
        }
        p += 1;
    }
    if n > 1 {
        result -= result / n;
    }
    result
}

// ── Combinatorics ───────────────────────────────────────────────────────────

/// Fibonacci number `F(n)` using fast doubling.
///
/// Returns exact results for `n <= 93` (max that fits in u64).
/// Returns `u64::MAX` for overflow.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::fibonacci;
/// assert_eq!(fibonacci(0), 0);
/// assert_eq!(fibonacci(1), 1);
/// assert_eq!(fibonacci(10), 55);
/// assert_eq!(fibonacci(50), 12586269025);
/// ```
#[must_use]
#[inline]
pub fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    // Fast doubling: F(2k) = F(k)[2F(k+1) - F(k)], F(2k+1) = F(k)^2 + F(k+1)^2
    let mut a: u64 = 0; // F(0)
    let mut b: u64 = 1; // F(1)
    let bits = 64 - n.leading_zeros();
    for i in (0..bits).rev() {
        // Double: (a, b) → (F(2k), F(2k+1))
        let c = a.saturating_mul(b.saturating_mul(2).saturating_sub(a));
        let d = a.saturating_mul(a).saturating_add(b.saturating_mul(b));
        a = c;
        b = d;
        if (n >> i) & 1 == 1 {
            let next = a.saturating_add(b);
            a = b;
            b = next;
        }
    }
    a
}

/// Binomial coefficient `C(n, k)` = `n! / (k! * (n-k)!)`.
///
/// Uses multiplicative formula to avoid intermediate overflow where possible.
/// Returns 0 if `k > n`.
///
/// # Examples
///
/// ```
/// use abaco::ntheory::binomial;
/// assert_eq!(binomial(10, 3), 120);
/// assert_eq!(binomial(0, 0), 1);
/// assert_eq!(binomial(5, 5), 1);
/// assert_eq!(binomial(20, 10), 184756);
/// ```
#[must_use]
#[inline]
pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    // Symmetry: C(n, k) = C(n, n-k)
    let k = k.min(n - k);
    let mut result: u64 = 1;
    for i in 0..k {
        // result = result * (n - i) / (i + 1), always exact for integers
        result = result
            .checked_mul(n - i)
            .map(|v| v / (i + 1))
            .unwrap_or(u64::MAX);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_prime ─────────────────────────────────────────────────────────

    #[test]
    fn primes_small() {
        let small_primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        for &p in &small_primes {
            assert!(is_prime(p), "{p} should be prime");
        }
    }

    #[test]
    fn composites_small() {
        let composites = [0, 1, 4, 6, 8, 9, 10, 12, 14, 15, 16, 18, 20, 21, 25, 27, 28];
        for &c in &composites {
            assert!(!is_prime(c), "{c} should not be prime");
        }
    }

    #[test]
    fn primes_large() {
        // Known large primes
        assert!(is_prime(104729));
        assert!(is_prime(1_000_000_007));
        assert!(is_prime(999_999_999_989));
    }

    #[test]
    fn composites_carmichael() {
        // Carmichael numbers — fool simple Fermat tests
        let carmichaels = [561, 1105, 1729, 2465, 2821, 6601, 8911];
        for &c in &carmichaels {
            assert!(!is_prime(c), "Carmichael number {c} should not be prime");
        }
    }

    #[test]
    fn prime_edge_cases() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(u64::MAX)); // even-ish, definitely composite
    }

    // ── next_prime / prev_prime ──────────────────────────────────────────

    #[test]
    fn next_prime_basic() {
        assert_eq!(next_prime(0), 2);
        assert_eq!(next_prime(1), 2);
        assert_eq!(next_prime(2), 3);
        assert_eq!(next_prime(4), 5);
        assert_eq!(next_prime(100), 101);
        assert_eq!(next_prime(113), 127);
    }

    #[test]
    fn prev_prime_basic() {
        assert_eq!(prev_prime(2), None);
        assert_eq!(prev_prime(3), Some(2));
        assert_eq!(prev_prime(10), Some(7));
        assert_eq!(prev_prime(100), Some(97));
        assert_eq!(prev_prime(128), Some(127));
    }

    // ── factor ───────────────────────────────────────────────────────────

    #[test]
    fn factor_basic() {
        assert_eq!(factor(0), Vec::<u64>::new());
        assert_eq!(factor(1), Vec::<u64>::new());
        assert_eq!(factor(2), vec![2u64]);
        assert_eq!(factor(12), vec![2, 2, 3]);
        assert_eq!(factor(360), vec![2, 2, 2, 3, 3, 5]);
    }

    #[test]
    fn factor_prime() {
        assert_eq!(factor(97), vec![97]);
        assert_eq!(factor(104729), vec![104729]);
    }

    #[test]
    fn factor_powers_of_two() {
        assert_eq!(factor(64), vec![2, 2, 2, 2, 2, 2]);
        assert_eq!(factor(1024), vec![2; 10]);
    }

    #[test]
    fn factor_product_is_original() {
        for n in [360, 1234567, 999999, 2u64.pow(20) - 1] {
            let factors = factor(n);
            let product: u64 = factors.iter().product();
            assert_eq!(product, n, "factor product mismatch for {n}");
        }
    }

    // ── totient ──────────────────────────────────────────────────────────

    #[test]
    fn totient_basic() {
        assert_eq!(totient(0), 0);
        assert_eq!(totient(1), 1);
        assert_eq!(totient(2), 1);
        assert_eq!(totient(6), 2);
        assert_eq!(totient(12), 4);
        assert_eq!(totient(36), 12);
    }

    #[test]
    fn totient_prime() {
        // totient(p) = p - 1 for prime p
        assert_eq!(totient(97), 96);
        assert_eq!(totient(101), 100);
    }

    #[test]
    fn totient_power_of_two() {
        // totient(2^k) = 2^(k-1)
        assert_eq!(totient(8), 4);
        assert_eq!(totient(16), 8);
        assert_eq!(totient(1024), 512);
    }

    // ── fibonacci ────────────────────────────────────────────────────────

    #[test]
    fn fibonacci_basic() {
        let expected = [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];
        for (i, &f) in expected.iter().enumerate() {
            assert_eq!(fibonacci(i as u64), f, "fibonacci({i})");
        }
    }

    #[test]
    fn fibonacci_large() {
        assert_eq!(fibonacci(50), 12_586_269_025);
        assert_eq!(fibonacci(93), 12_200_160_415_121_876_738); // max that fits u64
    }

    // ── binomial ─────────────────────────────────────────────────────────

    #[test]
    fn binomial_basic() {
        assert_eq!(binomial(0, 0), 1);
        assert_eq!(binomial(5, 0), 1);
        assert_eq!(binomial(5, 5), 1);
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(10, 3), 120);
        assert_eq!(binomial(20, 10), 184_756);
    }

    #[test]
    fn binomial_symmetry() {
        for n in 0..=20 {
            for k in 0..=n {
                assert_eq!(
                    binomial(n, k),
                    binomial(n, n - k),
                    "C({n},{k}) != C({n},{})",
                    n - k
                );
            }
        }
    }

    #[test]
    fn binomial_k_greater_than_n() {
        assert_eq!(binomial(3, 5), 0);
    }

    #[test]
    fn binomial_pascals_triangle() {
        // C(n,k) = C(n-1,k-1) + C(n-1,k)
        for n in 2..=15 {
            for k in 1..n {
                assert_eq!(
                    binomial(n, k),
                    binomial(n - 1, k - 1) + binomial(n - 1, k),
                    "Pascal's rule failed for C({n},{k})"
                );
            }
        }
    }
}
