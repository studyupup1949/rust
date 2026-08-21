use crate::create_object;

create_object! {
    /// Represents a document of any kind.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Document, Iri, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("4Q Sales Forecast").unwrap();
    /// let url = Iri::try_from("http://example.org/4q-sales-forecast.pdf").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Document",
    ///   "name": "{name}",
    ///   "url": "{url}"
    /// }}"#);
    ///
    /// let document = Document::new().with_name(name).with_url(url);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&document).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Document>(json_str.as_str()).unwrap(),
    ///     document
    /// );
    /// # }
    /// ```
    Document: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name};

    #[test]
    fn test_document() {
        let name = Name::try_from("4Q Sales Forecast").unwrap();
        let url = Iri::try_from("http://example.org/4q-sales-forecast.pdf").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Document",
  "name": "{name}",
  "url": "{url}"
}}"#
        );

        let document = Document::new().with_name(name).with_url(url);

        assert_eq!(serde_json::to_string_pretty(&document).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Document>(json_str.as_str()).unwrap(),
            document
        );
    }

    #[test]
    fn test_invalid_document() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Document>(json_str).is_err());
    }
}
