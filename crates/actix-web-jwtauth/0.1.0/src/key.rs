use jsonwebtoken as jwt;
use openssl::{ec, pkey, rsa};
use std::io;

use crate::errors;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum KeyType {
    Secret,
    Rsa,
    Ec,
}

impl From<jwt::Algorithm> for KeyType {
    #[rustfmt::skip]
    #[inline(always)]
    fn from(algorithm: jwt::Algorithm) -> Self {
        match algorithm {
            jwt::Algorithm::HS256 |
            jwt::Algorithm::HS384 |
            jwt::Algorithm::HS512 => KeyType::Secret,
            jwt::Algorithm::ES256 |
            jwt::Algorithm::ES384 => KeyType::Ec,
            jwt::Algorithm::RS256 |
            jwt::Algorithm::RS384 |
            jwt::Algorithm::RS512 |
            jwt::Algorithm::PS256 |
            jwt::Algorithm::PS384 |
            jwt::Algorithm::PS512 => KeyType::Rsa,
        }
    }
}

pub(crate) fn key_content(
    key: Vec<u8>,
    passphrase: Option<Vec<u8>>,
    algorithm: jwt::Algorithm,
) -> io::Result<Vec<u8>> {
    Ok(if let Some(passphrase) = passphrase {
        match KeyType::from(algorithm) {
            KeyType::Ec => private_key_from_ec(&key, &passphrase)?,
            KeyType::Rsa => private_key_from_rsa(&key, &passphrase)?,
            KeyType::Secret => key,
        }
    } else {
        key
    })
}

// To PEM-encoded PKCS#8
fn private_key_from_ec(pem: &[u8], passphrase: &[u8]) -> io::Result<Vec<u8>> {
    ec::EcKey::private_key_from_pem_passphrase(pem, passphrase)
        .and_then(pkey::PKey::from_ec_key)
        .and_then(|it| it.private_key_to_pem_pkcs8())
        .map_err(errors::invalid_data)
}

// To PEM-encoded PKCS#1
fn private_key_from_rsa(pem: &[u8], passphrase: &[u8]) -> io::Result<Vec<u8>> {
    rsa::Rsa::private_key_from_pem_passphrase(pem, passphrase)
        .and_then(|it| it.private_key_to_pem())
        .map_err(errors::invalid_data)
}

pub(crate) fn for_verifying(key: Vec<u8>, algorithm: jwt::Algorithm) -> io::Result<Vec<u8>> {
    match KeyType::from(algorithm) {
        KeyType::Ec => public_key_from_ec(&key),
        KeyType::Rsa => public_key_from_rsa(&key),
        KeyType::Secret => Ok(key),
    }
}

fn public_key_from_ec(pem: &[u8]) -> io::Result<Vec<u8>> {
    ec::EcKey::private_key_from_pem(pem)
        .and_then(pkey::PKey::from_ec_key)
        .and_then(|it| it.public_key_to_pem())
        .map_err(errors::invalid_data)
}

fn public_key_from_rsa(pem: &[u8]) -> io::Result<Vec<u8>> {
    rsa::Rsa::private_key_from_pem(pem)
        .and_then(pkey::PKey::from_rsa)
        .and_then(|it| it.public_key_to_pem())
        .map_err(errors::invalid_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSPHRASE: &str = "JamesBond";

    const EC_ENCRYPTED: &str = include_str!("../key-for-test/ec-encrypted.pem");
    const EC_PRIVATE: &str = include_str!("../key-for-test/ec-private.pem");
    const EC_PUBLIC: &str = include_str!("../key-for-test/ec-public.pem");

    const RSA_ENCRYPTED: &str = include_str!("../key-for-test/rsa-encrypted.pem");
    const RSA_PRIVATE: &str = include_str!("../key-for-test/rsa-private.pem");
    const RSA_PUBLIC: &str = include_str!("../key-for-test/rsa-public.pem");

    #[test]
    fn test_private_key_from_ec() {
        assert_eq!(
            EC_PRIVATE.as_bytes(),
            private_key_from_ec(EC_ENCRYPTED.as_bytes(), PASSPHRASE.as_bytes()).unwrap()
        );
    }

    #[test]
    fn test_private_key_from_rsa() {
        let key = private_key_from_rsa(RSA_ENCRYPTED.as_bytes(), PASSPHRASE.as_bytes()).unwrap();
        let key = String::from_utf8(key).unwrap();
        assert_eq!(RSA_PRIVATE, key);
    }

    #[test]
    fn test_public_key_from_ec() {
        assert_eq!(
            EC_PUBLIC.as_bytes(),
            public_key_from_ec(EC_PRIVATE.as_bytes()).unwrap()
        );
    }

    #[test]
    fn test_public_key_from_rsa() {
        assert_eq!(
            RSA_PUBLIC.as_bytes(),
            public_key_from_rsa(RSA_PRIVATE.as_bytes()).unwrap()
        );
    }
}
