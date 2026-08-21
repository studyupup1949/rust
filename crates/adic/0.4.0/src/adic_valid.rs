use crate::{AdicError, AdicNumber, Prime};


pub fn validate_digit_mod_p(p: Prime, digit: u32) {
    assert!(digit < p.into(), "adic numbers have digits in [0, p)");
}
pub fn validate_digits_mod_p(p: Prime, digits: &[u32]) {
    assert!(digits.iter().all(|d| *d < p.into()), "adic numbers have digits in [0, p)");
}
pub fn validate_mono_character(p1: Prime, p2: Prime) {
    assert!(p1 == p2, "{:?}", AdicError::MixedCharacteristic);
}
pub fn validate_adic_same_p<A: AdicNumber>(p: Prime, adics: &[A]) {
    assert!(adics.iter().all(|a| a.p() == p), "{:?}", AdicError::MixedCharacteristic);
}
