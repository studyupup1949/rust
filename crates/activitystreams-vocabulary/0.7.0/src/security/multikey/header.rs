use crate::{Error, Result, impl_default, impl_display};

/// Represents the encoding header used to indicate the base alphabet, e.g. `base58-btc`, `base-64-url-no-pad`, etc.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MultibaseHeader {
    Base58Btc,
    Base64UrlNoPad,
}

impl MultibaseHeader {
    pub const BASE58_BTC: &str = "z";
    pub const BASE64_URL_NO_PAD: &str = "u";
    pub const BASE58_BTC_BYTE: u8 = b'z';
    pub const BASE64_URL_NO_PAD_BYTE: u8 = b'u';

    /// Creates a new [MultibaseHeader].
    pub const fn new() -> Self {
        Self::Base58Btc
    }

    /// Gets whether the [MultibaseHeader] is for the `base-58-btc` encoding.
    pub const fn is_base58_btc(&self) -> bool {
        matches!(self, Self::Base58Btc)
    }

    /// Gets whether the [MultibaseHeader] is for the `base-64-url-no-pad` encoding.
    pub const fn is_base64_url_no_pad(&self) -> bool {
        matches!(self, Self::Base64UrlNoPad)
    }

    /// Gets the [MultibaseHeader] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Base58Btc => Self::BASE58_BTC,
            Self::Base64UrlNoPad => Self::BASE64_URL_NO_PAD,
        }
    }

    /// Gets the [MultibaseHeader] string representation.
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Base58Btc => Self::BASE58_BTC_BYTE,
            Self::Base64UrlNoPad => Self::BASE64_URL_NO_PAD_BYTE,
        }
    }
}

impl TryFrom<u8> for MultibaseHeader {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self> {
        match val {
            Self::BASE58_BTC_BYTE => Ok(Self::Base58Btc),
            Self::BASE64_URL_NO_PAD_BYTE => Ok(Self::Base64UrlNoPad),
            _ => Err(Error::multikey(format!("invalid multibase header: {val}"))),
        }
    }
}

impl TryFrom<&[u8]> for MultibaseHeader {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        val.first()
            .ok_or(Error::multikey("empty header"))
            .copied()
            .and_then(Self::try_from)
    }
}

impl TryFrom<&str> for MultibaseHeader {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val {
            Self::BASE58_BTC => Ok(Self::Base58Btc),
            Self::BASE64_URL_NO_PAD => Ok(Self::Base64UrlNoPad),
            _ => Err(Error::multikey("invalid multibase header: {val}")),
        }
    }
}

impl TryFrom<String> for MultibaseHeader {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl_default!(MultibaseHeader);
impl_display!(MultibaseHeader, str);
