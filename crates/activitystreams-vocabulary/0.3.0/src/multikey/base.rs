use base64::{Base64UrlUnpadded, Encoding};
use serde::{de, ser};

use crate::{
    Error, MultibaseHeader, MultikeyPublicKey, Result, field_access, impl_default, impl_display,
};

/// Represents a [Controlled Identifier `Multibase`](https://www.w3.org/TR/cid-1.0/#Multibase) encoding of key material.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultibasePublicKey {
    header: MultibaseHeader,
    key: MultikeyPublicKey,
}

impl MultibasePublicKey {
    /// Creates a new [MultibasePublicKey].
    pub const fn new() -> Self {
        Self {
            header: MultibaseHeader::new(),
            key: MultikeyPublicKey::new(),
        }
    }

    /// Encodes the [MultibasePublicKey].
    pub fn encode(&self) -> String {
        let header = self.header.as_str();
        let encode_input: Vec<u8> = Vec::from(&self.key);

        let encoded = match self.header {
            MultibaseHeader::Base58Btc => base58::encode(encode_input.as_slice()),
            MultibaseHeader::Base64UrlNoPad => {
                base64::Base64UrlUnpadded::encode_string(encode_input.as_slice())
            }
        };

        format!("{header}{encoded}")
    }

    /// Attempts to decode a [MultibasePublicKey] from the input string.
    pub fn decode(val: &str) -> Result<Self> {
        let header = MultibaseHeader::try_from(val.as_bytes())?;
        let rem = val.get(1..).ok_or(Error::multikey("missing key data"))?;

        let key_res = match header {
            MultibaseHeader::Base58Btc => base58::decode(rem)
                .map_err(|err| Error::multikey(format!("base-58-btc decoding error: {err}"))),
            MultibaseHeader::Base64UrlNoPad => Base64UrlUnpadded::decode_vec(rem).map_err(|err| {
                Error::multikey(format!("base-64-url-no-pad deooding error: {err}"))
            }),
        };

        key_res
            .and_then(MultikeyPublicKey::try_from)
            .map(|key| Self { header, key })
    }
}

field_access! {
    MultibasePublicKey {
        header: MultibaseHeader,
    }
}

field_access! {
    MultibasePublicKey {
        key: as_ref { MultikeyPublicKey },
    }
}

impl_default!(MultibasePublicKey);
impl_display!(MultibasePublicKey, json);

impl ser::Serialize for MultibasePublicKey {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.encode().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for MultibasePublicKey {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            Self::decode(s)
                .map_err(|err| de::Error::custom(format!("multibase public key decoding: {err}")))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ecdsa256() {
        let key_encoded = "zDnbAEaKPerXAXsd14HqECVZnUdGVzJea7UzFgYUE9tVqs6UZ";
        let key = MultikeyPublicKey::Ecdsa256([
            0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa,
            0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b,
            0x42, 0x87, 0xa6, 0x02,
        ]);

        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(key.clone());

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );

        let key_encoded = "ugCRmar4IkDXtSkV5WYn6cgoO4MSFSehFI0hcVvHIG0KHpgI";
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(key);

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );
    }

    #[test]
    fn test_valid_ecdsa384() {
        let key_encoded = "z82M31WPNPjtjqzJuyXtujwPFsJNzjkD3LhTo2XenACkRwXYjqjFGVkDuWszV5awWUaVtFU";
        let key = MultikeyPublicKey::Ecdsa384([
            0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa,
            0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b,
            0x42, 0x87, 0xa6, 0x02, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
            0x1b, 0x42, 0x87, 0xa6, 0x02, 0x03,
        ]);

        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(key.clone());

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );

        let key_encoded = "ugSRmar4IkDXtSkV5WYn6cgoO4MSFSehFI0hcVvHIG0KHpgKFSehFI0hcVvHIG0KHpgID";
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(key);

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );
    }

    #[test]
    fn test_valid_ed25519() {
        let key_encoded = "z6MkmM42vxfqZQsv4ehtTjFFxQ4sQKS2w6WR7emozFAn5cxu";
        let key = MultikeyPublicKey::Ed25519([
            0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa,
            0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b,
            0x42, 0x87, 0xa6,
        ]);
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(key.clone());

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );

        let key_encoded = "u7QFmar4IkDXtSkV5WYn6cgoO4MSFSehFI0hcVvHIG0KHpg";
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(key);

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );
    }

    #[test]
    fn test_valid_bls12() {
        let key_encoded = "zUC6r7o2yWNEKdzKSp3VTxGWvY4BYAdyxF2BQTm6wrnwYSCZyErr3vxwQHpwxTXiy2NMtW51s7Fc4ufLJxogyRLbhUmUGZfW2K5aQnBvaFyrrFDcpK5wua6F8do5SmqWAL4kJyB";
        let key = MultikeyPublicKey::Bls12([
            0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa,
            0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b,
            0x42, 0x87, 0xa6, 0x02, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
            0x1b, 0x42, 0x87, 0xa6, 0x02, 0x03, 0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a,
            0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa, 0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23,
            0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b, 0x42, 0x87, 0xa6, 0x02, 0x85, 0x49, 0xe8, 0x45,
            0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b, 0x42, 0x87, 0xa6,
        ]);

        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(key.clone());

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );

        let key_encoded = "u6wFmar4IkDXtSkV5WYn6cgoO4MSFSehFI0hcVvHIG0KHpgKFSehFI0hcVvHIG0KHpgIDZmq-CJA17UpFeVmJ-nIKDuDEhUnoRSNIXFbxyBtCh6YChUnoRSNIXFbxyBtCh6Y";
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(key);

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );
    }

    #[test]
    fn test_valid_sm2() {
        let key_encoded = "zEPK7nXZoxbmXUE3nVV7NTuALMY8GKQRhF8K3NeL2w24mZ2uw";
        let key = MultikeyPublicKey::Sm2([
            0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72, 0xa,
            0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8, 0x1b,
            0x42, 0x87, 0xa6, 0x02,
        ]);

        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(key.clone());

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );

        let key_encoded = "uhiRmar4IkDXtSkV5WYn6cgoO4MSFSehFI0hcVvHIG0KHpgI";
        let multikey = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(key);

        assert_eq!(multikey.encode(), key_encoded);
        assert_eq!(
            MultibasePublicKey::decode(key_encoded).as_ref(),
            Ok(&multikey)
        );
    }
}
