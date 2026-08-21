use serde::{Serialize, de, ser};

use crate::{ActivityVocabulary, Context, Iri, SecurityType, field_access, impl_default};

mod private;
mod public;

pub use private::PrivateKeyPem;
pub use public::PublicKeyPem;

/// Represents a key defined by [Security Vocabulary V1](w3c-ccg.github.io/security-vocab).
///
/// This format is obsolete, replaced by [Security Vocabulary V2](w3c.github.io/vc-data-integrity/vocab/security/vocabulary.html).
///
/// However, some popular ActivityPub implementations still make use of V1 definitions, e.g. [Mastodon `publicKey`](https://docs.joinmastodon.org/spec/activitypub/#publicKey).
///
/// # Example
///
/// ```rust
/// use activitystreams_vocabulary::{Iri, Key, PublicKeyPem};
/// # fn main() {
/// let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
/// let owner = Iri::try_from("https://example.dev/alice").unwrap();
///
///
/// let public_encoded = "-----BEGIN PUBLIC KEY-----
/// 9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
/// -----END PUBLIC KEY-----
/// ";
///
/// let public_json_str = serde_json::to_string(public_encoded).unwrap();
///
/// let json_str = format!(r#"{{
///   "id": "{id}",
///   "owner": "{owner}",
///   "publicKeyPem": {public_json_str}
/// }}"#
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
/// assert_eq!(serde_json::to_string_pretty(&key).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Key>(&json_str).unwrap(),
///     key
/// );
/// # }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    #[serde(rename = "@context", skip_serializing_if = "Option::is_none")]
    context_property: Option<Context>,
    #[serde(
        rename = "type",
        skip_serializing_if = "Option::is_none",
        serialize_with = "kind_ser"
    )]
    kind: Option<SecurityType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    controller: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_key_pem: Option<PrivateKeyPem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_key_pem: Option<PublicKeyPem>,
}

impl Key {
    /// Creates a new [Key].
    pub fn new() -> Self {
        Self {
            context_property: None,
            kind: None,
            id: None,
            controller: None,
            owner: None,
            private_key_pem: None,
            public_key_pem: None,
        }
    }
    /// Gets a reference to the PEM private key.
    pub fn private_key_pem(&self) -> Option<&PrivateKeyPem> {
        self.private_key_pem.as_ref()
    }

    /// Sets the PEM private key.
    pub fn set_private_key_pem<I: Into<PrivateKeyPem>>(&mut self, val: I) {
        self.private_key_pem = Some(val.into());
    }

    /// Builder function that sets the PEM private key.
    pub fn with_private_key_pem<I: Into<PrivateKeyPem>>(self, val: I) -> Self {
        Self {
            private_key_pem: Some(val.into()),
            ..self.clone()
        }
    }

    /// Gets a reference to the PEM public key.
    pub fn public_key_pem(&self) -> Option<&PublicKeyPem> {
        self.public_key_pem.as_ref()
    }

    /// Sets the PEM public key.
    pub fn set_public_key_pem<I: Into<PublicKeyPem>>(&mut self, val: I) {
        self.public_key_pem = Some(val.into());
    }

    /// Builder function that sets the PEM public key.
    pub fn with_public_key_pem<I: Into<PublicKeyPem>>(self, val: I) -> Self {
        Self {
            public_key_pem: Some(val.into()),
            ..self.clone()
        }
    }
}

impl_default!(Key);

field_access! {
    Key {
        /// Represents the vocabulary `type` field.
        ///
        /// Should always be `Key`.
        kind: option { SecurityType },
    }
}

field_access! {
    Key {
        /// Represents the special `@context` property to define the processing context.
        ///
        /// The value of the `@context` property is defined by the [JSON-LD](https://www.w3.org/TR/json-ld/#the-context) specification.
        context_property: option_ref { Context },
        /// References the IRI for fetching the key information.
        id: option_ref { Iri },
        /// References the controlling Actor for the key information.
        ///
        /// Alias of the `owner` property.
        controller: option_ref { Iri },
        /// References the owning Actor for the key information.
        ///
        /// Alias of the `controller` property.
        owner: option_ref { Iri },
    }
}

fn kind_ser<S>(kind: &Option<SecurityType>, s: S) -> core::result::Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    use ser::Serialize;

    if let Some(k) = kind.as_ref()
        && k.contains(SecurityType::Key.as_str())
    {
        k.serialize(s)
    } else if let Some(k) = kind.as_ref() {
        Err(ser::Error::custom(format!("invalid vocabulary type: {k}",)))
    } else {
        s.serialize_none()
    }
}

