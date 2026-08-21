/// Membership Witness Management

use crate::subroutines;
use crate::proofs;
use alloc::vec::Vec;
use core::str::FromStr;
use num_bigint::BigUint;


/// Given an old state, the product of a set of elements that have been added, and a single element from that
/// set, returns the witness for that element.
/// NOTE: "old_state" represents the state *before* the elements are added.
/// This function will likely be used by an online user.
pub fn mem_wit_create(old_state: BigUint, agg: BigUint, elem: BigUint) -> Option<BigUint> {
    if &agg % &elem != BigUint::from(0u8) {
        return None;
    }
    let quotient = agg / elem;
    return Some(subroutines::mod_exp_big_uint(&old_state, &quotient, &BigUint::from_str(super::MODULUS).unwrap()));
}

/// Verify the witness of an element.
pub fn verify_mem_wit(state: BigUint, witness: BigUint, elem: BigUint) -> bool {
    let result = subroutines::mod_exp_big_uint(&witness, &elem, &BigUint::from_str(super::MODULUS).unwrap());
    return result == state;
}

/// Updates a membership witness based on untracked additions and deletions. Algorithm is based on
/// section 3.2 of the paper titled "Dynamic Accumulators and Applications to Efficient Revocation of
/// Anonymous Credentials". Note that "additions" represent the product of the added elements
/// and "deletions" represents the product of the deleted elements.
/// NOTE: Does not do any error checking on unwrap.
pub fn update_mem_wit(elem: BigUint, mut witness: BigUint, new_state: BigUint, additions: BigUint, deletions: BigUint) -> BigUint {
    // Handle added elems
    witness = subroutines::mod_exp_big_uint(&witness, &additions, &BigUint::from_str(super::MODULUS).unwrap());

    // Handle deleted elems
    witness = subroutines::shamir_trick_big_uint(witness, new_state, elem, deletions).unwrap();
    return witness;
}


/// Takes two elements + membership witnesses and returns the aggregated witness and aggregated proof.
/// NOTE: Does very little error checking (Ex: Does not do any error checking on unwrap).
pub fn agg_mem_wit(state: BigUint, witness_x: BigUint, witness_y: BigUint, x: BigUint, y: BigUint) -> (BigUint, BigUint) {
    let aggregated = subroutines::shamir_trick_big_uint(witness_x, witness_y, x.clone(), y.clone()).unwrap();
    let proof = proofs::poe_big_uint(&aggregated, &subroutines::mul_mod_big_uint(&x, &y, &BigUint::from_str(super::MODULUS).unwrap()), &state);
    return (aggregated, proof);
}

/// Verifies that a membership witness + proof for a set of accumulator elements are valid. Acts as a
/// wrapper for the proof of exponentiation verifier.
pub fn verify_agg_mem_wit(state: BigUint, agg_elems: BigUint, witness: BigUint, proof: BigUint) -> bool {
    return proofs::verify_poe_big_uint(&witness, &agg_elems, &state, &proof);
}

/// Creates individual membership witnesses. Acts as a wrapper for the RootFactor subroutine.
/// NOTE: "old_state" represents the state *before* the elements are added.
/// This function will most likely be used by a service provider.
pub fn create_all_mem_wit(old_state: &BigUint, new_elems: &[BigUint]) -> Vec<BigUint> {
    return subroutines::root_factor_big_uint(old_state.clone(), new_elems);
}


/// Below contains all of the non-membership witness functions required for vector commitments.
/// It is important to note that these functions allow one to specify a reference generator(typically
/// as "old_state") for the non-membership proof since the accumulated set may be too large to be
/// inputted.

/// Creates a non-membership witness relative to some previous state. The current state should equal "old_state"
/// raised to the "agg_elems" power(represents product of added elements). The second value of the
/// tuple is the sign of the first value since the Bezout coefficient may be negative.
/// NOTE: Function assumes that "elem" is not contained in "agg_elems"
pub fn non_mem_wit_create(mut old_state: BigUint, agg_elems: BigUint, elem: BigUint) -> (BigUint, bool, BigUint) {
    let pair = subroutines::bezout_big_uint(agg_elems, elem).unwrap();

    if pair.sign_b {
        old_state = subroutines::mod_inverse_big_uint(old_state);
    }

    let B = subroutines::mod_exp_big_uint(&old_state, &BigUint::from(pair.coefficient_b), &BigUint::from_str(super::MODULUS).unwrap());
    return (pair.coefficient_a, pair.sign_a, B);
}

/// Verifies a non-membership witness. "state" represents the current state.
pub fn verify_non_mem_wit(old_state: BigUint, mut state: BigUint, witness: (BigUint, bool, BigUint), elem: BigUint) -> bool {
    let (a, sign_a, B) = witness;

    if sign_a {
        state = subroutines::mod_inverse_big_uint(state);
    }

    let exp_1 = subroutines::mod_exp_big_uint(&state, &BigUint::from(a), &BigUint::from_str(super::MODULUS).unwrap());
    let exp_2 = subroutines::mod_exp_big_uint(&B, &elem, &BigUint::from_str(super::MODULUS).unwrap());

    return subroutines::mul_mod_big_uint(&exp_1, &exp_2, &BigUint::from_str(super::MODULUS).unwrap()) == old_state;
}

