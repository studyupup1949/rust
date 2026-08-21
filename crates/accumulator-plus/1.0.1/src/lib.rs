use alloc::vec::Vec;
use rand;

use std::collections::HashSet;

extern crate alloc;
use num_bigint::{BigUint, ModInverse, RandPrime};


use core::str::FromStr;
use num_traits::{One, Zero};
use rand_core::CryptoRngCore;

pub mod subroutines;
pub mod proofs;
pub mod witnesses;
pub mod hashing;


pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    InvalidPrime,
    NprimesTooSmall,
    TooFewPrimes,

}

pub struct RsaPrivateKeyComponents {
    pub n: BigUint,
    pub e: BigUint,
    pub d: BigUint,
    pub primes: Vec<BigUint>,
}



#[inline]
pub(crate) fn compute_private_exponent_euler_totient(
    primes: &[BigUint],
    exp: &BigUint,
) -> Result<BigUint> {
    if primes.len() < 2 {
        return Result::Err(Error::InvalidPrime);
    }

    let mut totient = BigUint::one();

    for prime in primes {
        totient *= prime - BigUint::one();
    }

    // NOTE: `mod_inverse` checks if `exp` evenly divides `totient` and returns `None` if so.
    // This ensures that `exp` is not a factor of any `(prime - 1)`.
    if let Some(d) = exp.mod_inverse(totient) {
        Ok(d.to_biguint().unwrap())
    } else {
        // `exp` evenly divides `totient`
        Err(Error::InvalidPrime)
    }
}

pub fn generate_multi_prime_key_with_exp<R: CryptoRngCore + ?Sized>(
    rng: &mut R,
    nprimes: usize,
    bit_size: usize,
    exp: &BigUint,
) -> Result<RsaPrivateKeyComponents> {
    if nprimes < 2 {
        return Err(Error::NprimesTooSmall);
    }

    if bit_size < 64 {
        let prime_limit = (1u64 << (bit_size / nprimes) as u64) as f64;

        // pi aproximates the number of primes less than prime_limit
        let mut pi = prime_limit / (prime_limit.ln() - 1f64);
        // Generated primes start with 0b11, so we can only use a quarter of them.
        pi /= 4f64;
        // Use a factor of two to ensure that key generation terminates in a
        // reasonable amount of time.
        pi /= 2f64;

        if pi < nprimes as f64 {
            return Err(Error::TooFewPrimes);
        }
    }

    let mut primes = vec![BigUint::zero(); nprimes];
    let n_final: BigUint;
    let d_final: BigUint;

    'next: loop {
        let mut todo = bit_size;
        // `gen_prime` should set the top two bits in each prime.
        // Thus each prime has the form
        //   p_i = 2^bitlen(p_i) × 0.11... (in base 2).
        // And the product is:
        //   P = 2^todo × α
        // where α is the product of nprimes numbers of the form 0.11...
        //
        // If α < 1/2 (which can happen for nprimes > 2), we need to
        // shift todo to compensate for lost bits: the mean value of 0.11...
        // is 7/8, so todo + shift - nprimes * log2(7/8) ~= bits - 1/2
        // will give good results.
        if nprimes >= 7 {
            todo += (nprimes - 2) / 5;
        }

        for (i, prime) in primes.iter_mut().enumerate() {
            *prime = rng.gen_prime(todo / (nprimes - i));
            todo -= prime.bits();
        }

        // Makes sure that primes is pairwise unequal.
        for (i, prime1) in primes.iter().enumerate() {
            for prime2 in primes.iter().take(i) {
                if prime1 == prime2 {
                    continue 'next;
                }
            }
        }

        let n = subroutines::prime_product(&primes);

        if n.bits() != bit_size {
            // This should never happen for nprimes == 2 because
            // gen_prime should set the top two bits in each prime.
            // For nprimes > 2 we hope it does not happen often.
            continue 'next;
        }

        if let Ok(d) = compute_private_exponent_euler_totient(&primes, exp) {
            n_final = n;
            d_final = d;
            break;
        }
    }

    Result::Ok(RsaPrivateKeyComponents {
        n: n_final,
        e: exp.clone(),
        d: d_final,
        primes,
    })
}

pub fn gen_diff_prime_vec(k: usize, bits: usize, s: &mut Vec<BigUint>) {
    let mut rng = rand::thread_rng();
    let mut set = HashSet::new();
    let mut i = 0;
    if k == 0 {
        return;
    }
    loop {
        let gi = rng.gen_prime(bits);
        let si = gi;
        if set.contains(&si) {
            continue;
        }
        set.insert(si);
        i += 1;
        if i >= k { break; }
    }
    for x in set {
        s.push(x);
    }
}

