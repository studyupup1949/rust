use alloc::vec::Vec;
use core::str::FromStr;

use num_bigint::BigUint;
use crate::subroutines;



// TODO check!
pub fn my_big_uint_encode(u: &BigUint, x: &BigUint, w: &BigUint) -> Vec<u8>{
    let mut data = u.clone().to_bytes_le();
    data.append(& mut x.clone().to_bytes_le());
    data.append(& mut w.clone().to_bytes_le());
    return data;
}

/// Generates proof of exponentiation that u^x = w (based on Wesolowski). Protocol is only useful
/// if the verifier can compute the residue r = x mod l faster than computing u^x.
/// To investigate: Security parameter should be larger than that of accumulator elements.

pub fn poe_big_uint(u: &BigUint, x: &BigUint, w: &BigUint) -> BigUint {
    let l = subroutines::hash_to_prime_big_uint(&my_big_uint_encode(u, x, w));
    let q = x / &l;
    return subroutines::mod_exp_big_uint(&u, &q, &BigUint::from_str(super::MODULUS).unwrap());
}

/// Verifies proof of exponentiation.
pub fn verify_poe_big_uint(u: &BigUint, x: &BigUint, w: &BigUint, Q: &BigUint) -> bool {
    let l = subroutines::hash_to_prime_big_uint(&my_big_uint_encode(u, x, w));
    let r = x % &l;
    let lhs = subroutines::mul_mod_big_uint(&subroutines::mod_exp_big_uint(Q, &l, &BigUint::from_str(super::MODULUS).unwrap()), &subroutines::mod_exp_big_uint(u, &r, &BigUint::from_str(super::MODULUS).unwrap()),
                                            &BigUint::from_str(super::MODULUS).unwrap());
    return lhs == w.clone();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poe() {
        let mut proof = poe_big_uint(&BigUint::from(2u8), &BigUint::from(6u8), &BigUint::from(12u8));
        assert_eq!(verify_poe_big_uint(&BigUint::from(2u8), &BigUint::from(6u8), &BigUint::from(12u8), &proof), true);

        proof = poe_big_uint(&BigUint::from(121314usize), &BigUint::from(14123usize), &BigUint::from(6u8));
        assert_eq!(verify_poe_big_uint(&BigUint::from(121314usize), &BigUint::from(14123usize), &BigUint::from(6u8), &proof), true);

        // Fake proof
        assert_eq!(verify_poe_big_uint(&BigUint::from(2u8), &BigUint::from(6u8), &BigUint::from(12u8), &BigUint::from(3u8)), false);
        assert_eq!(verify_poe_big_uint(&BigUint::from(4u8), &BigUint::from(12u8), &BigUint::from(7u8), &BigUint::from(1u8)), false);
    }

}