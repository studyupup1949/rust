use serde::{Deserialize, Serialize};

use crate::{Error, Iri, Key, Result, impl_default, impl_display};

/// Represents a [Key], or an [Iri] referencing a [Key].
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{Iri, Key, KeyItem, PublicKeyPem};
///
/// # fn main() {
/// let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
/// let owner = Iri::try_from("https://example.dev/alice").unwrap();
///
/// let public_encoded = "-----BEGIN PUBLIC KEY-----
/// 9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
/// -----END PUBLIC KEY-----
/// ";
///
/// let public_json_str = serde_json::to_string(public_encoded).unwrap();
///
/// let json_str = format!(
/// r#"{{
///   "id": "{id}",
///   "owner": "{owner}",
///   "publicKeyPem": {public_json_str}
/// }}"#
///        );
///
/// let public_key_bytes = [
///     0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
///     0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
///     0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
///     0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
/// ];
///
/// let public_key_pem = PublicKeyPem::new().with_key(public_key_bytes);
///
/// let key = Key::new()
///     .with_id(id)
///     .with_owner(owner)
///     .with_public_key_pem(public_key_pem);
///
/// let key_item = KeyItem::key(key);
///
/// assert!(key_item.is_key());
/// assert_eq!(serde_json::to_string_pretty(&key_item).unwrap(), json_str);
/// assert_eq!(serde_json::from_str::<KeyItem>(&json_str).unwrap(), key_item);
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeyItem {
    Key(Key),
    Iri(Iri),
}

impl KeyItem {
    /// Creates a new [KeyItem].
    pub fn new() -> Self {
        Self::Key(Key::new())
    }

    /// Creates a new [KeyItem] [Key](Self::Key) variant.
    pub fn key<I: Into<Key>>(val: I) -> Self {
        Self::Key(val.into())
    }

