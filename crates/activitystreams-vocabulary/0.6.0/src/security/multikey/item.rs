use crate::{Iri, Multikey, create_item, create_list};

create_item! {
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
    MultikeyItem
        default: Self::Multikey(Multikey::new()),
    {
        Multikey(Multikey),
        Iri(Iri),
    }
}

create_list! {
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
    MultikeyItems: MultikeyItem,
}

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
