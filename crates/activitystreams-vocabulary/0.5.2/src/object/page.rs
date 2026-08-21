use crate::create_object;

create_object! {
    /// Represents a Web Page.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Page};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Omaha Weather Report").unwrap();
    /// let url = Iri::try_from("http://example.org/weather-in-omaha.html").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Page",
    ///   "name": "{name}",
    ///   "url": "{url}"
    /// }}"#
    ///     );
    ///
    /// let page = Page::new().with_url(url).with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&page).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Page>(json_str.as_str()).unwrap(),
    ///     page
    /// );
    /// # }
    /// ```
    Page: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("Omaha Weather Report").unwrap();
        let url = Iri::try_from("http://example.org/weather-in-omaha.html").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Page",
  "name": "{name}",
  "url": "{url}"
}}"#
        );

        let page = Page::new().with_url(url).with_name(name);

        assert_eq!(serde_json::to_string_pretty(&page).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Page>(json_str.as_str()).unwrap(),
            page
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Page>(json_str).is_err());
    }
}
