use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::sync::{Arc, RwLock};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
use ed25519_dalek::pkcs8::{DecodePrivateKey, EncodePublicKey};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, decode_header, encode,
};
use serde::{Deserialize, Serialize};

pub use jsonwebtoken::TokenData;

/// Error type for JWKS operations
#[derive(Debug)]
pub enum JwksError {
    /// Key generation error
    KeyGenerationError(String),
    /// Key loading error
    KeyLoadingError(String),
    /// Token generation error
    TokenGenerationError(jsonwebtoken::errors::Error),
    /// Token validation error
    TokenValidationError(jsonwebtoken::errors::Error),
    /// Serialization error
    SerializationError(serde_json::Error),
    /// Other error
    Other(String),
    /// IO error
    IoError(std::io::Error),
}

impl fmt::Display for JwksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyGenerationError(e) => write!(f, "Key generation error: {}", e),
            Self::KeyLoadingError(e) => write!(f, "Key loading error: {}", e),
            Self::TokenGenerationError(e) => write!(f, "Token generation error: {}", e),
            Self::TokenValidationError(e) => write!(f, "Token validation error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl StdError for JwksError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::TokenGenerationError(e) => Some(e),
            Self::TokenValidationError(e) => Some(e),
            Self::SerializationError(e) => Some(e),
            Self::IoError(e) => Some(e),
            Self::KeyGenerationError(_) | Self::KeyLoadingError(_) | Self::Other(_) => None,
        }
    }
}

impl From<jsonwebtoken::errors::Error> for JwksError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::TokenValidationError(error)
    }
}

impl From<serde_json::Error> for JwksError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError(error)
    }
}

impl From<std::io::Error> for JwksError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

/// JWK representation with all necessary fields for Ed25519 keys
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwkRepresentation {
    pub kty: String, // Key type (OKP for Ed25519)
    pub crv: String, // Curve (Ed25519)
    pub x: String,   // Public key (base64url encoded)
    pub kid: String, // Key ID
    pub alg: String, // Algorithm (EdDSA)
    #[serde(rename = "use")]
    pub r#use: String, // Key usage (sig for signature)
}

/// JWKS (JSON Web Key Set) representation
#[derive(Debug, Serialize, Deserialize)]
pub struct JwksRepresentation {
    pub keys: Vec<JwkRepresentation>,
}

/// Key pair configuration for loading keys from files
#[derive(Clone, Debug)]
pub struct KeyPairConfig {
    /// Name/identifier for the key pair
    pub name: String,
    /// Path to the private key file (PEM format)
    pub private_key_path: String,
}

/// Ed25519 keypair wrapper
pub struct Ed25519KeyPair {
    /// Public key bytes
    pub public_key: [u8; 32],
    /// Private key bytes (includes public key)
    pub private_key: [u8; 64],
}

/// Loaded key pair with encoding and decoding keys for use with jsonwebtoken
#[derive(Clone)]
pub struct LoadedKeyPair {
    /// Key ID
    pub kid: String,
    /// Encoding key for signing tokens
    pub encoding_key: EncodingKey,
    /// Decoding key for verifying tokens
    pub decoding_key: DecodingKey,
    /// Public key in JWK format
    pub jwk: JwkRepresentation,
}

/// JWK Manager for handling key generation, rotation, and JWKS endpoint
pub struct JwksManager {
    /// Loaded key pairs from files
    loaded_keys: Arc<RwLock<Vec<LoadedKeyPair>>>,
    /// Current JWK set for public endpoint (derived from loaded_keys)
    jwks: Arc<RwLock<JwksRepresentation>>,
}

