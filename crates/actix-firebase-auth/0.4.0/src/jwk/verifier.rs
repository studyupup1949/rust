use std::env;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
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

    pub fn verify<T: DeserializeOwned>(
        &self,
        token: &str,
    ) -> VerificationResult<T> {
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

        let header = jsonwebtoken::decode_header(token)
            .map_err(|_| VerificationError::InvalidSignature)?;

        if header.alg != Algorithm::RS256 {
            return Err(VerificationError::InvalidKeyAlgorithm);
        }

        let Some(kid) = header.kid else {
            return Err(VerificationError::NoKidHeader);
        };

        let Some(public_key) = self.keys.keys.iter().find(|v| v.kid == kid)
        else {
            return Err(VerificationError::NoMatchingKid);
        };

        let decoding_key =
            DecodingKey::from_rsa_components(&public_key.n, &public_key.e)
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
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use openssl::rsa::Rsa;
    use serde::{Deserialize, Serialize};
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        sync::{LazyLock, Mutex},
        time::Duration,
    };

    // Use a static Mutex to synchronize key creation and cleanup across tests
    static KEY_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    // We'll generate keys in a temp directory per test to avoid collisions
    fn create_temp_test_dir(test_name: &str) -> PathBuf {
        let base = std::env::temp_dir()
            .join("my_project_test_keys")
            .join(test_name);
        if base.exists() {
            let _ = fs::remove_dir_all(&base);
        }
        fs::create_dir_all(&base)
            .expect("Failed to create temp test directory");
        base
    }

    // This ensures that keys exist in the given directory
    fn ensure_test_keys_exist(dir: &Path) {
        let private_key = dir.join("private.pem");
        let public_key = dir.join("public.pem");

        if private_key.exists() && public_key.exists() {
            return;
        }

        let _guard = KEY_MUTEX.lock().unwrap();

        // Double-check after locking
        if private_key.exists() && public_key.exists() {
            return;
        }

        let status = Command::new("openssl")
            .args(["genrsa", "-out", private_key.to_str().unwrap(), "2048"])
            .status()
            .expect("Failed to run openssl genrsa");
        assert!(status.success());

        let status = Command::new("openssl")
            .args([
                "rsa",
                "-in",
                private_key.to_str().unwrap(),
                "-pubout",
                "-out",
                public_key.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to run openssl rsa -pubout");
        assert!(status.success());
    }

    // Load keys from specified directory
    fn load_rsa_keys(dir: &Path) -> (String, String, EncodingKey) {
        let private_pem_path = dir.join("private.pem");
        let public_pem_path = dir.join("public.pem");

        let private_pem = fs::read_to_string(&private_pem_path)
            .expect("Failed to read private.pem");
        let public_pem = fs::read_to_string(&public_pem_path)
            .expect("Failed to read public.pem");

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes())
            .expect("Failed to create encoding key");

        let rsa_pub = Rsa::public_key_from_pem(public_pem.as_bytes())
            .expect("Failed to parse public key PEM");

        let n_bytes = rsa_pub.n().to_vec();
        let e_bytes = rsa_pub.e().to_vec();

        let n = BASE64_URL_SAFE_NO_PAD.encode(&n_bytes);
        let e = BASE64_URL_SAFE_NO_PAD.encode(&e_bytes);

        (n, e, encoding_key)
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct DummyClaims {
        sub: String,
        aud: String,
        iss: String,
        exp: u64,
        iat: u64,
    }

    fn now_as_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn valid_claims() -> DummyClaims {
        let now = now_as_secs();
        DummyClaims {
            sub: "user123".into(),
            aud: "test-project-id".into(),
            iss: "https://securetoken.google.com/test-project-id".into(),
            exp: now + 3600_u64,
            iat: now,
        }
    }

    fn jwk_keys_with_kid(kid: &str, n: &str, e: &str, alg: &str) -> JwkKeys {
        JwkKeys {
            keys: vec![JwkKey {
                kty: "RSA".into(),
                alg: alg.into(),
                kid: kid.into(),
                n: n.into(),
                e: e.into(),
            }],
            max_age: Duration::from_secs(3600),
        }
    }

    #[actix_rt::test]
    async fn returns_error_on_invalid_jwt_format() {
        // Ensure emulator is OFF for this test to force full verification path
        //
        // SAFETY: `set_var` and `remove_var` are wrapped in `unsafe` because this test
        // modifies a global environment variable that may be accessed concurrently by
        // other async tests. This is acceptable here because the test suite is expected
        // to run in isolation (e.g., via `cargo test -- --test-threads=1`), or this test
        // is known to be the only one touching FIREBASE_AUTH_EMULATOR_HOST.
        #[expect(unsafe_code)]
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
            "Expected InvalidToken error, got {result:?}"
        );

        // Restore environment to avoid side effects
        #[expect(unsafe_code)]
        #[expect(clippy::undocumented_unsafe_blocks)]
        unsafe {
            env::remove_var("FIREBASE_AUTH_EMULATOR_HOST");
        }
    }

    #[actix_rt::test]
    async fn fails_on_missing_kid() {
        let test_dir = create_temp_test_dir("fails_on_missing_kid");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let header = Header::new(Algorithm::RS256);
        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test",
            jwk_keys_with_kid("some-valid-kid", &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);
        assert!(
            matches!(result, Err(VerificationError::NoKidHeader)),
            "Expected NoKidHeader error, got {result:?}"
        );
    }

    #[actix_rt::test]
    async fn fails_on_invalid_kid() {
        let test_dir = create_temp_test_dir("fails_on_invalid_kid");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("mismatch-kid".into());
        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test",
            jwk_keys_with_kid("a-different-kid", &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);
        assert!(
            matches!(result, Err(VerificationError::NoMatchingKid)),
            "Expected NoMatchingKid error, got {result:?}"
        );
    }
    #[actix_rt::test]
    async fn fails_on_wrong_algorithm() {
        let test_dir = create_temp_test_dir("fails_on_wrong_algorithm");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let kid = "test-kid";

        // Token is RS256
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let token = encode(&header, &valid_claims(), &encoding_key).unwrap();

        // Create JWK keys where the algorithm for 'test-kid' is claimed to be HS256, but the token is RS256.
        // JWK says HS256 for this kid
        let verifier =
            JwkVerifier::new("test", jwk_keys_with_kid(kid, &n, &e, "HS256"));
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            result.is_err(), // jsonwebtoken::errors::ErrorKind::InvalidToken
            "Expected InvalidKeyAlgorithm error, got {result:?}"
        );
    }

    #[actix_rt::test]
    async fn fails_on_expired_token() {
        let test_dir = create_temp_test_dir("fails_on_expired_token");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut expired_claims = valid_claims();

        // Set expiration to 100 seconds in the past
        expired_claims.exp = now_as_secs() - 100;

        let token = encode(&header, &expired_claims, &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test-project-id",
            jwk_keys_with_kid(kid, &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Expired token typically falls under InvalidToken due to validation failure
            "Expected InvalidToken for expired token, got {result:?}"
        );
    }

    #[actix_rt::test]
    async fn fails_on_incorrect_audience() {
        let test_dir = create_temp_test_dir("fails_on_incorrect_audience");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut claims_with_wrong_aud = valid_claims();
        claims_with_wrong_aud.aud = "wrong-project-id".into();

        let token =
            encode(&header, &claims_with_wrong_aud, &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test-project-id", // Correct audience for verifier
            jwk_keys_with_kid(kid, &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Incorrect aud typically falls under InvalidToken due to validation failure
            "Expected InvalidToken for incorrect audience, got {result:?}"
        );
    }

    #[actix_rt::test]
    async fn fails_on_incorrect_issuer() {
        let test_dir = create_temp_test_dir("fails_on_incorrect_issuer");
        ensure_test_keys_exist(&test_dir);
        let (n, e, encoding_key) = load_rsa_keys(&test_dir);

        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let mut claims_with_wrong_iss = valid_claims();
        claims_with_wrong_iss.iss = "https://wrong-issuer.com".into();

        let token =
            encode(&header, &claims_with_wrong_iss, &encoding_key).unwrap();

        let verifier = JwkVerifier::new(
            "test-project-id",
            jwk_keys_with_kid(kid, &n, &e, "RS256"),
        );
        let result: Result<DummyClaims, _> = verifier.verify(&token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)),
            "Expected InvalidToken for incorrect issuer, got {result:?}"
        );
    }

    #[actix_rt::test]
    async fn fails_on_invalid_signature() {
        let kid = "test-key-id";
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.into());

        let claims = valid_claims();

        // Create a different key to sign an *incorrect* token
        let test_dir = create_temp_test_dir("fails_on_invalid_signature_other");
        ensure_test_keys_exist(&test_dir);
        let (n, e, wrong_encoding_key) = load_rsa_keys(&test_dir);

        let wrong_token =
            encode(&header, &claims, &wrong_encoding_key).unwrap();

        // Verifier has the correct key
        let verifier = JwkVerifier::new(
            "test-project-id",
            jwk_keys_with_kid(kid, &n, &e, "RS256"),
        );

        // Try to verify the incorrectly signed token
        let result: Result<DummyClaims, _> = verifier.verify(&wrong_token);

        assert!(
            matches!(result, Err(VerificationError::InvalidToken)), // Invalid signature maps to InvalidToken
            "Expected InvalidToken for invalid signature, got {result:?}"
        );
    }
}
