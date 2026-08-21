use activitystreams_vocabulary::impl_default;
use serde::{Deserialize, Serialize};

/// Represents the SQL key type variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "key_type")]
pub enum KeyType {
    Ecdsa256,
    Ecdsa384,
    Ed25519,
    Bls12,
    Sm2,
    Rsa2048,
    Rsa3072,
    Rsa4096,
}

impl KeyType {
    /// Creates a new [KeyType].
    pub const fn new() -> Self {
        Self::Ed25519
    }
}

impl_default!(KeyType);
