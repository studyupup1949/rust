use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Multikey, Result, impl_default, impl_display};

/// Represents a [Multikey], or an [Iri] referencing a [Multikey]s.
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{
///   Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyItem, MultikeyPublicKey,
/// };
///
/// # fn main() {
/// let id = Iri::try_from("https://controller.example/123456789abcdefghi#keys-1").unwrap();
/// let controller = Iri::try_from("https://controller.example/123456789abcdefghi").unwrap();
/// let multibase = MultibasePublicKey::new()
///     .with_header(MultibaseHeader::Base58Btc)
///     .with_key(MultikeyPublicKey::Ed25519([
///         0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72,
///         0xa, 0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
///         0x1b, 0x42, 0x87, 0xa6,
///     ]));
/// let encoded_multibase = multibase.encode();
///
/// let json_str = format!(
/// r#"{{
///   "@context": "https://www.w3.org/ns/cid/v1",
///   "type": "Multikey",
///   "id": "{id}",
///   "controller": "{controller}",
///   "publicKeyMultibase": "{encoded_multibase}"
/// }}"#
///         );
///
/// let multikey = Multikey::new()
///     .with_id(id)
///     .with_controller(controller)
///     .with_public_key_multibase(multibase);
///
/// let multikey_item = MultikeyItem::multikey(multikey);
///
/// assert!(multikey_item.is_multikey());
/// assert_eq!(serde_json::to_string_pretty(&multikey_item).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<MultikeyItem>(&json_str).unwrap(),
///     multikey_item
/// );
/// # }
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MultikeyItem {
    Multikey(Multikey),
    Iri(Iri),
}

impl MultikeyItem {
    /// Creates a new [MultikeyItem].
    pub fn new() -> Self {
        Self::Multikey(Multikey::new())
    }

    /// Creates a new [MultikeyItem] [Multikey](Self::Multikey) variant.
    pub fn multikey<I: Into<Multikey>>(val: I) -> Self {
        Self::Multikey(val.into())
    }

    /// Gets whether the [MultikeyItem] contains a [Multikey](Self::Multikey) variant.
    pub const fn is_multikey(&self) -> bool {
        matches!(self, Self::Multikey(_))
    }

    /// Attempts to get a reference to the [Multikey](Self::Multikey) variant.
    pub fn as_multikey(&self) -> Result<&Multikey> {
        match self {
            Self::Multikey(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Attempts to convert to a [Multikey](Self::Multikey) variant.
    pub fn to_multikey(self) -> Result<Multikey> {
        match self {
            Self::Multikey(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Creates a new [MultikeyItem] [Iri](Self::Iri) variant.
    pub fn iri<I: Into<Iri>>(val: I) -> Self {
        Self::Iri(val.into())
    }

    /// Gets whether the [MultikeyItem] contains a [Iri](Self::Iri) variant.
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to get a reference to the [Iri](Self::Iri) variant.
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Attempts to convert to a [Iri](Self::Iri) variant.
    pub fn to_iri(self) -> Result<Iri> {
        match self {
            Self::Iri(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }
}

impl From<Multikey> for MultikeyItem {
    fn from(val: Multikey) -> Self {
        Self::multikey(val)
    }
}

impl From<Iri> for MultikeyItem {
    fn from(val: Iri) -> Self {
        Self::iri(val)
    }
}

impl<'a> TryFrom<&'a MultikeyItem> for &'a Multikey {
    type Error = Error;

    fn try_from(val: &'a MultikeyItem) -> Result<Self> {
        val.as_multikey()
    }
}

impl TryFrom<MultikeyItem> for Multikey {
    type Error = Error;

    fn try_from(val: MultikeyItem) -> Result<Self> {
        val.to_multikey()
    }
}

impl<'a> TryFrom<&'a MultikeyItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a MultikeyItem) -> Result<Self> {
        val.as_iri()
    }
}

impl TryFrom<MultikeyItem> for Iri {
    type Error = Error;

    fn try_from(val: MultikeyItem) -> Result<Self> {
        val.to_iri()
    }
}

impl_default!(MultikeyItem);
impl_display!(MultikeyItem, json);

/// Represents a list of [MultikeyItem]s.
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{
///   Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyItem, MultikeyItems, MultikeyPublicKey,
/// };
///
/// # fn main() {
/// let id = Iri::try_from("https://controller.example/123456789abcdefghi#keys-1").unwrap();
/// let controller = Iri::try_from("https://controller.example/123456789abcdefghi").unwrap();
/// let multibase = MultibasePublicKey::new()
///     .with_header(MultibaseHeader::Base58Btc)
///     .with_key(MultikeyPublicKey::Ed25519([
///         0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72,
///         0xa, 0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
///         0x1b, 0x42, 0x87, 0xa6,
///     ]));
/// let encoded_multibase = multibase.encode();
/// let multikey_iri = Iri::try_from("https://controller.example/123456789abcdefghi#keys-2").unwrap();
///
/// let json_str = format!(
/// r#"[
///   {{
///     "@context": "https://www.w3.org/ns/cid/v1",
///     "type": "Multikey",
///     "id": "{id}",
///     "controller": "{controller}",
///     "publicKeyMultibase": "{encoded_multibase}"
///   }},
///   "{multikey_iri}"
/// ]"#
///         );
///
/// let multikey = Multikey::new()
///     .with_id(id)
///     .with_controller(controller)
///     .with_public_key_multibase(multibase);
///
/// let multikey_item0 = MultikeyItem::multikey(multikey);
/// let multikey_item1 = MultikeyItem::iri(multikey_iri);
///
/// assert!(multikey_item0.is_multikey());
/// assert!(multikey_item1.is_iri());
///
/// let multikey_items = MultikeyItems::list([multikey_item0, multikey_item1]);
///
/// assert_eq!(serde_json::to_string_pretty(&multikey_items).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<MultikeyItems>(&json_str).unwrap(),
///     multikey_items
/// );
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MultikeyItems {
    Single(MultikeyItem),
    List(Vec<MultikeyItem>),
}

impl MultikeyItems {
    /// Creates a new [MultikeyItems].
    pub fn new() -> Self {
        Self::Single(MultikeyItem::new())
    }

    /// Creates a new [MultikeyItems] [Single](Self::Single) variant.
    pub fn single<I: Into<MultikeyItem>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [MultikeyItems] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&MultikeyItem> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Attempts to convert to a [Single](Self::Single) variant.
    pub fn into_single(self) -> Result<MultikeyItem> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Creates a new [MultikeyItems] [Single](Self::Single) variant.
    pub fn list<T, I>(val: I) -> Self
    where
        T: Into<MultikeyItem>,
        I: IntoIterator<Item = T>,
    {
        Self::List(val.into_iter().map(|i| i.into()).collect())
    }

    /// Gets whether the [MultikeyItems] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[MultikeyItem]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }

    /// Attempts to convert to a [List](Self::List) variant.
    pub fn into_list(self) -> Result<Vec<MultikeyItem>> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid multikeys type")),
        }
    }
}

impl From<MultikeyItem> for MultikeyItems {
    fn from(val: MultikeyItem) -> Self {
        Self::single(val)
    }
}

impl From<Vec<MultikeyItem>> for MultikeyItems {
    fn from(val: Vec<MultikeyItem>) -> Self {
        Self::list(val)
    }
}

impl From<&[MultikeyItem]> for MultikeyItems {
    fn from(val: &[MultikeyItem]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<&[MultikeyItem; N]> for MultikeyItems {
    fn from(val: &[MultikeyItem; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<[MultikeyItem; N]> for MultikeyItems {
    fn from(val: [MultikeyItem; N]) -> Self {
        Self::list(val)
    }
}

impl<'a> TryFrom<&'a MultikeyItems> for &'a MultikeyItem {
    type Error = Error;

    fn try_from(val: &'a MultikeyItems) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<MultikeyItems> for MultikeyItem {
    type Error = Error;

    fn try_from(val: MultikeyItems) -> Result<Self> {
        val.into_single()
    }
}

impl<'a> TryFrom<&'a MultikeyItems> for &'a [MultikeyItem] {
    type Error = Error;

    fn try_from(val: &'a MultikeyItems) -> Result<Self> {
        val.as_list()
    }
}

impl TryFrom<MultikeyItems> for Vec<MultikeyItem> {
    type Error = Error;

    fn try_from(val: MultikeyItems) -> Result<Self> {
        val.into_list()
    }
}

impl_default!(MultikeyItems);
impl_display!(MultikeyItems, json);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MultibaseHeader, MultibasePublicKey, MultikeyPublicKey};

    #[test]
    fn test_multikey_item() {
        let id = Iri::try_from("https://controller.example/123456789abcdefghi#keys-1").unwrap();
        let controller = Iri::try_from("https://controller.example/123456789abcdefghi").unwrap();
        let multibase = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(MultikeyPublicKey::Ed25519([
                0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72,
                0xa, 0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
                0x1b, 0x42, 0x87, 0xa6,
            ]));
        let encoded_multibase = multibase.encode();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/cid/v1",
  "type": "Multikey",
  "id": "{id}",
  "controller": "{controller}",
  "publicKeyMultibase": "{encoded_multibase}"
}}"#
        );

        let multikey = Multikey::new()
            .with_id(id)
            .with_controller(controller)
            .with_public_key_multibase(multibase);

        let multikey_item = MultikeyItem::multikey(multikey);

        assert!(multikey_item.is_multikey());
        assert_eq!(
            serde_json::to_string_pretty(&multikey_item).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<MultikeyItem>(&json_str).unwrap(),
            multikey_item
        );
    }

    #[test]
    fn test_multikey_items() {
        let id = Iri::try_from("https://controller.example/123456789abcdefghi#keys-1").unwrap();
        let controller = Iri::try_from("https://controller.example/123456789abcdefghi").unwrap();
        let multibase = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(MultikeyPublicKey::Ed25519([
                0x66, 0x6a, 0xbe, 0x8, 0x90, 0x35, 0xed, 0x4a, 0x45, 0x79, 0x59, 0x89, 0xfa, 0x72,
                0xa, 0xe, 0xe0, 0xc4, 0x85, 0x49, 0xe8, 0x45, 0x23, 0x48, 0x5c, 0x56, 0xf1, 0xc8,
                0x1b, 0x42, 0x87, 0xa6,
            ]));
        let encoded_multibase = multibase.encode();
        let multikey_iri =
            Iri::try_from("https://controller.example/123456789abcdefghi#keys-2").unwrap();

        let json_str = format!(
            r#"[
  {{
    "@context": "https://www.w3.org/ns/cid/v1",
    "type": "Multikey",
    "id": "{id}",
    "controller": "{controller}",
    "publicKeyMultibase": "{encoded_multibase}"
  }},
  "{multikey_iri}"
]"#
        );

        let multikey = Multikey::new()
            .with_id(id)
            .with_controller(controller)
            .with_public_key_multibase(multibase);

        let multikey_item0 = MultikeyItem::multikey(multikey);
        let multikey_item1 = MultikeyItem::iri(multikey_iri);

        assert!(multikey_item0.is_multikey());
        assert!(multikey_item1.is_iri());

        let multikey_items = MultikeyItems::list([multikey_item0, multikey_item1]);

        assert_eq!(
            serde_json::to_string_pretty(&multikey_items).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<MultikeyItems>(&json_str).unwrap(),
            multikey_items
        );
    }
}
