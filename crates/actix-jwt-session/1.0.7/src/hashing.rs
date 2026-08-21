//! Encrypting and decrypting password
//!
//! This module is available by default or by enabling `hashing` feature.
//! Library docs covers using it in context of `register` and `sign in`.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Encrypting and decrypting password
pub struct Hashing;

impl Hashing {
    /// Takes password and returns encrypted hash with random salt
    pub fn encrypt(password: &str) -> argon2::password_hash::Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
    }

    /// Takes password hash and password and validates it.
    pub fn verify(password_hash: &str, password: &str) -> argon2::password_hash::Result<()> {
        let parsed_hash = PasswordHash::new(password_hash)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_always_random_salt() {
        let pass = "ahs9dya8tsd7fa8tsa86tT&^R%^DS^%ARS&A";
        let hash = Hashing::encrypt(pass).unwrap();
        assert!(Hashing::verify(hash.as_str(), pass).is_ok());
    }
}
