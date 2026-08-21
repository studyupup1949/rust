use activitystreams_vocabulary::{create_activity, field_access};

use crate::GrantItem;

create_activity! {
    /// Indicates that a [Ticket](crate::Ticket) is being closed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{GrantItem, Resolve, context};
    /// use activitystreams_vocabulary::{
    ///     Cryptosuite, DataIntegrityProof, DateTime, Iri, MultibaseData, MultibaseHeader,
    /// };
    ///
    /// # fn main() {
    /// let id =
    ///     Iri::try_from("https://grape.fr33domlover.site/people/WZpnG/outbox/GQvnR").unwrap();
    /// let actor = Iri::try_from("https://grape.fr33domlover.site/people/WZpnG").unwrap();
    /// let object =
    ///     Iri::try_from("https://fig.fr33domlover.site/decks/W058b/tickets/3bPVn").unwrap();
    /// let capability =
    ///     Iri::try_from("https://grape.fr33domlover.site/groups/OZLdZ/outbox/ZLdDZ").unwrap();
    ///
    /// let to0 = Iri::try_from("https://grape.fr33domlover.site/people/WZpnG/followers").unwrap();
    /// let to1 = Iri::try_from("https://fig.fr33domlover.site/decks/W058b").unwrap();
    /// let to2 = Iri::try_from("https://fig.fr33domlover.site/decks/W058b/followers").unwrap();
    /// let to3 =
    ///     Iri::try_from("https://fig.fr33domlover.site/decks/W058b/tickets/3bPVn/followers")
    ///         .unwrap();
    ///
    /// let proof_created = "2024-07-09T07:02:45.038760903Z";
    /// let proof_cryptosuite = Cryptosuite::EddsaJcs2022;
    /// let proof_purpose = Iri::try_from("assertionMethod").unwrap();
    /// let proof_value = "z4yiaWtjPeUjBvoXpHffjguVd8uHsSfYr4zxxFJbZwPVLtRNCk3uMKtnC3nRZ8vRM3h36B6VLrFagVerxcymhj2NP";
    /// let verification_method = Iri::try_from("https://grape.fr33domlover.site/akey1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Resolve",
    ///   "id": "{id}",
    ///   "to": [
    ///     "{to0}",
    ///     "{to1}",
    ///     "{to2}",
    ///     "{to3}"
    ///   ],
    ///   "proof": {{
    ///     "type": "DataIntegrityProof",
    ///     "cryptosuite": "{proof_cryptosuite}",
    ///     "proofValue": "{proof_value}",
    ///     "proofPurpose": "{proof_purpose}",
    ///     "created": "{proof_created}",
    ///     "verificationMethod": "{verification_method}"
    ///   }},
    ///   "actor": "{actor}",
    ///   "object": "{object}",
    ///   "capability": "{capability}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let proof_data = MultibaseData::new()
    ///     .with_header(MultibaseHeader::Base58Btc)
    ///     .with_data([
    ///         0xc6, 0xf6, 0x22, 0x5d, 0x9f, 0xaf, 0x21, 0xc2, 0x23, 0x3b, 0x2f, 0x8d, 0xda, 0x7f,
    ///         0xd1, 0xbc, 0x92, 0x53, 0x13, 0xcc, 0xb8, 0x80, 0x7b, 0xba, 0x51, 0xcd, 0xd2, 0x9a,
    ///         0xdd, 0x20, 0x03, 0x94, 0x39, 0x36, 0x2e, 0x00, 0x60, 0xd6, 0x81, 0x5e, 0x26, 0xb0,
    ///         0xc8, 0x96, 0x3f, 0x56, 0xb5, 0x6b, 0xfb, 0x75, 0x60, 0x8a, 0xaf, 0x35, 0x00, 0x1e,
    ///         0x28, 0xf2, 0x26, 0x03, 0xfa, 0xf6, 0x21, 0x0c,
    ///     ]);
    ///
    /// let proof = DataIntegrityProof::new_inner()
    ///     .with_created(proof_created.parse::<DateTime>().unwrap())
    ///     .with_cryptosuite(proof_cryptosuite)
    ///     .with_proof_purpose(proof_purpose)
    ///     .with_proof_value(proof_data)
    ///     .with_verification_method(verification_method);
    ///
    /// let resolve = Resolve::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_capability(capability)
    ///     .with_to([to0, to1, to2, to3])
    ///     .with_proof(proof);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&resolve).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Resolve>(json_str.as_str()).unwrap(),
    ///     resolve
    /// );
    /// # }
    /// ```
    Resolve: crate::ActivityType::Resolve {
        capability: Option<GrantItem>,
    }
}

