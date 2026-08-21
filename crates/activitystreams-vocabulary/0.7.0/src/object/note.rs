use crate::create_object;

create_object! {
    /// Represents a short written work typically less than a single paragraph in length.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Note};
    ///
    /// # fn main() {
    /// let name = Name::try_from("A Word of Warning").unwrap();
    /// let content = "Looks like it is going to rain today. Bring an umbrella!";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Note",
    ///   "name": "{name}",
    ///   "content": "{content}"
    /// }}"#
    ///     );
    ///
    /// let note = Note::new().with_name(name).with_content(content);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Note>(json_str.as_str()).unwrap(),
    ///     note
    /// );
    /// # }
    /// ```
    Note: ObjectType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Name;

    #[test]
    fn test_valid() {
        let name = Name::try_from("A Word of Warning").unwrap();
        let content = "Looks like it is going to rain today. Bring an umbrella!";

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Note",
  "name": "{name}",
  "content": "{content}"
}}"#
        );

        let note = Note::new().with_name(name).with_content(content);

        assert_eq!(serde_json::to_string_pretty(&note).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Note>(json_str.as_str()).unwrap(),
            note
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Note>(json_str).is_err());
    }
}
