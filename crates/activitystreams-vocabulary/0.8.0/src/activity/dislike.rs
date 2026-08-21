use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` dislikes the `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Dislike, Iri};
    ///
    /// # fn test_activity() {
    /// let summary = "Sally disliked a post";
    /// let actor = Iri::try_from("http://sally.example.org").unwrap();
    /// let object = Iri::try_from("http://example.org/posts/1").unwrap();
    ///
    /// let dislike = Dislike::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor.clone())
    ///     .with_object(object.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Dislike",
    ///   "summary": "{summary}",
    ///   "actor": "{actor}",
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&dislike).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Dislike>(json_str.as_str()).unwrap(),
    ///     dislike
    /// );
    /// # }
    /// ```
    Dislike {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Iri;

    #[test]
    fn test_activity() {
        let summary = "Sally disliked a post";
        let actor = Iri::try_from("http://sally.example.org").unwrap();
        let object = Iri::try_from("http://example.org/posts/1").unwrap();

        let dislike = Dislike::new()
            .with_summary(summary)
            .with_actor(actor.clone())
            .with_object(object.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Dislike",
  "summary": "{summary}",
  "actor": "{actor}",
  "object": "{object}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&dislike).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Dislike>(json_str.as_str()).unwrap(),
            dislike
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object","id":"{id}","name":"{name}"}}"#;

        assert!(serde_json::from_str::<Dislike>(json_str).is_err());
    }
}