    /// Gets whether the [KeyItem] contains a [Key](Self::Key) variant.
    pub const fn is_key(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    /// Attempts to get a reference to the [Key](Self::Key) variant.
    pub fn as_key(&self) -> Result<&Key> {
        match self {
            Self::Key(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Attempts to convert to a [Key](Self::Key) variant.
    pub fn to_key(self) -> Result<Key> {
        match self {
            Self::Key(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Creates a new [KeyItem] [Iri](Self::Iri) variant.
    pub fn iri<I: Into<Iri>>(val: I) -> Self {
        Self::Iri(val.into())
    }

    /// Gets whether the [KeyItem] contains a [Iri](Self::Iri) variant.
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Attempts to get a reference to the [Iri](Self::Iri) variant.
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Attempts to convert to a [Iri](Self::Iri) variant.
    pub fn to_iri(self) -> Result<Iri> {
        match self {
            Self::Iri(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }
}

impl From<Key> for KeyItem {
    fn from(val: Key) -> Self {
        Self::key(val)
    }
}

impl From<Iri> for KeyItem {
    fn from(val: Iri) -> Self {
        Self::iri(val)
    }
}

impl<'a> TryFrom<&'a KeyItem> for &'a Key {
    type Error = Error;

    fn try_from(val: &'a KeyItem) -> Result<Self> {
        val.as_key()
    }
}

impl TryFrom<KeyItem> for Key {
    type Error = Error;

    fn try_from(val: KeyItem) -> Result<Self> {
        val.to_key()
    }
}

impl<'a> TryFrom<&'a KeyItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a KeyItem) -> Result<Self> {
        val.as_iri()
    }
}

impl TryFrom<KeyItem> for Iri {
    type Error = Error;

    fn try_from(val: KeyItem) -> Result<Self> {
        val.to_iri()
    }
}

impl_default!(KeyItem);
impl_display!(KeyItem, json);

/// Represents a list of [KeyItem]s.
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{Iri, Key, KeyItem, KeyItems, PublicKeyPem};
///
/// # fn main() {
/// let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
/// let owner = Iri::try_from("https://example.dev/alice").unwrap();
///
/// let public_encoded = "-----BEGIN PUBLIC KEY-----
/// 9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
/// -----END PUBLIC KEY-----
/// ";
///
/// let public_json_str = serde_json::to_string(public_encoded).unwrap();
/// let key_iri = Iri::try_from("https://example.dev/alice#second-key").unwrap();
///
/// let json_str = format!(
/// r#"[
///   {{
///     "id": "{id}",
///     "owner": "{owner}",
///     "publicKeyPem": {public_json_str}
///   }},
///   "{key_iri}"
/// ]"#
///         );
///
/// let public_key_bytes = [
///     0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
///     0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
///     0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
///     0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
/// ];
///
/// let public_key_pem = PublicKeyPem::new().with_key(public_key_bytes);
///
/// let key = Key::new()
///     .with_id(id)
///     .with_owner(owner)
///     .with_public_key_pem(public_key_pem);
///
/// let key_item0 = KeyItem::key(key);
/// let key_item1 = KeyItem::iri(key_iri);
///
/// assert!(key_item0.is_key());
/// assert!(key_item1.is_iri());
///
/// let key_items = KeyItems::list([key_item0, key_item1]);
///
/// assert_eq!(serde_json::to_string_pretty(&key_items).unwrap(), json_str);
/// assert_eq!(serde_json::from_str::<KeyItems>(&json_str).unwrap(), key_items);
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum KeyItems {
    Single(KeyItem),
    List(Vec<KeyItem>),
}

impl KeyItems {
    /// Creates a new [KeyItems].
    pub fn new() -> Self {
        Self::Single(KeyItem::new())
    }

    /// Creates a new [KeyItems] [Single](Self::Single) variant.
    pub fn single<I: Into<KeyItem>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [KeyItems] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&KeyItem> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Attempts to convert to a [Single](Self::Single) variant.
    pub fn to_single(self) -> Result<KeyItem> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Creates a new [KeyItems] [Single](Self::Single) variant.
    pub fn list<I: IntoIterator<Item = KeyItem>>(val: I) -> Self {
        Self::List(val.into_iter().collect())
    }

    /// Gets whether the [KeyItems] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[KeyItem]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid keys type")),
        }
    }

    /// Attempts to convert to a [List](Self::List) variant.
    pub fn to_list(self) -> Result<Vec<KeyItem>> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid keys type")),
        }
    }
}

impl From<KeyItem> for KeyItems {
    fn from(val: KeyItem) -> Self {
        Self::single(val)
    }
}

impl From<Vec<KeyItem>> for KeyItems {
    fn from(val: Vec<KeyItem>) -> Self {
        Self::list(val)
    }
}

impl From<&[KeyItem]> for KeyItems {
    fn from(val: &[KeyItem]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<&[KeyItem; N]> for KeyItems {
    fn from(val: &[KeyItem; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<[KeyItem; N]> for KeyItems {
    fn from(val: [KeyItem; N]) -> Self {
        Self::list(val)
    }
}

impl<'a> TryFrom<&'a KeyItems> for &'a KeyItem {
    type Error = Error;

    fn try_from(val: &'a KeyItems) -> Result<Self> {
        val.as_single()
    }
}

impl TryFrom<KeyItems> for KeyItem {
    type Error = Error;

    fn try_from(val: KeyItems) -> Result<Self> {
        val.to_single()
    }
}

impl<'a> TryFrom<&'a KeyItems> for &'a [KeyItem] {
    type Error = Error;

    fn try_from(val: &'a KeyItems) -> Result<Self> {
        val.as_list()
    }
}

impl TryFrom<KeyItems> for Vec<KeyItem> {
    type Error = Error;

    fn try_from(val: KeyItems) -> Result<Self> {
        val.to_list()
    }
}

impl_default!(KeyItems);
impl_display!(KeyItems, json);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PublicKeyPem;

    #[test]
    fn test_key_item() {
        let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
        let owner = Iri::try_from("https://example.dev/alice").unwrap();

        let public_encoded = "-----BEGIN PUBLIC KEY-----
9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
-----END PUBLIC KEY-----
";

        let public_json_str = serde_json::to_string(public_encoded).unwrap();

        let json_str = format!(
            r#"{{
  "id": "{id}",
  "owner": "{owner}",
  "publicKeyPem": {public_json_str}
}}"#
        );

        let public_key_bytes = [
            0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
            0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
            0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
            0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
        ];

        let public_key_pem = PublicKeyPem::new().with_key(public_key_bytes);

        let key = Key::new()
            .with_id(id)
            .with_owner(owner)
            .with_public_key_pem(public_key_pem);

        let key_item = KeyItem::key(key);

        assert!(key_item.is_key());
        assert_eq!(serde_json::to_string_pretty(&key_item).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<KeyItem>(&json_str).unwrap(),
            key_item
        );
    }

    #[test]
    fn test_key_items() {
        let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
        let owner = Iri::try_from("https://example.dev/alice").unwrap();

        let public_encoded = "-----BEGIN PUBLIC KEY-----
9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
-----END PUBLIC KEY-----
";

        let public_json_str = serde_json::to_string(public_encoded).unwrap();
        let key_iri = Iri::try_from("https://example.dev/alice#second-key").unwrap();

        let json_str = format!(
            r#"[
  {{
    "id": "{id}",
    "owner": "{owner}",
    "publicKeyPem": {public_json_str}
  }},
  "{key_iri}"
]"#
        );

        let public_key_bytes = [
            0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
            0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
            0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
            0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
        ];

        let public_key_pem = PublicKeyPem::new().with_key(public_key_bytes);

        let key = Key::new()
            .with_id(id)
            .with_owner(owner)
            .with_public_key_pem(public_key_pem);

        let key_item0 = KeyItem::key(key);
        let key_item1 = KeyItem::iri(key_iri);

        assert!(key_item0.is_key());
        assert!(key_item1.is_iri());

        let key_items = KeyItems::list([key_item0, key_item1]);

        assert_eq!(serde_json::to_string_pretty(&key_items).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<KeyItems>(&json_str).unwrap(),
            key_items
        );
    }
}
