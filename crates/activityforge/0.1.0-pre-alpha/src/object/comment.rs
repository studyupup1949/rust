/// Represents a comment in a merge request code review.
///
/// Every comment in the tree of comments within a review,
/// including both the review comments by the reviewer and the replies on them,
/// is represented as a regular [Note](activitystreams_vocabulary::Note) (much like issue comments),
/// except its attachments may contain code suggestions.
///
/// # Example
///
/// ```rust
/// use activityforge::{Comment, Suggestion, context};
/// use activitystreams_vocabulary::{Collection, Content, Iri, MimeType};
///
/// # fn main() {
/// let attributed_to = Iri::try_from("https://example.dev/ba55").unwrap();
/// let context = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review-thread/1").unwrap();
/// let in_reply_to = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review-thread/1/comment/1").unwrap();
///
/// let content = "<p>Small suggestion</p>";
/// let media_type = MimeType::TextHtml;
///
/// let source_content = "Small suggestion";
/// let source_type = MimeType::TextPlain;
///
/// let suggestion_id = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/suggestion/1").unwrap();
///
/// let json_str = format!(
/// r#"{{
///   "@context": [
///     "https://www.w3.org/ns/activitystreams",
///     "https://forgefed.org/ns"
///   ],
///   "type": "Note",
///   "attachment": {{
///     "type": "Collection",
///     "items": [
///       {{
///         "type": "Suggestion",
///         "id": "{suggestion_id}"
///       }}
///     ],
///     "totalItems": 1
///   }},
///   "attributedTo": "{attributed_to}",
///   "content": "{content}",
///   "context": "{context}",
///   "inReplyTo": "{in_reply_to}",
///   "mediaType": "{media_type}",
///   "source": {{
///     "content": "{source_content}",
///     "mediaType": "{source_type}"
///   }}
/// }}"#
///             );
///
/// let context_property = context::forgefed_context();
///
/// let source = Content::new()
///     .with_content(source_content)
///     .with_media_type(source_type);
///
/// let suggestion = Suggestion::new_inner()
///     .with_id(suggestion_id);
///
/// let attachment = Collection::new_inner()
///     .with_total_items(1u64)
///     .with_items([suggestion]);
///
/// let comment = Comment::new()
///     .with_context_property(context_property)
///     .with_attributed_to(attributed_to)
///     .with_context(context)
///     .with_in_reply_to(in_reply_to)
///     .with_content(content)
///     .with_media_type(media_type)
///     .with_source(source)
///     .with_attachment(attachment);
///
/// assert_eq!(serde_json::to_string_pretty(&comment).unwrap(), json_str);
/// assert_eq!(
///     serde_json::from_str::<Comment>(json_str.as_str()).unwrap(),
///     comment
/// );
/// # }
/// ```
pub use activitystreams_vocabulary::Note as Comment;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Suggestion, context};

    use activitystreams_vocabulary::{Collection, Content, Iri, MimeType};

    #[test]
    fn test_comment() {
        let attributed_to = Iri::try_from("https://example.dev/ba55").unwrap();
        let context =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review-thread/1").unwrap();
        let in_reply_to =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review-thread/1/comment/1")
                .unwrap();

        let content = "<p>Small suggestion</p>";
        let media_type = MimeType::TextHtml;

        let source_content = "Small suggestion";
        let source_type = MimeType::TextPlain;

        let suggestion_id =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/suggestion/1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Note",
  "attachment": {{
    "type": "Collection",
    "items": [
      {{
        "type": "Suggestion",
        "id": "{suggestion_id}"
      }}
    ],
    "totalItems": 1
  }},
  "attributedTo": "{attributed_to}",
  "content": "{content}",
  "context": "{context}",
  "inReplyTo": "{in_reply_to}",
  "mediaType": "{media_type}",
  "source": {{
    "content": "{source_content}",
    "mediaType": "{source_type}"
  }}
}}"#
        );

        let context_property = context::forgefed_context();

        let source = Content::new()
            .with_content(source_content)
            .with_media_type(source_type);

        let suggestion = Suggestion::new_inner().with_id(suggestion_id);

        let attachment = Collection::new_inner()
            .with_total_items(1u64)
            .with_items([suggestion]);

        let comment = Comment::new()
            .with_context_property(context_property)
            .with_attributed_to(attributed_to)
            .with_context(context)
            .with_in_reply_to(in_reply_to)
            .with_content(content)
            .with_media_type(media_type)
            .with_source(source)
            .with_attachment(attachment);

        assert_eq!(serde_json::to_string_pretty(&comment).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Comment>(json_str.as_str()).unwrap(),
            comment
        );
    }
}
