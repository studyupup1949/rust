use crate::{ActivityType, create_activity};

create_activity! {
    ///  Indicates that the `actor` is calling the target's attention the `object`.
    ///
    /// The `origin` typically has no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Block, Iri};
    ///
    /// # fn main() {
    /// let summary = "Sally blocked Joe";
    /// let actor = Iri::try_from("http://sally.example.org").unwrap();
    /// let object = Iri::try_from("http://joe.example.org").unwrap();
    ///
    /// let block = Block::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor.clone())
    ///     .with_object(object.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Block",
    ///   "summary": "{summary}",
    ///   "actor": "{actor}",
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&block).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Block>(json_str.as_str()).unwrap(),
    ///     block
    /// );
    /// # }
    /// ```
    Block {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Iri;

    #[test]
    fn test_activity() {
        let summary = "Sally blocked Joe";
        let actor = Iri::try_from("http://sally.example.org").unwrap();
        let object = Iri::try_from("http://joe.example.org").unwrap();

        let block = Block::new()
            .with_summary(summary)
            .with_actor(actor.clone())
            .with_object(object.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Block",
  "summary": "{summary}",
  "actor": "{actor}",
  "object": "{object}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&block).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Block>(json_str.as_str()).unwrap(),
            block
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object","id":"{id}","name":"{name}"}}"#;

        assert!(serde_json::from_str::<Block>(json_str).is_err());
    }
}
