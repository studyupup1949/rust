use num_prime::nt_funcs::is_prime;
use crate::AdicError;


pub fn validate_p(p: u32) {
    assert!(is_prime(&p, None).probably(), "{p} is not prime");
}
pub fn validate_digit_mod_p(p: u32, digit: u32) {
    assert!(digit < p, "adic numbers have digits in [0, p)");
}
pub fn validate_digits_mod_p(p: u32, digits: &[u32]) {
    assert!(digits.iter().all(|d| *d < p), "adic numbers have digits in [0, p)");
}
pub fn validate_mono_character(p1: u32, p2: u32) {
    assert!(p1 == p2, "{:?}", AdicError::MixedCharacteristic);
}
