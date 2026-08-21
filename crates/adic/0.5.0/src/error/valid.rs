use itertools::Itertools;
use crate::divisible::Prime;
use super::AdicError;


pub (crate) fn validate_matching_p<I>(p: I)
where I: IntoIterator<Item=Prime> {
    assert!(p.into_iter().all_equal(), "{:?}", AdicError::MixedCharacteristic);
}
pub (crate) fn validate_digits_mod_p(p: Prime, digits: &[u32]) {
    assert!(digits.iter().all(|d| *d < p.into()), "adic numbers have digits in [0, p)");
}