field_access! {
    Resolve {
        /// Represents the [Grant](crate::Grant) capability of the `actor` who resolved the [Ticket](crate::Ticket).
        capability: option_ref { GrantItem },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{
        Cryptosuite, DataIntegrityProof, DateTime, Iri, MultibaseData, MultibaseHeader,
    };

    #[test]
    fn test_resolve() {
        let id =
            Iri::try_from("https://grape.fr33domlover.site/people/WZpnG/outbox/GQvnR").unwrap();
        let actor = Iri::try_from("https://grape.fr33domlover.site/people/WZpnG").unwrap();
        let object =
            Iri::try_from("https://fig.fr33domlover.site/decks/W058b/tickets/3bPVn").unwrap();
        let capability =
            Iri::try_from("https://grape.fr33domlover.site/groups/OZLdZ/outbox/ZLdDZ").unwrap();

        let to0 = Iri::try_from("https://grape.fr33domlover.site/people/WZpnG/followers").unwrap();
        let to1 = Iri::try_from("https://fig.fr33domlover.site/decks/W058b").unwrap();
        let to2 = Iri::try_from("https://fig.fr33domlover.site/decks/W058b/followers").unwrap();
        let to3 =
            Iri::try_from("https://fig.fr33domlover.site/decks/W058b/tickets/3bPVn/followers")
                .unwrap();

        let proof_created = "2024-07-09T07:02:45.038760903Z";
        let proof_cryptosuite = Cryptosuite::EddsaJcs2022;
        let proof_purpose = Iri::try_from("assertionMethod").unwrap();
        let proof_value = "z4yiaWtjPeUjBvoXpHffjguVd8uHsSfYr4zxxFJbZwPVLtRNCk3uMKtnC3nRZ8vRM3h36B6VLrFagVerxcymhj2NP";
        let verification_method = Iri::try_from("https://grape.fr33domlover.site/akey1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Resolve",
  "id": "{id}",
  "to": [
    "{to0}",
    "{to1}",
    "{to2}",
    "{to3}"
  ],
  "proof": {{
    "type": "DataIntegrityProof",
    "cryptosuite": "{proof_cryptosuite}",
    "proofValue": "{proof_value}",
    "proofPurpose": "{proof_purpose}",
    "created": "{proof_created}",
    "verificationMethod": "{verification_method}"
  }},
  "actor": "{actor}",
  "object": "{object}",
  "capability": "{capability}"
}}"#
        );

        let context = context::forgefed_context();

        let proof_data = MultibaseData::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_data([
                0xc6, 0xf6, 0x22, 0x5d, 0x9f, 0xaf, 0x21, 0xc2, 0x23, 0x3b, 0x2f, 0x8d, 0xda, 0x7f,
                0xd1, 0xbc, 0x92, 0x53, 0x13, 0xcc, 0xb8, 0x80, 0x7b, 0xba, 0x51, 0xcd, 0xd2, 0x9a,
                0xdd, 0x20, 0x03, 0x94, 0x39, 0x36, 0x2e, 0x00, 0x60, 0xd6, 0x81, 0x5e, 0x26, 0xb0,
                0xc8, 0x96, 0x3f, 0x56, 0xb5, 0x6b, 0xfb, 0x75, 0x60, 0x8a, 0xaf, 0x35, 0x00, 0x1e,
                0x28, 0xf2, 0x26, 0x03, 0xfa, 0xf6, 0x21, 0x0c,
            ]);

        let proof = DataIntegrityProof::new_inner()
            .with_created(proof_created.parse::<DateTime>().unwrap())
            .with_cryptosuite(proof_cryptosuite)
            .with_proof_purpose(proof_purpose)
            .with_proof_value(proof_data)
            .with_verification_method(verification_method);

        let resolve = Resolve::new()
            .with_context_property(context)
            .with_id(id)
            .with_actor(actor)
            .with_object(object)
            .with_capability(capability)
            .with_to([to0, to1, to2, to3])
            .with_proof(proof);

        assert_eq!(serde_json::to_string_pretty(&resolve).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Resolve>(json_str.as_str()).unwrap(),
            resolve
        );
    }
}
