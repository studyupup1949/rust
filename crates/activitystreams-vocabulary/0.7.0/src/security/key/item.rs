use crate::{Iri, Key, create_item, create_list};

create_item! {
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
    KeyItem
        default: Self::Key(Key::new()),
    {
        Key(Key),
        Iri(Iri),
    }
}

create_list! {
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
    KeyItems: KeyItem,
}

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