/// Standard claims for JWT tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (e.g., user ID)
    pub sub: String,
    /// Issued at timestamp (seconds since epoch)
    pub iat: u64,
    /// Expiration timestamp (seconds since epoch)
    pub exp: u64,
    /// Token ID
    pub jti: String,
    /// Custom claims
    #[serde(flatten)]
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl JwksManager {
    /// Create a new JWKS manager with key pairs loaded from files
    pub fn new(key_configs: Vec<KeyPairConfig>) -> Result<Self, JwksError> {
        let loaded_keys = Arc::new(RwLock::new(Vec::new()));
        let jwks = Arc::new(RwLock::new(JwksRepresentation { keys: Vec::new() }));

        let jwks_manager = Self { loaded_keys, jwks };

        // Load all keys from the configs
        for config in key_configs {
            jwks_manager.load_key_pair(config)?;
        }

        // Check if at least one key was loaded
        if jwks_manager.loaded_keys.read().unwrap().is_empty() {
            return Err(JwksError::KeyLoadingError(
                "No keys were loaded successfully".to_string(),
            ));
        }

        Ok(jwks_manager)
    }

    /// Load a key pair from file and add it to the manager
    pub fn load_key_pair(&self, config: KeyPairConfig) -> Result<String, JwksError> {
        // Read the private key PEM file
        let private_pem = fs::read_to_string(&config.private_key_path).map_err(|e| {
            JwksError::KeyLoadingError(format!(
                "Failed to read private key file {}: {}",
                config.private_key_path, e
            ))
        })?;

        // Create encoding key
        let encoding_key = EncodingKey::from_ed_pem(private_pem.as_bytes()).map_err(|e| {
            JwksError::KeyLoadingError(format!("Invalid Ed25519 private PEM: {}", e))
        })?;

        // Derive the public key from the private key
        let signing_key = SigningKey::from_pkcs8_pem(&private_pem).map_err(|e| {
            JwksError::KeyLoadingError(format!("Invalid Ed25519 private PEM: {}", e))
        })?;
        let verifying_key = signing_key.verifying_key();

        // Generate public key PEM for the decoding key
        let public_pem = verifying_key
            .to_public_key_pem(LineEnding::default())
            .map_err(|e| {
                JwksError::KeyLoadingError(format!("Failed to generate public key PEM: {}", e))
            })?;

        // Create decoding key from the derived public key
        let decoding_key = DecodingKey::from_ed_pem(public_pem.as_bytes()).map_err(|e| {
            JwksError::KeyLoadingError(format!("Invalid derived public key: {}", e))
        })?;

        // Generate a key ID using the name as a basis
        let kid = config.name.clone();

        // Extract the public key bytes (this is a bit of a hack, but jsonwebtoken doesn't expose this directly)
        // We'll use a placeholder since we don't need this for the actual operation, just for JWK representation
        let public_key_bytes = verifying_key.to_bytes();

        // Create JWK representation
        let jwk = JwkRepresentation {
            kty: "OKP".to_string(),
            crv: "Ed25519".to_string(),
            x: URL_SAFE_NO_PAD.encode(public_key_bytes),
            kid: kid.clone(),
            alg: "EdDSA".to_string(),
            r#use: "sig".to_string(),
        };

        // Create the loaded key pair
        let key_pair = LoadedKeyPair {
            kid: kid.clone(),
            encoding_key,
            decoding_key,
            jwk: jwk.clone(),
        };

        // Update key sets
        {
            let mut loaded_keys = self.loaded_keys.write().unwrap();
            loaded_keys.push(key_pair);

            let mut jwks = self.jwks.write().unwrap();
            jwks.keys.push(jwk);
        }

        Ok(kid)
    }

    /// Get the current JWKS for the /.well-known/jwks.json endpoint
    pub fn get_jwks(&self) -> Result<String, JwksError> {
        let jwks = self.jwks.read().unwrap();
        serde_json::to_string(&*jwks).map_err(JwksError::SerializationError)
    }

    /// Generate a JWT token signed with a private key
    pub fn generate_token(&self, claims: Claims) -> Result<String, JwksError> {
        // Get the first loaded key for signing
        let key_pair = {
            let loaded_keys = self.loaded_keys.read().unwrap();
            if loaded_keys.is_empty() {
                return Err(JwksError::Other("No keys available".to_string()));
            }
            loaded_keys[0].clone()
        };

        // Create a header with key ID
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(key_pair.kid);

        // Generate token using the encoding key
        encode(&header, &claims, &key_pair.encoding_key).map_err(JwksError::TokenGenerationError)
    }

    /// Generate a JWT token with a specific key ID
    pub fn generate_token_with_kid(&self, claims: Claims, kid: &str) -> Result<String, JwksError> {
        // Find the key with the specified ID
        let key_pair = {
            let loaded_keys = self.loaded_keys.read().unwrap();
            loaded_keys
                .iter()
                .find(|k| k.kid == kid)
                .cloned()
                .ok_or_else(|| JwksError::Other(format!("Key with ID {} not found", kid)))?
        };

        // Create a header with key ID
        let mut header = Header::new(Algorithm::EdDSA);
        header.kid = Some(key_pair.kid);

        // Generate token using the encoding key
        encode(&header, &claims, &key_pair.encoding_key).map_err(JwksError::TokenGenerationError)
    }

    /// Validate a JWT token
    pub fn validate_token<T>(&self, token: &str) -> Result<TokenData<T>, JwksError>
    where
        T: for<'a> Deserialize<'a>,
    {
        // Extract token header to get key ID
        let header = decode_header(token)?;

        // Verify the algorithm in the header matches what we expect
        if header.alg != Algorithm::EdDSA {
            return Err(JwksError::TokenValidationError(
                jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
                ),
            ));
        }

        // Find the key with the specified kid
        let kid = header.kid.ok_or_else(|| {
            JwksError::TokenValidationError(jsonwebtoken::errors::Error::from(
                jsonwebtoken::errors::ErrorKind::InvalidToken,
            ))
        })?;

        // Find the matching key
        let key_pair = {
            let loaded_keys = self.loaded_keys.read().unwrap();
            loaded_keys
                .iter()
                .find(|k| k.kid == kid)
                .cloned()
                .ok_or_else(|| JwksError::Other(format!("Key with ID {} not found", kid)))?
        };

        // Create validation parameters for Ed25519
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.validate_exp = true;

        // Validate the token using the decoding key
        decode::<T>(token, &key_pair.decoding_key, &validation)
            .map_err(JwksError::TokenValidationError)
    }

    /// Get the number of active keys
    pub fn key_count(&self) -> usize {
        self.loaded_keys.read().unwrap().len()
    }

    /// Get all key IDs
    pub fn list_key_ids(&self) -> Vec<String> {
        let loaded_keys = self.loaded_keys.read().unwrap();
        loaded_keys.iter().map(|k| k.kid.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_key_pair() -> KeyPairConfig {
        KeyPairConfig {
            name: "test".to_string(),
            private_key_path: "tests/test.key".to_string(),
        }
    }

    #[test]
    fn jwks_manager_creation() {
        let config = get_test_key_pair();
        let manager = JwksManager::new(vec![config]).unwrap();
        assert_eq!(manager.key_count(), 1);
    }

    #[test]
    fn jwks_endpoint() {
        let config = get_test_key_pair();
        let manager = JwksManager::new(vec![config]).unwrap();
        let jwks_json = manager.get_jwks().unwrap();
        let jwks: JwksRepresentation = serde_json::from_str(&jwks_json).unwrap();
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kty, "OKP");
        assert_eq!(jwks.keys[0].crv, "Ed25519");
        assert_eq!(jwks.keys[0].alg, "EdDSA");
    }

    #[test]
    fn jwks_initialization() {
        // Test that we can create the manager and it has the expected initial state
        let config = get_test_key_pair();
        let manager = JwksManager::new(vec![config]).unwrap();
        assert_eq!(manager.key_count(), 1);

        // Check JWKS representation
        let jwks_json = manager.get_jwks().unwrap();
        let jwks: JwksRepresentation = serde_json::from_str(&jwks_json).unwrap();

        // Verify one key exists
        assert_eq!(jwks.keys.len(), 1);

        // The first key should have Ed25519 parameters
        assert_eq!(jwks.keys[0].kty, "OKP");
        assert_eq!(jwks.keys[0].crv, "Ed25519");
        assert_eq!(jwks.keys[0].alg, "EdDSA");
        assert_eq!(jwks.keys[0].r#use, "sig");

        // The key ID should not be empty
        assert!(!jwks.keys[0].kid.is_empty());

        // Public key should be base64url encoded and not empty
        assert!(!jwks.keys[0].x.is_empty());
    }
}
