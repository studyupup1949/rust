use activitystreams_vocabulary::{LinkItems, create_object, field_access};

create_object! {
    /// Represents an item that requires work or attention.
    ///
    /// Tickets exist in the context of a project (which may or may not be a version-control repository),
    /// and are used to track ideas, proposals, tasks, bugs and more.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Ticket, context};
    /// use activitystreams_vocabulary::{Iri, MimeType, Content};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
    /// let context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
    /// let attributed_to = Iri::try_from("https://dev.community/bob").unwrap();
    /// let summary = "Nothing works!";
    /// let content = "<p>Please fix. <i>Everything</i> is broken!</p>";
    /// let media_type = MimeType::TextHtml;
    ///
    /// let source_content = "Please fix. *Everything* is broken!";
    /// let source_type = MimeType::TextMarkdownCommonMark;
    ///
    /// let assignments = Iri::try_from("https://example.dev/alice/myrepo/issues/42/assignments").unwrap();
    /// let is_resolved = false;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Ticket",
    ///   "id": "{id}",
    ///   "attributedTo": "{attributed_to}",
    ///   "summary": "{summary}",
    ///   "content": "{content}",
    ///   "context": "{context}",
    ///   "mediaType": "{media_type}",
    ///   "source": {{
    ///     "content": "{source_content}",
    ///     "mediaType": "{source_type}"
    ///   }},
    ///   "assignments": "{assignments}",
    ///   "isResolved": {is_resolved}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let source = Content::new().with_content(source_content).with_media_type(source_type);
    ///
    /// let ticket = Ticket::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_context(context)
    ///     .with_attributed_to(attributed_to)
    ///     .with_summary(summary)
    ///     .with_content(content)
    ///     .with_media_type(media_type)
    ///     .with_source(source)
    ///     .with_assignments(assignments)
    ///     .with_is_resolved(is_resolved);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&ticket).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Ticket>(json_str.as_str()).unwrap(),
    ///     ticket
    /// );
    /// # }
    /// ```
    Ticket: crate::ObjectType::Ticket {
        #[serde(skip_serializing_if = "Option::is_none")]
        assignments: Option<LinkItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_resolved: Option<bool>,
    }
}

field_access! {
    Ticket {
        /// Represents a reference to the assignments for the ticket.
        assignments: option_ref { LinkItems },
    }
}

field_access! {
    Ticket {
        /// Represents whether the ticket is resolved.
        is_resolved: option { bool },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{Content, Iri, MimeType};

    #[test]
    fn test_ticket() {
        let id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
        let context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
        let attributed_to = Iri::try_from("https://dev.community/bob").unwrap();
        let summary = "Nothing works!";
        let content = "<p>Please fix. <i>Everything</i> is broken!</p>";
        let media_type = MimeType::TextHtml;

        let source_content = "Please fix. *Everything* is broken!";
        let source_type = MimeType::TextMarkdownCommonMark;

        let assignments =
            Iri::try_from("https://example.dev/alice/myrepo/issues/42/assignments").unwrap();
        let is_resolved = false;

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Ticket",
  "id": "{id}",
  "attributedTo": "{attributed_to}",
  "summary": "{summary}",
  "content": "{content}",
  "context": "{context}",
  "mediaType": "{media_type}",
  "source": {{
    "content": "{source_content}",
    "mediaType": "{source_type}"
  }},
  "assignments": "{assignments}",
  "isResolved": {is_resolved}
}}"#
        );

        let context_property = context::forgefed_context();

        let source = Content::new()
            .with_content(source_content)
            .with_media_type(source_type);

        let ticket = Ticket::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_context(context)
            .with_attributed_to(attributed_to)
            .with_summary(summary)
            .with_content(content)
            .with_media_type(media_type)
            .with_source(source)
            .with_assignments(assignments)
            .with_is_resolved(is_resolved);

        assert_eq!(serde_json::to_string_pretty(&ticket).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Ticket>(json_str.as_str()).unwrap(),
            ticket
        );
    }
}
