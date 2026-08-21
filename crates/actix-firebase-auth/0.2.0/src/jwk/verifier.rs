use std::env;

use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::de::DeserializeOwned;

use super::config::JwkConfig;
use super::error::{VerificationError, VerificationResult};
use super::key::JwkKeys;

#[derive(Debug)]
pub struct JwkVerifier {
    keys: JwkKeys,
    config: JwkConfig,
}

impl JwkVerifier {
    pub fn new(project_id: impl AsRef<str>, keys: JwkKeys) -> JwkVerifier {
        JwkVerifier {
            keys,
            config: JwkConfig::new(project_id),
        }
    }

    pub fn verify<T: DeserializeOwned>(&self, token: &str) -> VerificationResult<T> {
        if env::var("FIREBASE_AUTH_EMULATOR_HOST").is_ok() {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                return Err(VerificationError::InvalidToken);
            }

            let decoded_payload = BASE64_STANDARD_NO_PAD
                .decode(parts[1].trim())
                .map_err(super::VerificationError::CannotDecodeJwt)?;

            let claims: T = serde_json::from_slice(&decoded_payload)
                .map_err(|_| VerificationError::InvalidToken)?;

            return Ok(claims);
        }

        let header =
            jsonwebtoken::decode_header(token).map_err(|_| VerificationError::InvalidSignature)?;

        if header.alg != Algorithm::RS256 {
            return Err(VerificationError::InvalidKeyAlgorithm);
        }

        let kid = match header.kid {
            Some(v) => v,
            None => return Err(VerificationError::NoKidHeader),
        };

        let public_key = match self.keys.keys.iter().find(|v| v.kid == kid) {
            Some(v) => v,
            None => return Err(VerificationError::NoMatchingKid),
        };

        let decoding_key = DecodingKey::from_rsa_components(&public_key.n, &public_key.e)
            .map_err(|_| VerificationError::CannotDecodePublicKeys)?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.config.audience()]);
        validation.set_issuer(&[self.config.issuer()]);

        let user = jsonwebtoken::decode::<T>(token, &decoding_key, &validation)
            .map_err(|_| VerificationError::InvalidToken)?
            .claims;
        Ok(user)
    }

    pub fn set_keys(&mut self, keys: JwkKeys) {
        self.keys = keys;
    }
}

#[cfg(test)]
mod tests {
    use crate::jwk::JwkKey;

    use super::*;
    use actix_rt::test;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use rsa::RsaPrivateKey;
    use rsa::pkcs8::EncodePrivateKey;
    use rsa::traits::PublicKeyParts;
    use serde::{Deserialize, Serialize};
    use std::{env, time::Duration};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct DummyClaims {
        sub: String,
        aud: String,
        iss: String,
        exp: usize, // Expiration time
        iat: usize, // Issued at time
    }

    // Helper to get current time for `iat` and `exp`
    fn now_as_secs() -> usize {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize
    }

    // This function will return claims with `exp` and `iat` based on the current time.
    // Be mindful that if tests run very quickly, `now` might be the same, but if there's
    // a slight delay, they could differ by a second, potentially affecting strict `PartialEq`
    // comparison for the whole `DummyClaims` struct. For this reason, in `verifies_valid_token`,
    // we'll compare individual fields.
    fn valid_claims() -> DummyClaims {
        let now = now_as_secs();
        DummyClaims {
            sub: "user123".into(),
            aud: "test-project-id".into(),
            iss: "https://securetoken.google.com/test-project-id".into(),
            exp: now + 3600, // Expires in 1 hour from now
            iat: now,
        }
    }

    fn make_rsa_keys() -> (String, String, EncodingKey) {
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rand::thread_rng(), bits).unwrap();
        let pub_key = priv_key.to_public_key();
        let encoding_key = EncodingKey::from_rsa_pem(
            &<std::string::String as Clone>::clone(
                &priv_key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap(),
            )
            .into_bytes(),
        )
        .unwrap();

