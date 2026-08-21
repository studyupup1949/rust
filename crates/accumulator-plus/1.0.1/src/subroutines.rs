/// Integer Subroutines for Accumulator Functions.

use alloc::vec::Vec;
use core::str::FromStr;
use num_bigint::{BigUint};
use num_traits::FromPrimitive;
use crate::{hashing};


use crate::BezoutPairBigUint;

/// Implements fast modular exponentiation. Algorithm inspired by https://github.com/pwoolcoc/mod_exp-rs/blob/master/src/lib.rs
/// NOTE: Possible overflow error occurs when size of result exceeds U2048.
pub fn mod_exp_big_uint(base: &BigUint, exp: &BigUint, modulus: &BigUint) -> BigUint {
    let mut result = BigUint::from(1u8);
    let mut base = base % modulus;
    let mut exp = exp.clone();

    while exp > BigUint::from(0u8) {
        if (&exp % BigUint::from(2u8)) == BigUint::from(1u8) {
            result = crate::subroutines::mul_mod_big_uint(&result, &base, &modulus);
        }
        if exp == BigUint::from(1u8) {
            return result;
        }
        exp = &exp >> 1;
        base = crate::subroutines::mul_mod_big_uint(&base, &base, &modulus);
    }
    return result;
}

/// Defines the multiplication operation for the group. Idea courtesy of:
/// https://www.geeksforgeeks.org/how-to-avoid-overflow-in-modular-multiplication/
pub fn mul_mod_big_uint(a: &BigUint, b: &BigUint, modulus: &BigUint) -> BigUint {
    let mut result = BigUint::from(0u8);
    let mut a = a % modulus;
    let mut b = b.clone();

    while b > BigUint::from(0u8) {
        if &b % BigUint::from(2u8) == BigUint::from(1u8) {
            result = (result + &a) % modulus;
        }

        a = (a * BigUint::from(2u8)) % modulus;
        b /= BigUint::from(2u8);
    }
    return result % modulus;
}


/// Given the xth root of g and yth root of g, finds the xyth root. If the roots are invalid or
/// x and y are not coprime, None is returned. Otherwise, the function performs relevant modular
/// inverse operations on the Bezout coefficients and finds the xyth root.
pub fn shamir_trick_big_uint(mut xth_root: BigUint, mut yth_root: BigUint, x: BigUint, y: BigUint) -> Option<BigUint> {
    // Check if the inputs are valid.
    if mod_exp_big_uint(&xth_root, &x, &BigUint::from_str(super::MODULUS).unwrap())
        != mod_exp_big_uint(&yth_root, &y, &BigUint::from_str(super::MODULUS).unwrap()) {
        return None;
    }

    match bezout_big_uint(x, y) {
        None => {
            return None;
        }
        Some(coefficients) => {
            // Receive coefficient
            let pair = coefficients;

            // Calculate relevant modular inverses to allow for exponentiation later on.
            if pair.sign_b {
                xth_root = mod_inverse_big_uint(xth_root);
            }

            if pair.sign_a {
                yth_root = mod_inverse_big_uint(yth_root);
            }

            let combined_root: BigUint = (mod_exp_big_uint(&xth_root, &BigUint::from(pair.coefficient_b), &BigUint::from_str(super::MODULUS).unwrap())
                * mod_exp_big_uint(&yth_root, &BigUint::from(pair.coefficient_a), &BigUint::from_str(super::MODULUS).unwrap())) % BigUint::from_str(super::MODULUS).unwrap();
            return Some(combined_root);
        }
    }
}

