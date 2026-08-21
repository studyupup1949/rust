use base64::{Base64UrlUnpadded, Encoding};
use serde::{de, ser};

use crate::{Error, MultibaseHeader, Result, field_access, impl_default, impl_display};

/// Represents a [Controlled Identifier `Multibase`](https://www.w3.org/TR/cid-1.0/#Multibase) encoding of a [DataIntegrityProof](crate::DataIntegrityProof).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultibaseData {
    header: MultibaseHeader,
    data: Vec<u8>,
}

impl MultibaseData {
    /// Creates a new [MultibaseData].
    pub const fn new() -> Self {
        Self {
            header: MultibaseHeader::new(),
            data: Vec::new(),
        }
    }

    /// Encodes the [MultibaseData].
    pub fn encode(&self) -> String {
        let header = self.header.as_str();

        let encoded = match self.header {
            MultibaseHeader::Base58Btc => base58::encode(self.data.as_ref()),
            MultibaseHeader::Base64UrlNoPad => {
                base64::Base64UrlUnpadded::encode_string(self.data.as_ref())
            }
        };

        format!("{header}{encoded}")
    }

    /// Attempts to decode a [MultibaseData] from the input string.
    pub fn decode(val: &str) -> Result<Self> {
        let header = MultibaseHeader::try_from(val.as_bytes())?;
        let rem = val.get(1..).ok_or(Error::multikey("missing key data"))?;

        let data_res = match header {
            MultibaseHeader::Base58Btc => base58::decode(rem)
                .map_err(|err| Error::multikey(format!("base-58-btc decoding error: {err}"))),
            MultibaseHeader::Base64UrlNoPad => Base64UrlUnpadded::decode_vec(rem).map_err(|err| {
                Error::multikey(format!("base-64-url-no-pad deooding error: {err}"))
            }),
        };

        data_res.map(|data| Self { header, data })
    }
}

field_access! {
    MultibaseData {
        header: MultibaseHeader,
    }
}

field_access! {
    MultibaseData {
        data: as_ref { &[u8], Vec<u8> },
    }
}

impl_default!(MultibaseData);
impl_display!(MultibaseData, json);

impl ser::Serialize for MultibaseData {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.encode().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for MultibaseData {
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
    fn test_valid() {
        let data_encoded = "z7tnzLiRQDsPSx9sBnAHR7JWsakABXDG4Rdrt9yCmAQBX";
        let data = [
            0x66, 0x6a, 0xbe, 0x08, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72,
            0x0a, 0x0e, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
            0x1b, 0x42, 0x87, 0xa6,
        ];

        let multibase = MultibaseData::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_data(data);

        assert_eq!(multibase.encode(), data_encoded);
        assert_eq!(MultibaseData::decode(data_encoded).as_ref(), Ok(&multibase));

        let data_encoded = "uZmq-CJA17UpFeVmJ-nIKDuDEhUnoRSNIXFbxyBtCh6Y";
        let multibase = MultibaseData::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_data(data);

        assert_eq!(multibase.encode(), data_encoded);
        assert_eq!(MultibaseData::decode(data_encoded).as_ref(), Ok(&multibase));
    }
}
