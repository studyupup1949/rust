use activitystreams_vocabulary::{
    Iri, Key, PublicKeyPem, field_access, impl_default, impl_display,
};

use serde::{Deserialize, Serialize};

use crate::crypto::{KeyType, PublicKey};
use crate::{Error, Result};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PemPublicKey {
    id: Iri,
    owner: Iri,
    public_key_pem: PublicKey,
}

impl PemPublicKey {
    /// Creates a new [PemPublicKey].
    pub fn new() -> Self {
        Self {
            id: Iri::new(),
            owner: Iri::new(),
            public_key_pem: PublicKey::new(),
        }
    }
}

field_access! {
    PemPublicKey {
        /// Represents the key ID used to fetch the key from a remote server.
        id: as_ref { Iri },
        /// Represents the owner ID used to fetch the actor record from a remote server.
        owner: as_ref { Iri },
        /// Represents the public key to be encoded to PEM ASN.1 DER for network transmission.
        public_key_pem: as_ref { PublicKey },
    }
}

impl_default!(PemPublicKey);
impl_display!(PemPublicKey, json);

impl TryFrom<Key> for PemPublicKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for PemPublicKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        let id = val.id().cloned().ok_or(Error::crypto("pem: missing ID"))?;

        let owner = val
            .owner()
            .cloned()
            .ok_or(Error::crypto("pem: missing owner"))
            .or_else(|_| {
                val.controller()
                    .cloned()
                    .ok_or(Error::crypto("pem: missing controller"))
            })?;

        let pem = val
            .public_key_pem()
            .ok_or(Error::crypto("pem: missing public key"))
            .map(|k| k.to_string())?;

        PublicKey::from_pem(KeyType::Ed25519, &pem)
            .or_else(|_| PublicKey::from_pem(KeyType::Ecdsa256, &pem))
            .or_else(|_| PublicKey::from_pem(KeyType::Ecdsa384, &pem))
            .or_else(|_| PublicKey::from_pem(KeyType::Rsa2048, &pem))
            .map_err(|err| Error::crypto(format!("pem: invalid key: {err}")))
            .map(|public_key_pem| Self {
                id,
                owner,
                public_key_pem,
            })
    }
}

impl TryFrom<PemPublicKey> for Key {
    type Error = Error;

    fn try_from(val: PemPublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&PemPublicKey> for Key {
    type Error = Error;

    fn try_from(val: &PemPublicKey) -> Result<Self> {
        val.public_key_pem().to_der().map(|der| {
            Self::new()
                .with_id(val.id().clone())
                .with_owner(val.owner().clone())
                .with_public_key_pem(PublicKeyPem::new().with_key(der))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Ed25519PublicKey;

    const ED25519_PUBKEY_BYTES: [u8; 32] = [
        0x7d, 0xb0, 0x56, 0xf4, 0xe, 0xa7, 0x10, 0xb6, 0x80, 0xa7, 0x6a, 0xd7, 0x26, 0xfd, 0xbd,
        0x3e, 0x70, 0xea, 0xd1, 0xd6, 0x20, 0xa7, 0x74, 0xdb, 0x4b, 0x1a, 0x2a, 0x48, 0xa0, 0xce,
        0xa3, 0xe5,
    ];

    #[test]
    fn test_pem_key() {
        let pubkey =
            PublicKey::Ed25519(Ed25519PublicKey::from_bytes(&ED25519_PUBKEY_BYTES).unwrap());
        let pubkey_json = pubkey.to_string();

        let owner_id = Iri::try_from("https://example.dev/api/v1/persons/test_user").unwrap();
        let key_id = Iri::try_from(format!("{owner_id}?keyId=test-key-ed25519")).unwrap();

        let pemkey_json = format!(
            r#"{{
  "id": "{key_id}",
  "owner": "{owner_id}",
  "publicKeyPem": {pubkey_json}
}}"#
        );

        let pemkey = PemPublicKey::new()
            .with_id(key_id)
            .with_owner(owner_id)
            .with_public_key_pem(pubkey);

        assert_eq!(serde_json::to_string_pretty(&pemkey).unwrap(), pemkey_json);
        assert_eq!(
            serde_json::from_str::<PemPublicKey>(&pemkey_json).unwrap(),
            pemkey
        );
    }
}