pub fn shamir_batch_big_uint(vpks: &Vec<BigUint>, vms: &Vec<BigUint>, new_state: &BigUint) -> bool {
    let mut x = vpks[0].clone();
    let mut xth_root = vms[0].clone();
    for i in 1..vpks.len() {
        let y = vpks[i].clone();
        let mut yth_root = vms[i].clone();

        match bezout_big_uint(x.clone(), y.clone()) {
            None => {
                return false;
            }
            Some(coefficients) => {
                // Receive coefficient
                let pair = coefficients;
                // Calculate relevant modular inverses to allow for exponentiation later on.
                if pair.sign_b {
                    xth_root = mod_inverse_big_uint(xth_root);
                }
                if pair.sign_a {
                    yth_root = mod_inverse_big_uint(yth_root);
                }
                xth_root = (mod_exp_big_uint(&xth_root, &BigUint::from(pair.coefficient_b), &BigUint::from_str(super::MODULUS).unwrap())
                    * mod_exp_big_uint(&yth_root, &BigUint::from(pair.coefficient_a), &BigUint::from_str(super::MODULUS).unwrap())) % BigUint::from_str(super::MODULUS).unwrap();
                x = x * y;

            }
        }
    }
    if mod_exp_big_uint(&xth_root, &x, &BigUint::from_str(super::MODULUS).unwrap()) != *new_state {
        return false;
    }
    return true;
    // let proof = proofs::poe_big_uint(&xth_root, &x, &new_state);
    // return true;

}

pub fn loop_batch_big_uint(vpks: &Vec<BigUint>, vms: &Vec<BigUint>, new_state: &BigUint) -> bool{
    for (i, vpk) in vpks.iter().enumerate() {
        let state = mod_exp_big_uint(vms.get(i).unwrap(), vpk, &BigUint::from_str(super::MODULUS).unwrap());
        if state != *new_state{
            return false;
        }
    }
    return true;
}


/// Computes the modular multiplicative inverse.
/// NOTE: Does not check if gcd != 1(none exists if so).
pub fn mod_inverse_big_uint(elem: BigUint) -> BigUint {
    let (_, pair) = extended_gcd_big_uint(elem, BigUint::from_str(super::MODULUS).unwrap());

    // Accommodate for negative x coefficient
    if pair.sign_a {
        // Since we're assuming that the U2048::from(super::MODULUS) will always be larger than than coefficient in
        // absolute value, we simply subtract x from the U2048::from(super::MODULUS) to get a positive value mod N.
        let pos_a = BigUint::from_str(super::MODULUS).unwrap() - pair.coefficient_a;
        return pos_a % BigUint::from_str(super::MODULUS).unwrap();
    }
    return BigUint::from(pair.coefficient_a) % BigUint::from_str(super::MODULUS).unwrap();
}


/// Returns Bezout coefficients. Acts as a wrapper for extended_gcd.
pub fn bezout_big_uint(a: BigUint, b: BigUint) -> Option<BezoutPairBigUint> {
    let (gcd, pair) = extended_gcd_big_uint(a, b);
    // Check if a and b are coprime
    if gcd != BigUint::from(1u8) {
        return None;
    } else {
        return Some(pair);
    }
}

/// Implements the Extended Euclidean Algorithm (https://en.wikipedia.org/wiki/Extended_Euclidean_algorithm).
/// IMPORTANT NOTE: Instead of representing the coefficients as signed integers, I have represented
/// them as (|a|, sign of a) and (|b|, sign of b). This is because the current project lacks
/// support for signed BigInts.
pub fn extended_gcd_big_uint(a: BigUint, b: BigUint) -> (BigUint, BezoutPairBigUint) {
    let (mut s, mut old_s): (BigUint, BigUint) = (BigUint::from(0u8), BigUint::from(1u8));
    let (mut t, mut old_t): (BigUint, BigUint) = (BigUint::from(1u8), BigUint::from(0u8));
    let (mut r, mut old_r): (BigUint, BigUint) = (b, a);

    let (mut prev_sign_s, mut prev_sign_t): (bool, bool) = (false, false);
    let (mut sign_s, mut sign_t): (bool, bool) = (false, false);

    while r != BigUint::from(0u8) {
        let quotient = &old_r / &r;
        let new_r = &old_r - BigUint::from(quotient.clone()) * &r;
        old_r = r;
        r = new_r;

        // Hacky workaround to track the coefficient "a" as (|a|, sign of a)
        let mut new_s = &quotient * &s;
        if prev_sign_s == sign_s && new_s > old_s {
            new_s = new_s - old_s;
            if !sign_s { sign_s = true; } else { sign_s = false; }
        } else if prev_sign_s != sign_s {
            new_s = old_s + new_s;
            prev_sign_s = sign_s;
            sign_s = !sign_s;
        } else { new_s = old_s - new_s; }
        old_s = s;
        s = new_s;

        // Hacky workaround to track the coefficient "b" as (|b|, sign of b)
        let mut new_t = &quotient * &t;
        if prev_sign_t == sign_t && new_t > old_t {
            new_t = new_t - old_t;
            if !sign_t { sign_t = true; } else { sign_t = false; }
        } else if prev_sign_t != sign_t {
            new_t = old_t + new_t;
            prev_sign_t = sign_t;
            sign_t = !sign_t;
        } else { new_t = old_t - new_t; }
        old_t = t;
        t = new_t;
    }

    let pair = BezoutPairBigUint {
        coefficient_a: old_s,
        coefficient_b: old_t,
        sign_a: prev_sign_s,
        sign_b: prev_sign_t,
    };

    return (old_r, pair);
}