impl<'de> de::Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'vde> de::Visitor<'vde> for Visitor {
            type Value = Key;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("Key")
            }

            #[allow(unused)]
            fn visit_map<V>(self, mut map: V) -> core::result::Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'vde>,
            {
                let mut context_property = None;
                let mut kind: Option<SecurityType> = None;
                let mut id = None;
                let mut controller = None;
                let mut owner = None;
                let mut private_key_pem = None;
                let mut public_key_pem = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@context" => {
                            if context_property.is_some() {
                                return Err(de::Error::duplicate_field("@context"));
                            }
                            context_property = Some(map.next_value()?);
                        }
                        "type" => {
                            if kind.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }

                            kind = Some(map.next_value()?);
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "controller" => {
                            if controller.is_some() {
                                return Err(de::Error::duplicate_field("controller"));
                            }
                            controller = Some(map.next_value()?);
                        }
                        "owner" => {
                            if owner.is_some() {
                                return Err(de::Error::duplicate_field("owner"));
                            }
                            owner = Some(map.next_value()?);
                        }
                        "privateKeyPem" => {
                            if private_key_pem.is_some() {
                                return Err(de::Error::duplicate_field("privateKeyPem"));
                            }
                            private_key_pem = Some(map.next_value()?);
                        }
                        "publicKeyPem" => {
                            if public_key_pem.is_some() {
                                return Err(de::Error::duplicate_field("publicKeyPem"));
                            }
                            public_key_pem = Some(map.next_value()?);
                        }
                        _ => (),
                    }
                }

                if let Some(k) = kind.as_ref()
                    && !k.contains(SecurityType::Key.as_str())
                {
                    Err(de::Error::custom("invalid Key type: {k}"))
                } else {
                    Ok(Self::Value {
                        context_property,
                        kind,
                        id,
                        controller,
                        owner,
                        private_key_pem,
                        public_key_pem,
                    })
                }
            }
        }

        deserializer.deserialize_map(Visitor {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Iri;

    #[test]
    fn test_key() {
        let id = Iri::try_from("https://example.dev/alice#main-key").unwrap();
        let controller = Iri::try_from("https://example.dev/alice").unwrap();

        let private_encoded = "-----BEGIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIBftnHPp22SewYmmEoMcX8VwI4IHwaqd+9LFPj/15eqF
-----END PRIVATE KEY-----
";

        let private_json_str = serde_json::to_string(private_encoded).unwrap();

        let public_encoded = "-----BEGIN PUBLIC KEY-----
9IiXDqOOsPkAZIpt7CoJC9pFkd9w8Z7USKjNa7AVmA+rZbZ0C/BSeh0Ywy4ZrncS
-----END PUBLIC KEY-----
";

        let public_json_str = serde_json::to_string(public_encoded).unwrap();

        // privateKeyPem is included just for an example
        //
        // **WARN**: you should NEVER send private keys over insecure channels,
        // the key is not encrypted
        let json_str = format!(
            r#"{{
  "id": "{id}",
  "controller": "{controller}",
  "privateKeyPem": {private_json_str},
  "publicKeyPem": {public_json_str}
}}"#
        );

        let private_key_bytes = [
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20, 0x17, 0xed, 0x9c, 0x73, 0xe9, 0xdb, 0x64, 0x9e, 0xc1, 0x89, 0xa6, 0x12,
            0x83, 0x1c, 0x5f, 0xc5, 0x70, 0x23, 0x82, 0x07, 0xc1, 0xaa, 0x9d, 0xfb, 0xd2, 0xc5,
            0x3e, 0x3f, 0xf5, 0xe5, 0xea, 0x85,
        ];

        let public_key_bytes = [
            0xf4, 0x88, 0x97, 0x0e, 0xa3, 0x8e, 0xb0, 0xf9, 0x00, 0x64, 0x8a, 0x6d, 0xec, 0x2a,
            0x09, 0x0b, 0xda, 0x45, 0x91, 0xdf, 0x70, 0xf1, 0x9e, 0xd4, 0x48, 0xa8, 0xcd, 0x6b,
            0xb0, 0x15, 0x98, 0x0f, 0xab, 0x65, 0xb6, 0x74, 0x0b, 0xf0, 0x52, 0x7a, 0x1d, 0x18,
            0xc3, 0x2e, 0x19, 0xae, 0x77, 0x12,
        ];

        let private_key_pem = PrivateKeyPem::new().with_key(private_key_bytes);

        let public_key_pem = PublicKeyPem::new().with_key(public_key_bytes);

        let key = Key::new()
            .with_id(id)
            .with_controller(controller)
            .with_private_key_pem(private_key_pem)
            .with_public_key_pem(public_key_pem);

        assert_eq!(serde_json::to_string_pretty(&key).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Key>(&json_str).unwrap(), key);
    }

    #[test]
    fn test_key_mastodon() {
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

        assert_eq!(serde_json::to_string_pretty(&key).unwrap(), json_str);
        assert_eq!(serde_json::from_str::<Key>(&json_str).unwrap(), key);
    }
}
