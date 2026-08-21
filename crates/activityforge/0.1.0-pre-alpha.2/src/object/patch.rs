use activitystreams_vocabulary::create_object;

create_object! {
    /// Represents a named set of changes that are being proposed for applying to a specific [Repository](crate::Repository).
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Patch, context};
    /// use activitystreams_vocabulary::{Iri, MimeType};
    ///
    /// # fn main() {
    /// let id =
    ///     Iri::try_from("https://dev.example/aviva/game-of-life/pulls/825/versions/1/patches/1").unwrap();
    /// let attributed_to = Iri::try_from("https://forge.example/luke").unwrap();
    /// let context =
    ///     Iri::try_from("https://dev.example/aviva/game-of-life/pulls/825/versions/1").unwrap();
    /// let media_type = MimeType::ApplicationGitPatch;
    /// let content = "From c9ae5f4ff4a330b6e1196ceb7db1665bd4c1...";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Patch",
    ///   "id": "{id}",
    ///   "attributedTo": "{attributed_to}",
    ///   "content": "{content}",
    ///   "context": "{context}",
    ///   "mediaType": "{media_type}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let patch = Patch::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_content(content)
    ///     .with_context(context)
    ///     .with_attributed_to(attributed_to)
    ///     .with_media_type(media_type);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&patch).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Patch>(json_str.as_str()).unwrap(),
    ///     patch
    /// );
    /// # }
    /// ```
    Patch: crate::ObjectType::Patch {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{Iri, MimeType};

    #[test]
    fn test_patch() {
        let id =
            Iri::try_from("https://dev.example/aviva/game-of-life/pulls/825/versions/1/patches/1")
                .unwrap();
        let attributed_to = Iri::try_from("https://forge.example/luke").unwrap();
        let context =
            Iri::try_from("https://dev.example/aviva/game-of-life/pulls/825/versions/1").unwrap();
        let media_type = MimeType::ApplicationGitPatch;
        let content = "From c9ae5f4ff4a330b6e1196ceb7db1665bd4c1...";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Patch",
  "id": "{id}",
  "attributedTo": "{attributed_to}",
  "content": "{content}",
  "context": "{context}",
  "mediaType": "{media_type}"
}}"#
        );

        let context_property = context::forgefed_context();

        let patch = Patch::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_content(content)
            .with_context(context)
            .with_attributed_to(attributed_to)
            .with_media_type(media_type);

        assert_eq!(serde_json::to_string_pretty(&patch).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Patch>(json_str.as_str()).unwrap(),
            patch
        );
    }
}