/// Continuously hashes the input until the result is prime. Assumes input values are transcoded in
/// little endian(uses parity-scale-codec).
/// Consideration: Currently unclear about the impact of Lambda on the security of the scheme.
pub fn hash_to_prime_big_uint(elem: &[u8]) -> BigUint {
    let mut hash = hashing::blake2_256(elem);
    let mut result = BigUint::from_bytes_le(&hash) % BigUint::from(super::LAMBDA);

    // While the resulting hash is not a prime, keep trying
    while !miller_rabin_big_uint(&result) {
        hash = hashing::blake2_256(&hash);
        result = BigUint::from_bytes_le(&hash) % BigUint::from(super::LAMBDA);
    }

    return result;
}


/// Implements a deterministic variant of the Miller-Rabin primality test for u64/u32 integers based
/// on the algorithm from the following link: https://en.wikipedia.org/wiki/Miller–Rabin_primality_test
/// Complexity of the algorithm is O((log n)^4) in soft-O notation.
pub fn miller_rabin_big_uint(n: &BigUint) -> bool {
    // Find r and d such that 2^r * d + 1 = n
    let r = (n - BigUint::from(1u8)).trailing_zeros().unwrap();
    let d = (n - BigUint::from(1u8)) >> r;

    // See https://stackoverflow.com/questions/7594307/simple-deterministic-primality-testing-for-small-numbers
    //let bases = [2,3,5,7,11,13,17]; // Deterministic for 64 bit integers
    let bases = [2, 7, 61];  // Deterministic for 32 bit integers

    'outer: for &a in bases.iter() {
        // Annoying edge case to make sure a is within [2, n-2] for small n

        if n - BigUint::from(2u8) < BigUint::from_i32(a).unwrap() { break; }

        let mut x = mod_exp_big_uint(&BigUint::from_i32(a).unwrap(), &d, n);

        if x == BigUint::from(1u8) || x == (n - BigUint::from(1u8)) {
            continue;
        }
        for _ in 1..r {
            x = mod_exp_big_uint(&x, &BigUint::from(2u8), n);
            if x == (n - BigUint::from(1u8)) {
                continue 'outer;
            }
        }
        return false;
    }
    return true;
}

/// Given an element g and a set of elements x, computes the xith root of g^x for each element
/// in the set. Runs in O(n log(n)).
pub fn root_factor_big_uint(g: BigUint, elems: &[BigUint]) -> Vec<BigUint> {
    if elems.len() == 1 {
        let mut ret = Vec::new();
        ret.push(g);
        return ret;
    }

    let n_prime = elems.len() / 2;

    let mut g_left = g.clone();
    for i in 0..n_prime {
        g_left = mod_exp_big_uint(&g_left, &elems[i], &BigUint::from_str(super::MODULUS).unwrap());
    }

    let mut g_right = g.clone();
    for i in n_prime..elems.len() {
        g_right = mod_exp_big_uint(&g_right, &elems[i], &BigUint::from_str(super::MODULUS).unwrap());
    }

    let mut left = crate::subroutines::root_factor_big_uint(g_right.clone(), &elems[0..n_prime]);
    let mut right = crate::subroutines::root_factor_big_uint(g_left.clone(), &elems[n_prime..]);
    left.append(&mut right);
    return left;
}

