use crate::{LinkType, create_link, derived_kind_serde};

create_link! {
    /// A specialized [Link](crate::Link) that represents an `@mention`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{CoreType, Iri, Mention, Name};
    ///
    /// # fn main() {
    /// let href = Iri::try_from("http://example.org/joe").unwrap();
    /// let name = Name::try_from("Joe").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Mention",
    ///   "href": "{href}",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let mention = Mention::new().with_name(name).with_href(href);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&mention).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Mention>(json_str.as_str()).unwrap(),
    ///     mention
    /// );
    /// # }
    /// ```
    Mention:
        #[serde(serialize_with = "obj_serde::ser")]
        LinkType {}
}

derived_kind_serde!(crate::LinkType::Mention);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreType, Iri, Name};

    #[test]
    fn test_valid() {
        let href = Iri::try_from("http://example.org/joe").unwrap();
        let name = Name::try_from("Joe").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Mention",
  "href": "{href}",
  "name": "{name}"
}}"#
        );

        let mention = Mention::new().with_name(name).with_href(href);

        assert_eq!(serde_json::to_string_pretty(&mention).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Mention>(json_str.as_str()).unwrap(),
            mention
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Mention>(json_str).is_err());
        assert!(
            serde_json::to_string(&Mention::new().with_kind(CoreType::Link.to_vocabulary_types()))
                .is_err()
        );
    }
}
