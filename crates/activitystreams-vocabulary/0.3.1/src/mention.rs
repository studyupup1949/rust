use crate::create_link;

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
    Mention: LinkType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreType, Iri, Name, VocabularyTypes};

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
        let mention = Mention::<VocabularyTypes>::new().with_kind(CoreType::Link);

        assert!(serde_json::to_string(&mention).is_err());
        assert!(serde_json::from_str::<Mention>(json_str).is_err());
    }
}