/// Short helper function that calculates the product of elements in the vector.
pub fn prime_product(primes: &[BigUint]) -> BigUint {
    primes.iter().product()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::MODULUS;

    #[test]
    fn test_mul_mod() {
        assert_eq!(mul_mod_big_uint(&BigUint::from(121usize), &BigUint::from(12314usize), &BigUint::from_str(MODULUS).unwrap()),
                   BigUint::from(12usize));
        assert_eq!(mul_mod_big_uint(&BigUint::from(128usize), &BigUint::from(23usize), &BigUint::from(75usize)),
                   BigUint::from(19usize));
    }

    #[test]
    fn test_mod_exp() {
        assert_eq!(mod_exp_big_uint(&BigUint::from(2u8), &BigUint::from(7u8), &BigUint::from_str(MODULUS).unwrap()), BigUint::from(11u8));
        assert_eq!(mod_exp_big_uint(&BigUint::from(7u8), &BigUint::from(15u8), &BigUint::from_str(MODULUS).unwrap()), BigUint::from(5u8));
    }

    #[test]
    fn test_extended_gcd() {
        assert_eq!(extended_gcd_big_uint(BigUint::from(180u8), BigUint::from(150u8)),
                   (BigUint::from(30u8), BezoutPairBigUint { coefficient_a: BigUint::from(1u8), coefficient_b: BigUint::from(1u8), sign_a: false, sign_b: true }));
        assert_eq!(extended_gcd_big_uint(BigUint::from(13u8), BigUint::from(17u8)),
                   (BigUint::from(1u8), BezoutPairBigUint { coefficient_a: BigUint::from(4u8), coefficient_b: BigUint::from(3u8), sign_a: false, sign_b: true }));
    }

    #[test]
    fn test_bezout() {
        assert_eq!(bezout_big_uint(BigUint::from(4u8), BigUint::from(10u8)), None);
        assert_eq!(bezout_big_uint(BigUint::from(3434usize), BigUint::from(2423usize)),
                   Some(BezoutPairBigUint { coefficient_a: BigUint::from(997usize), coefficient_b: BigUint::from(1413usize), sign_a: true, sign_b: false }));
    }

    #[test]
    fn test_shamir_trick() {
        assert_eq!(shamir_trick_big_uint(BigUint::from(11u8), BigUint::from(6u8), BigUint::from(7u8), BigUint::from(5u8)), Some(BigUint::from(7u8)));
        assert_eq!(shamir_trick_big_uint(BigUint::from(11u8), BigUint::from(7u8), BigUint::from(7u8), BigUint::from(11u8), ), Some(BigUint::from(6u8)));
        assert_eq!(shamir_trick_big_uint(BigUint::from(6u8), BigUint::from(7u8), BigUint::from(5u8), BigUint::from(11u8)), Some(BigUint::from(11u8)));
        assert_eq!(shamir_trick_big_uint(BigUint::from(12u8), BigUint::from(7u8), BigUint::from(7u8), BigUint::from(11u8)), None);
    }

    #[test]
    fn test_mod_inverse() {
        assert_eq!(mod_inverse_big_uint(BigUint::from(9u8)), BigUint::from(3u8));
        assert_eq!(mod_inverse_big_uint(BigUint::from(6u8)), BigUint::from(11u8));
    }


    #[test]
    fn test_miller_rabin() {
        assert_eq!(miller_rabin_big_uint(&BigUint::from(5usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(7usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(241usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(7919usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(48131usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(76463usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(4222234741usize)), true);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(187278659180417234321u128)), true);

        assert_eq!(miller_rabin_big_uint(&BigUint::from(21usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(87usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(155usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(9167usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(102398usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(801435usize)), false);
        assert_eq!(miller_rabin_big_uint(&BigUint::from(51456119958243usize)), false);
    }

    #[test]
    fn test_hash_to_prime() {
        // assert_eq!(hash_to_prime(&[7, 10]), U2048::from(...));
        // Key values checked: 0, 1, 2
    }

    #[test]
    fn test_root_factor() {
        assert_eq!(root_factor_big_uint(BigUint::from(2u8), &vec![BigUint::from(3u8), BigUint::from(5u8), BigUint::from(7u8), BigUint::from(11u8)]),
                   vec![BigUint::from(2u8), BigUint::from(8u8), BigUint::from(5u8), BigUint::from(5u8)]);
    }

    #[test]
    fn test_prime_product() {
        let elems = vec![BigUint::from(2u8), BigUint::from(3u8), BigUint::from(4u8)];
        assert_eq!(prime_product(&elems), BigUint::from(24u8));
    }
}