use serde::{Deserialize, Serialize};

use crate::{Context, Error, Iri, Result, SecurityType, field_access, impl_default, impl_display};

mod base;
mod data;
mod header;
mod public;

pub use base::MultibasePublicKey;
pub use data::MultibaseData;
pub use header::MultibaseHeader;
pub use public::MultikeyPublicKey;

/// Represents a [Controlled Identifier `Multikey`](https://www.w3.org/TR/cid-1.0/#Multikey) object.
///
/// Used in [FEP-521a](https://codeberg.org/fediverse/fep/src/branch/main/fep/521a/fep-521a.md) +
/// [FEP-521b](https://codeberg.org/fediverse/fep/pulls/723).
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{Context, Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey};
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
/// let encoded_multibase = "z6MkmM42vxfqZQsv4ehtTjFFxQ4sQKS2w6WR7emozFAn5cxu";
///
/// let json_str = format!(
/// r#"{{
///   "@context": "https://www.w3.org/ns/cid/v1",
///   "type": "Multikey",
///   "id": "{id}",
///   "controller": "{controller}",
///   "publicKeyMultibase": "{encoded_multibase}"
/// }}"#
///        );
///
/// let multikey = Multikey::new()
///     .with_id(id)
///     .with_controller(controller)
///     .with_public_key_multibase(multibase);
///
/// assert_eq!(serde_json::to_string_pretty(&multikey).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Multikey>(&json_str).unwrap(),
///     multikey
/// );
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Multikey {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    context: Option<Context>,
    #[serde(rename = "type")]
    kind: SecurityType,
    id: Iri,
    controller: Iri,
    public_key_multibase: MultibasePublicKey,
}

impl Multikey {
    pub const CONTEXT_IRI: &str = "https://www.w3.org/ns/cid/v1";

    /// Creates a new [Multikey].
    pub fn new() -> Self {
        Self {
            context: Some(Context::Iri(Iri::new_trusted(Self::CONTEXT_IRI.into()))),
            kind: SecurityType::Multikey,
            id: Iri::new(),
            controller: Iri::new(),
            public_key_multibase: MultibasePublicKey::new(),
        }
    }

    /// Creates a new [Multikey] without the `@context` property.
    pub fn new_inner() -> Self {
        Self::new().without_context()
    }

    /// Builder function that unsets the `@context` property.
    pub fn without_context(self) -> Self {
        Self {
            context: None,
            ..self
        }
    }
}

field_access! {
    Multikey {
        context: option_ref { Context },
    }
}

field_access! {
    Multikey {
        kind: SecurityType,
    }
}

field_access! {
    Multikey {
        id: as_ref { Iri },
        controller: as_ref { Iri },
        public_key_multibase: as_ref { MultibasePublicKey },
    }
}

impl_default!(Multikey);
impl_display!(Multikey, json);

/// Represents a list of [Multikey]s.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Multikeys {
    Single(Multikey),
    List(Vec<Multikey>),
}

impl Multikeys {
    /// Creates a new [Multikeys].
    pub fn new() -> Self {
        Self::Single(Multikey::new())
    }

    /// Creates a new [Multikeys] [Single](Self::Single) variant.
    pub fn single<I: Into<Multikey>>(val: I) -> Self {
        Self::Single(val.into())
    }

    /// Gets whether the [Multikeys] contains a [Single](Self::Single) variant.
    pub const fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    /// Attempts to get a reference to the [Single](Self::Single) variant.
    pub fn as_single(&self) -> Result<&Multikey> {
        match self {
            Self::Single(ty) => Ok(ty),
            _ => Err(Error::item("invalid items type")),
        }
    }

    /// Creates a new [Multikeys] [Single](Self::Single) variant.
    pub fn list<I: IntoIterator<Item = Multikey>>(val: I) -> Self {
        Self::List(val.into_iter().collect())
    }

    /// Gets whether the [Multikeys] contains a [List](Self::List) variant.
    pub const fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    /// Attempts to get a reference to the [List](Self::List) variant.
    pub fn as_list(&self) -> Result<&[Multikey]> {
        match self {
            Self::List(tys) => Ok(tys),
            _ => Err(Error::item("invalid items type")),
        }
    }
}

impl From<Multikey> for Multikeys {
    fn from(val: Multikey) -> Self {
        Self::single(val)
    }
}

impl From<Vec<Multikey>> for Multikeys {
    fn from(val: Vec<Multikey>) -> Self {
        Self::list(val)
    }
}

impl From<&[Multikey]> for Multikeys {
    fn from(val: &[Multikey]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<&[Multikey; N]> for Multikeys {
    fn from(val: &[Multikey; N]) -> Self {
        Self::list(val.iter().cloned())
    }
}

impl<const N: usize> From<[Multikey; N]> for Multikeys {
    fn from(val: [Multikey; N]) -> Self {
        Self::list(val)
    }
}

impl_default!(Multikeys);
impl_display!(Multikeys, json);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multikey() {
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

        assert_eq!(serde_json::to_string_pretty(&multikey).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Multikey>(&json_str).unwrap(),
            multikey
        );
    }
}
