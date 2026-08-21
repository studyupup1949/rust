use crate::create_object;

create_object! {
    /// Represents any kind of multi-paragraph written work.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Article, Iri, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("What a Crazy Day I Had").unwrap();
    /// let content = "<div>... you will never believe ...</div>";
    /// let attributed_to = Iri::try_from("http://sally.example.org").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Article",
    ///   "name": "{name}",
    ///   "attributedTo": "{attributed_to}",
    ///   "content": "{content}"
    /// }}"#);
    ///
    /// let article = Article::new()
    ///     .with_name(name)
    ///     .with_content(content)
    ///     .with_attributed_to(attributed_to);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&article).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Article>(json_str.as_str()).unwrap(),
    ///     article
    /// );
    /// # }
    /// ```
    Article: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name};

    #[test]
    fn test_valid() {
        let name = Name::try_from("What a Crazy Day I Had").unwrap();
        let content = "<div>... you will never believe ...</div>";
        let attributed_to = Iri::try_from("http://sally.example.org").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Article",
  "name": "{name}",
  "attributedTo": "{attributed_to}",
  "content": "{content}"
}}"#
        );

        let article = Article::new()
            .with_name(name)
            .with_content(content)
            .with_attributed_to(attributed_to);

        assert_eq!(serde_json::to_string_pretty(&article).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Article>(json_str.as_str()).unwrap(),
            article
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Article>(json_str).is_err());
    }
}
