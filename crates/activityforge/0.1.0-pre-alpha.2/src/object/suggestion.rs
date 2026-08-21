use activitystreams_vocabulary::create_object;

create_object! {
    /// A suggested edit within a review on a merge request.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Suggestion, context};
    /// use activitystreams_vocabulary::{Content, Iri, MimeType, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Replacing an old line with a new line.").unwrap();
    /// let attributed_to = Iri::try_from("https://example.dev/ba55").unwrap();
    /// let context =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1#issueComment=1").unwrap();
    ///
    /// let source_content = r#"
    /// - old line
    /// + new line
    /// "#;
    ///
    /// let source_type = MimeType::TextDiff;
    /// let source_content_json = serde_json::to_string(&source_content).unwrap();
    /// let source = Content::new()
    ///     .with_content(source_content)
    ///     .with_media_type(source_type);
    ///
    /// let content = r#"
    /// <code class="diff">
    ///   <p class="diff-old">old line</p>
    ///   <p class="diff-new">new line</p>
    /// </code>
    /// "#;
    /// let content_json = serde_json::to_string(&content).unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Suggestion",
    ///   "name": "{name}",
    ///   "attributedTo": "{attributed_to}",
    ///   "content": {content_json},
    ///   "context": "{context}",
    ///   "source": {{
    ///     "content": {source_content_json},
    ///     "mediaType": "{source_type}"
    ///   }}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let suggestion = Suggestion::new()
    ///     .with_context_property(context_property)
    ///     .with_name(name)
    ///     .with_attributed_to(attributed_to)
    ///     .with_context(context)
    ///     .with_source(source)
    ///     .with_content(content);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&suggestion).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Suggestion>(json_str.as_str()).unwrap(),
    ///     suggestion
    /// );
    /// # }
    /// ```
    Suggestion: crate::ObjectType::Suggestion {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{Content, Iri, MimeType, Name};

    #[test]
    fn test_suggestion() {
        let name = Name::try_from("Replacing an old line with a new line.").unwrap();
        let attributed_to = Iri::try_from("https://example.dev/ba55").unwrap();
        let context =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1#issueComment=1").unwrap();

        let source_content = r#"
- old line
+ new line
"#;
        let source_type = MimeType::TextDiff;
        let source_content_json = serde_json::to_string(&source_content).unwrap();
        let source = Content::new()
            .with_content(source_content)
            .with_media_type(source_type);

        let content = r#"
<code class="diff">
  <p class="diff-old">old line</p>
  <p class="diff-new">new line</p>
</code>
"#;
        let content_json = serde_json::to_string(&content).unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Suggestion",
  "name": "{name}",
  "attributedTo": "{attributed_to}",
  "content": {content_json},
  "context": "{context}",
  "source": {{
    "content": {source_content_json},
    "mediaType": "{source_type}"
  }}
}}"#
        );

        let context_property = context::forgefed_context();

        let suggestion = Suggestion::new()
            .with_context_property(context_property)
            .with_name(name)
            .with_attributed_to(attributed_to)
            .with_context(context)
            .with_source(source)
            .with_content(content);

        assert_eq!(serde_json::to_string_pretty(&suggestion).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Suggestion>(json_str.as_str()).unwrap(),
            suggestion
        );
    }
}
