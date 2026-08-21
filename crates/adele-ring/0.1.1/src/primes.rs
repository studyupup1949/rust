//! Prime utilities, factorization, and channel (base) selection.
//!
//! All functions here are pure and have no dependency on the rest of the crate.
//!
//! # The "natural base" concept
//!
//! The *natural base* of a computation is the LCM of the radicals of all
//! denominators involved. This is the minimal base in which every fraction in
//! the computation terminates exactly. For example,
//! `natural_base(&[6, 10, 15]) == 30`, because `radical(6)=6`, `radical(10)=10`,
//! `radical(15)=15` and `lcm(6,10,15)=30`.

/// Sieve of Eratosthenes: all primes `<= n`.
pub fn primes_up_to(n: usize) -> Vec<u64> {
    if n < 2 {
        return Vec::new();
    }
    let mut is_prime = vec![true; n + 1];
    is_prime[0] = false;
    is_prime[1] = false;
    let mut i = 2usize;
    while i * i <= n {
        if is_prime[i] {
            let mut j = i * i;
            while j <= n {
                is_prime[j] = false;
                j += i;
            }
        }
        i += 1;
    }
    (2..=n)
        .filter(|&i| is_prime[i])
        .map(|i| i as u64)
        .collect()
}

/// The first `n` primes.
///
/// Uses the prime-counting upper bound `n * (ln n + ln ln n)` (scaled with a
/// safety factor) to pick a sieve limit, then expands if necessary.
pub fn first_n_primes(n: usize) -> Vec<u64> {
    if n == 0 {
        return Vec::new();
    }
    if n < 6 {
        // ln ln n is undefined / unhelpful for tiny n; use a fixed small limit.
        return primes_up_to(15).into_iter().take(n).collect();
    }
    let nf = n as f64;
    let mut limit = (nf * (nf.ln() + nf.ln().ln()) * 1.3).ceil() as usize;
    loop {
        let primes = primes_up_to(limit);
        if primes.len() >= n {
            return primes.into_iter().take(n).collect();
        }
        limit *= 2;
    }
}

/// Factorize `n` into `(prime, exponent)` pairs in ascending prime order.
/// `factorize(0)` and `factorize(1)` return an empty vector.
pub fn factorize(mut n: u64) -> Vec<(u64, u32)> {
    let mut factors = Vec::new();
    if n < 2 {
        return factors;
    }
    let mut d = 2u64;
    while d * d <= n {
        if n % d == 0 {
            let mut exp = 0u32;
            while n % d == 0 {
                n /= d;
                exp += 1;
            }
            factors.push((d, exp));
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

/// Product of the *distinct* prime factors of `n` (the squarefree kernel).
/// `radical(12) == 6`, `radical(1) == 1`, `radical(0) == 0`.
pub fn radical(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    factorize(n)
        .into_iter()
        .map(|(p, _)| p)
        .product::<u64>()
        .max(1)
}

/// The distinct prime factors of `n`, ascending.
pub fn distinct_primes(n: u64) -> Vec<u64> {
    factorize(n).into_iter().map(|(p, _)| p).collect()
}

/// Euclidean greatest common divisor.
pub fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Least common multiple. `lcm(0, x) == 0`.
pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        return 0;
    }
    a / gcd(a, b) * b
}

/// Extended Euclidean algorithm.
///
/// Returns `(g, x, y)` such that `a*x + b*y == g == gcd(a, b)`.
pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        // gcd(a,0) = |a|, keep sign info in the coefficient.
        return (a.abs(), if a < 0 { -1 } else { 1 }, 0);
    }
    let (g, x, y) = extended_gcd(b, a % b);
    (g, y, x - (a / b) * y)
}

/// Modular inverse of `a` modulo `m`, or `None` when `gcd(a, m) != 1`.
pub fn mod_inverse(a: u64, m: u64) -> Option<u64> {
    if m == 0 {
        return None;
    }
    if m == 1 {
        return Some(0);
    }
    let (g, x, _) = extended_gcd((a % m) as i64, m as i64);
    if g != 1 {
        return None;
    }
    let m_i = m as i64;
    Some((((x % m_i) + m_i) % m_i) as u64)
}

/// Determine how `1/n` behaves when written in the given `base`.
///
/// Returns:
/// - `Some(0)`  — the expansion terminates (every prime factor of `n` divides `base`),
/// - `Some(k)`  — the expansion repeats with period `k`,
/// - `None`     — degenerate input (`n == 0` or `base < 2`).
///
/// The period of the repeating part is the multiplicative order of `base`
/// modulo the part of `n` coprime to `base`.
pub fn termination_period(n: u64, base: u64) -> Option<u64> {
    if n == 0 || base < 2 {
        return None;
    }
    // Strip the factors shared with `base` — those produce the terminating prefix.
    let mut coprime_part = n;
    loop {
        let g = gcd(coprime_part, base);
        if g == 1 {
            break;
        }
        coprime_part /= g;
    }
    if coprime_part == 1 {
        return Some(0); // terminates
    }
    // Period = multiplicative order of `base` modulo `coprime_part`.
    let b = base % coprime_part;
    let mut order = 1u64;
    let mut acc = b % coprime_part;
    while acc != 1 {
        acc = (acc * b) % coprime_part;
        order += 1;
    }
    Some(order)
}

/// Minimal exact base for a set of denominators: the LCM of their radicals.
///
/// `natural_base(&[6, 10, 15]) == 30`.
pub fn natural_base(denominators: &[u64]) -> u64 {
    denominators
        .iter()
        .filter(|&&d| d != 0)
        .map(|&d| radical(d))
        .fold(1u64, lcm)
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sieve_basic() {
        assert_eq!(primes_up_to(11), vec![2, 3, 5, 7, 11]);
        assert!(primes_up_to(1).is_empty());
    }

    #[test]
    fn first_n() {
        assert_eq!(first_n_primes(5), vec![2, 3, 5, 7, 11]);
        let p = first_n_primes(32);
        assert_eq!(p.len(), 32);
        assert_eq!(*p.last().unwrap(), 131); // 32nd prime is 131
    }

    #[test]
    fn factor() {
        assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
        assert_eq!(factorize(13), vec![(13, 1)]);
        assert!(factorize(1).is_empty());
    }

    #[test]
    fn radicals() {
        assert_eq!(radical(12), 6);
        assert_eq!(radical(1), 1);
        assert_eq!(radical(30), 30);
    }

    #[test]
    fn gcd_lcm() {
        assert_eq!(gcd(12, 18), 6);
        assert_eq!(lcm(4, 6), 12);
    }

    #[test]
    fn egcd_and_inverse() {
        let (g, x, y) = extended_gcd(240, 46);
        assert_eq!(g, 2);
        assert_eq!(240 * x + 46 * y, 2);
        assert_eq!(mod_inverse(3, 11), Some(4)); // 3*4=12≡1 mod 11
        assert_eq!(mod_inverse(2, 4), None);
    }

    #[test]
    fn termination() {
        assert_eq!(termination_period(8, 10), Some(0)); // 1/8 terminates in base 10
        assert_eq!(termination_period(3, 10), Some(1)); // 1/3 = 0.333..., period 1
        assert_eq!(termination_period(7, 10), Some(6)); // 1/7 period 6
        assert_eq!(termination_period(6, 10), Some(1)); // 1/6 = 0.1666...
    }

    #[test]
    fn nat_base() {
        assert_eq!(natural_base(&[6, 10, 15]), 30);
        assert_eq!(natural_base(&[12]), 6);
    }
}