pub fn update_non_mem_wit() {}

/// OPTIONAL FUNCTION.
/// Given the current state, the previous state, the product of the added elements, and a subset of
/// those elements, creates a witness for thoise elements.
pub fn mem_wit_create_star(cur_state: BigUint, old_state: BigUint, agg: BigUint, new_elems: Vec<BigUint>) -> (BigUint, BigUint) {
    let product = subroutines::prime_product(&new_elems);
    let witness = mem_wit_create(old_state, agg, product.clone()).unwrap();
    let proof = proofs::poe_big_uint(&witness, &product, &cur_state);
    return (witness, proof);
}


#[cfg(test)]
mod tests {
    use crate::batch_add_big_uint;
    use super::*;

    #[test]
    fn test_mem_wit_create() {
        assert_eq!(mem_wit_create(BigUint::from(2u8), BigUint::from(1155u16), BigUint::from(3u8)).unwrap(), BigUint::from(2u8));
        assert_eq!(mem_wit_create(BigUint::from(2u8), BigUint::from(1155u16), BigUint::from(5u8)).unwrap(), BigUint::from(8u8));
        assert_eq!(mem_wit_create(BigUint::from(2u8), BigUint::from(1155u16), BigUint::from(7u8)).unwrap(), BigUint::from(5u8));
        assert_eq!(mem_wit_create(BigUint::from(2u8), BigUint::from(1155u16),BigUint::from(11u8)).unwrap(), BigUint::from(5u8));
        assert_eq!(mem_wit_create(BigUint::from(2u8), BigUint::from(1155u16),BigUint::from(4u8)).is_none(), true);
    }

    #[test]
    fn test_agg_mem_wit() {
        let (aggregate, proof) = agg_mem_wit(BigUint::from(8u8), BigUint::from(6u8), BigUint::from(8u8),BigUint::from(3u8), BigUint::from(5u8));
        assert_eq!(aggregate, BigUint::from(2u8));
        assert_eq!(verify_agg_mem_wit(BigUint::from(8u8), BigUint::from(15u8), aggregate, proof), true);
    }

    #[test]
    fn test_verify_agg_mem_wit() {
        let proof = proofs::poe_big_uint(&BigUint::from(2u8), &BigUint::from(12123u16), &BigUint::from(8u8));
        assert_eq!(verify_agg_mem_wit(BigUint::from(8u8), BigUint::from(12123u16), BigUint::from(2u8), proof.clone()), true);
        assert_eq!(verify_agg_mem_wit(BigUint::from(7u8), BigUint::from(12123u16), BigUint::from(2u8), proof.clone()), false);
    }

    #[test]
    fn test_update_mem_wit() {
        let deletions = BigUint::from(15u8);
        let additions = BigUint::from(77u8);

        let elem = BigUint::from(12131u16);
        let witness = BigUint::from(8u8);
        let new_state = BigUint::from(11u8);

        assert_eq!(update_mem_wit(elem, witness, new_state, additions, deletions), BigUint::from(6u8));
    }

    #[test]
    fn test_create_all_mem_wit() {
        assert_eq!(create_all_mem_wit(&BigUint::from(2u8), &vec![BigUint::from(3u8), BigUint::from(5u8), BigUint::from(7u8), BigUint::from(11u8)]),
                   vec![BigUint::from(2u8), BigUint::from(8u8), BigUint::from(5u8), BigUint::from(5u8)]);
    }

    // Begin tests for non-membership witnesses.

    #[test]
    fn test_non_mem_wit() {
        let (a, sign_a, B) = non_mem_wit_create(BigUint::from(2u8), BigUint::from(105u8), BigUint::from(11u8));

        assert_eq!(verify_non_mem_wit(BigUint::from(2u8), BigUint::from(5u8), (a.clone(), sign_a.clone(), B.clone()), BigUint::from(11u8)), true);
        assert_eq!(verify_non_mem_wit(BigUint::from(2u8), BigUint::from(6u8), (a.clone(), sign_a.clone(), B.clone()), BigUint::from(11u8)), false);
        assert_eq!(verify_non_mem_wit(BigUint::from(2u8), BigUint::from(5u8), (a.clone(), sign_a.clone(), B.clone()), BigUint::from(5u8)), false);
    }

    #[test]
    fn test_mem_wit_create_star() {
        let old_state = BigUint::from(2u8);
        let new_elems = vec![BigUint::from(3u8), BigUint::from(5u8), BigUint::from(7u8), BigUint::from(11u8), BigUint::from(17u8)];
        let (new_state, agg, _) = batch_add_big_uint(&old_state, &new_elems);

        let subset = vec![BigUint::from(5u8), BigUint::from(11u8), BigUint::from(17u8)];
        let subset_product = subroutines::prime_product(&subset);
        let (witness, proof) = mem_wit_create_star(new_state.clone(), old_state, agg, subset);

        assert_eq!(witness, BigUint::from(5u8));
        assert_eq!(proofs::verify_poe_big_uint(&witness, &subset_product, &new_state, &proof), true);
    }


}