pub const EXP: u64 = 65537;

/// Defines the RSA group. Arbitrary set at MODULUS = 13 for testing.
/// Example (insecure) modulus -> RSA 100: "1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139"
pub const MODULUS: &str = "13";

/// Security parameter that represents the size of elements added to the accumulator.
pub const LAMBDA: u32 = u32::max_value();

#[derive(Clone, PartialEq, Debug)]
pub struct BezoutPairBigUint {
    pub coefficient_a: BigUint,
    pub coefficient_b: BigUint,
    sign_a: bool, // True indicates negative and false indicates positive
    sign_b: bool,
}


/// Add a single element to an accumulator.
pub fn add(state: BigUint, elem: BigUint) -> BigUint {
    return subroutines::mod_exp_big_uint(&state, &elem, &BigUint::from_str(MODULUS).unwrap());
}

/// Delete an element from the accumulator given a membership proof.
pub fn delete(state: BigUint, elem: BigUint, proof: BigUint) -> Option<BigUint> {
    if subroutines::mod_exp_big_uint(&proof, &elem, &BigUint::from_str(MODULUS).unwrap()) == state {
        return Some(proof);
    }
    return None;
}

/// Aggregates a set of accumulator elements + witnesses and batch deletes them from the accumulator.
/// Returns the state after deletion, the product of the deleted elements, and a proof of exponentiation.
pub fn batch_delete_by_shamir_trick(state: BigUint, elems: &Vec<(BigUint, BigUint)>) -> (BigUint, BigUint, BigUint) {
    let (mut x_agg, mut new_state) = elems[0].clone();
    for i in 1..elems.len() {
        let (x, witness) = elems[i].clone();
        new_state = subroutines::shamir_trick_big_uint(new_state, witness, x_agg.clone(), x.clone()).unwrap();
        x_agg *= x;
    }
    let proof = proofs::poe_big_uint(&new_state, &x_agg, &state);
    return (new_state, x_agg, proof);
}

pub fn batch_delete_by_normal_loop(base_state: BigUint, state: BigUint, elems_pair: &Vec<(BigUint, BigUint)>, elems: &Vec<BigUint>) -> (BigUint, BigUint, BigUint) {
    let (mut x_agg, mut new_state) = elems_pair[0].clone();
    if elems_pair.len() == 1 {
        return (new_state, x_agg, BigUint::from(1u8));
    }
    for i in 1 .. elems_pair.len() {
        x_agg *= elems_pair[i].clone().0;
    }

    let new_wi_list = subroutines::root_factor_big_uint(base_state.clone(), &elems[elems_pair.len()-1..]);
    new_state = new_wi_list.get(0).or_else(|| Some(&new_state)).unwrap().clone();

    let proof = proofs::poe_big_uint(&new_state, &x_agg, &state);
    return (new_state, x_agg, proof);

}



pub fn batch_auth_by_shamir_trick(vpks: &Vec<BigUint>, vms: &Vec<BigUint>, new_state: &BigUint) -> bool {
    subroutines::shamir_batch_big_uint(vpks, vms, new_state)
}


pub fn batch_auth_by_normal_loop(vpks: &Vec<BigUint>, vms: &Vec<BigUint>, new_state: &BigUint) -> bool {
    subroutines::loop_batch_big_uint(vpks, vms, new_state)
}


/// Aggregates a set of accumulator elements + witnesses and batch adds them to the accumulator.
/// Returns the state after addition, the product of the added elements, and a proof of exponentiation.
pub fn batch_add_big_uint(state: &BigUint, elems: &Vec<BigUint>) -> (BigUint, BigUint, BigUint) {
    let mut x_agg = BigUint::from(1u8);
    for i in 0..elems.len() {
        x_agg *= &elems[i];
    }

    let new_state = subroutines::mod_exp_big_uint(state, &x_agg, &BigUint::from_str(MODULUS).unwrap());
    // let proof = proofs::poe(state, &x_agg, &new_state);
    // return (new_state, x_agg, proof);
    return (new_state, x_agg, BigUint::from(1u8));
}



#[cfg(test)]
mod tests{
    use num_bigint::BigUint;
    use crate::{generate_multi_prime_key_with_exp, EXP};

    #[test]
    fn gen_prims(){
        let prime_key_with_exp = generate_multi_prime_key_with_exp(&mut rand::thread_rng(),
                                                       3,
                                                       56,
                                                       &BigUint::from(EXP));
        let key_components = prime_key_with_exp.unwrap();
        println!("{}", &key_components.n);
        println!("{}", &key_components.e);
        println!("{}", &key_components.d);
        println!("{:?}", &key_components.primes);

    }


}


