use crate::{ActivityType, create_activity};

create_activity! {
    ///	Indicates that the `actor` is "flagging" the `object`.
    ///
    ///	Flagging is defined in the sense common to many social platforms as reporting content as being inappropriate for any number of reasons.
    ///
    ///	# Example
    ///
    ///	```rust
    /// use activitystreams_vocabulary::{Flag, Iri, Note};
    ///
    /// # fn main() {
    /// let summary = "Sally flagged an inappropriate note";
    /// let actor = Iri::try_from("http://sally.example.org").unwrap();
    ///
    /// let object_content = "An inappropriate note";
    /// let object = Note::new_inner().with_content(object_content);
    ///
    /// let flag = Flag::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor.clone())
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Flag",
    ///   "summary": "{summary}",
    ///   "actor": "{actor}",
    ///   "object": {{
    ///     "type": "Note",
    ///     "content": "{object_content}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&flag).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Flag>(json_str.as_str()).unwrap(),
    ///     flag
    /// );
    /// # }
    ///	```
    Flag {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Note};

    #[test]
    fn test_activity() {
        let summary = "Sally flagged an inappropriate note";
        let actor = Iri::try_from("http://sally.example.org").unwrap();

        let object_content = "An inappropriate note";
        let object = Note::new_inner().with_content(object_content);

        let flag = Flag::new()
            .with_summary(summary)
            .with_actor(actor.clone())
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Flag",
  "summary": "{summary}",
  "actor": "{actor}",
  "object": {{
    "type": "Note",
    "content": "{object_content}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&flag).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Flag>(json_str.as_str()).unwrap(),
            flag
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Flag>(json_str).is_err());
    }
}