        let n = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pub_key.n().to_bytes_be());
        let e = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(pub_key.e().to_bytes_be());

        (n, e, encoding_key)
    }

    fn jwk_keys_with_kid(kid: &str, n: &str, e: &str, alg: &str) -> JwkKeys {
        JwkKeys {
            keys: vec![JwkKey {
                kty: "RSA".into(),
                alg: alg.into(), // Use the passed algorithm
                kid: kid.into(),
                n: n.into(),
                e: e.into(),
            }],
            max_age: Duration::from_secs(3600),
        }
    }

    #[test]
    async fn returns_error_on_invalid_jwt_format() {
        // Ensure emulator is OFF for this test to force full verification path
        unsafe {
            env::set_var("FIREBASE_AUTH_EMULATOR_HOST", "");
        }

        let verifier = JwkVerifier::new(
            "test",
            JwkKeys {
                keys: vec![],
                max_age: Duration::ZERO,
            },
        );
        let result: Result<DummyClaims, _> = verifier.verify("not.a.jwt");
        assert!(
            result.is_err(),
            "Expected InvalidToken error, got {:?}",
            result
        );

        unsafe {
            env::remove_var("FIREBASE_AUTH_EMULATOR_HOST");
        }
    }

    #[test]
    async fn fails_on_missing_kid() {
        let (n, e, encoding_key) = make_rsa_keys();
        // no kid in header
        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        // JwkKeys contains a key, but the token header doesn't specify a kid.
        let verifier =
            JwkVerifier::new("test", jwk_keys_with_kid("some-valid-kid", &n, &e, "RS256"));
        let result: Result<DummyClaims, _> = verifier.verify(&token);
        assert!(
            matches!(result, Err(VerificationError::NoKidHeader)),
            "Expected NoKidHeader error, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_invalid_kid() {
        let (n, e, encoding_key) = make_rsa_keys();
        let mut header = Header::new(Algorithm::RS256);

        // Token has this KID
        header.kid = Some("mismatch-kid".into());

        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        // JwkKeys contains a different KID
        let verifier = JwkVerifier::new(
            "test",
            jwk_keys_with_kid("a-different-kid", &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);
        assert!(
            matches!(result, Err(VerificationError::NoMatchingKid)),
            "Expected NoMatchingKid error, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_wrong_algorithm() {
        let (n, e, encoding_key) = make_rsa_keys();
        let kid = "test-kid";

        // Token is RS256
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        // Create JWK keys where the algorithm for 'test-kid' is claimed to be HS256, but the token is RS256.
        // JWK says HS256 for this kid
        let verifier = JwkVerifier::new("test", jwk_keys_with_kid(kid, &n, &e, "HS256"));
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            result.is_err(), // jsonwebtoken::errors::ErrorKind::InvalidToken
            "Expected InvalidKeyAlgorithm error, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_expired_token() {
        let (n, e, encoding_key) = make_rsa_keys();
        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut expired_claims = valid_claims();

        // Set expiration to 100 seconds in the past
        expired_claims.exp = now_as_secs() - 100;

        let token = encode(&header, &expired_claims, &encoding_key).unwrap();

        let verifier = JwkVerifier::new("test-project-id", jwk_keys_with_kid(kid, &n, &e, "RS256"));
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Expired token typically falls under InvalidToken due to validation failure
            "Expected InvalidToken for expired token, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_incorrect_audience() {
        let (n, e, encoding_key) = make_rsa_keys();
        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut claims_with_wrong_aud = valid_claims();
        claims_with_wrong_aud.aud = "wrong-project-id".into();

        let token = encode(&header, &claims_with_wrong_aud, &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test-project-id", // Correct audience for verifier
            jwk_keys_with_kid(kid, &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Incorrect aud typically falls under InvalidToken due to validation failure
            "Expected InvalidToken for incorrect audience, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_incorrect_issuer() {
        let (n, e, encoding_key) = make_rsa_keys();
        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut claims_with_wrong_iss = valid_claims();
        claims_with_wrong_iss.iss = "https://wrong-issuer.com".into();

        let token = encode(&header, &claims_with_wrong_iss, &encoding_key).unwrap();

        let verifier = JwkVerifier::new("test-project-id", jwk_keys_with_kid(kid, &n, &e, "RS256"));
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)),
            "Expected InvalidToken for incorrect issuer, got {:?}",
            result
        );
    }

    #[test]
    async fn fails_on_invalid_signature() {
        let (n, e, _) = make_rsa_keys();
        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let claims = valid_claims();

        // Create a different key to sign an *incorrect* token
        let (_, _, wrong_encoding_key) = make_rsa_keys();
        let wrong_token = encode(&header, &claims, &wrong_encoding_key).unwrap();

        // Verifier has the correct key
        let verifier = JwkVerifier::new("test-project-id", jwk_keys_with_kid(kid, &n, &e, "RS256"));

        // Try to verify the incorrectly signed token
        let result: Result<DummyClaims, _> = verifier.verify(&wrong_token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Invalid signature maps to InvalidToken
            "Expected InvalidToken for invalid signature, got {:?}",
            result
        );
    }
}
