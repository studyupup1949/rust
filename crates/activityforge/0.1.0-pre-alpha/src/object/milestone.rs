use activitystreams_vocabulary::{Collection, create_object, field_access};

create_object! {
    /// Describes a project milestone, under which issues, releases, and merge requests can be grouped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Milestone, Ticket, context};
    /// use activitystreams_vocabulary::{Collection, DateTime, Iri, MimeType, Name, Content};
    ///
    /// # fn main() {
    /// let name = Name::try_from("milestone #1").unwrap();
    /// let content = "<p>The first project Milestone</p>";
    /// let context = Iri::try_from("https://example.dev/alice/myrepo/roadmap").unwrap();
    /// let start_time = "2026-04-20T13:37:42Z";
    /// let end_time = "2026-07-10T13:37:42Z";
    /// let published = "2026-04-20T13:37:42Z";
    /// let source = "The first project Milestone";
    /// let is_resolved = false;
    ///
    /// let ticket_id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
    /// let ticket_context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
    /// let ticket_attributed_to = Iri::try_from("https://dev.community/bob").unwrap();
    /// let ticket_summary = "Nothing works!";
    /// let ticket_content = "<p>Please fix. <i>Everything</i> is broken!</p>";
    /// let ticket_media_type = MimeType::TextHtml;
    ///
    /// let ticket_source_content = "Please fix. *Everything* is broken!";
    /// let ticket_source_type = MimeType::TextMarkdownCommonMark;
    ///
    /// let ticket_assignments =
    ///     Iri::try_from("https://example.dev/alice/myrepo/issues/42/assignments").unwrap();
    /// let ticket_is_resolved = false;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Milestone",
    ///   "name": "{name}",
    ///   "content": "{content}",
    ///   "context": "{context}",
    ///   "startTime": "{start_time}",
    ///   "endTime": "{end_time}",
    ///   "published": "{published}",
    ///   "source": "{source}",
    ///   "isResolved": {is_resolved},
    ///   "milestoneTickets": {{
    ///     "type": "Collection",
    ///     "totalItems": 1,
    ///     "items": [
    ///       {{
    ///         "type": "Ticket",
    ///         "id": "{ticket_id}",
    ///         "attributedTo": "{ticket_attributed_to}",
    ///         "summary": "{ticket_summary}",
    ///         "content": "{ticket_content}",
    ///         "context": "{ticket_context}",
    ///         "mediaType": "{ticket_media_type}",
    ///         "source": {{
    ///           "content": "{ticket_source_content}",
    ///           "mediaType": "{ticket_source_type}"
    ///         }},
    ///         "assignments": "{ticket_assignments}",
    ///         "isResolved": {ticket_is_resolved}
    ///       }}
    ///     ]
    ///   }}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let ticket_source = Content::new()
    ///     .with_content(ticket_source_content)
    ///     .with_media_type(ticket_source_type);
    ///
    /// let ticket = Ticket::new_inner()
    ///     .with_id(ticket_id)
    ///     .with_context(ticket_context)
    ///     .with_attributed_to(ticket_attributed_to)
    ///     .with_summary(ticket_summary)
    ///     .with_content(ticket_content)
    ///     .with_media_type(ticket_media_type)
    ///     .with_source(ticket_source)
    ///     .with_assignments(ticket_assignments)
    ///     .with_is_resolved(ticket_is_resolved);
    ///
    /// let tickets = Collection::new_inner()
    ///     .with_total_items(1u64)
    ///     .with_items([ticket]);
    ///
    /// let milestone = Milestone::new()
    ///     .with_context_property(context_property)
    ///     .with_name(name)
    ///     .with_content(content)
    ///     .with_source(source)
    ///     .with_published(published.parse::<DateTime>().unwrap())
    ///     .with_start_time(start_time.parse::<DateTime>().unwrap())
    ///     .with_end_time(end_time.parse::<DateTime>().unwrap())
    ///     .with_context(context)
    ///     .with_is_resolved(is_resolved)
    ///     .with_milestone_tickets(tickets);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&milestone).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Milestone>(json_str.as_str()).unwrap(),
    ///     milestone
    /// );
    /// # }
    /// ```
    Milestone: crate::ObjectType::Milestone {
        #[serde(skip_serializing_if = "Option::is_none")]
        is_resolved: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        milestone_releases: Option<Collection>,
        #[serde(skip_serializing_if = "Option::is_none")]
        milestone_tickets: Option<Collection>,
    }
}

field_access! {
    Milestone {
        /// Whether the milestone is closed.
        is_resolved: option { bool },
    }
}

field_access! {
    Milestone {
        /// Collection of [Release](crate::Release)s associated with this milestone.
        milestone_releases: option_ref { Collection },
        /// Collection of [Ticket](crate::Ticket)s that belong to this milestone.
        milestone_tickets: option_ref { Collection },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Ticket, context};

    use activitystreams_vocabulary::{Content, DateTime, Iri, MimeType, Name};

    #[test]
    fn test_milestone() {
        let name = Name::try_from("milestone #1").unwrap();
        let content = "<p>The first project Milestone</p>";
        let source = "The first project Milestone";
        let published = "2026-04-20T13:37:42Z";
        let start_time = "2026-04-20T13:37:42Z";
        let end_time = "2026-07-10T13:37:42Z";
        let context = Iri::try_from("https://example.dev/alice/myrepo/roadmap").unwrap();
        let is_resolved = false;

        let ticket_id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
        let ticket_context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
        let ticket_attributed_to = Iri::try_from("https://dev.community/bob").unwrap();
        let ticket_summary = "Nothing works!";
        let ticket_content = "<p>Please fix. <i>Everything</i> is broken!</p>";
        let ticket_media_type = MimeType::TextHtml;

        let ticket_source_content = "Please fix. *Everything* is broken!";
        let ticket_source_type = MimeType::TextMarkdownCommonMark;

        let ticket_assignments =
            Iri::try_from("https://example.dev/alice/myrepo/issues/42/assignments").unwrap();
        let ticket_is_resolved = false;

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Milestone",
  "name": "{name}",
  "content": "{content}",
  "context": "{context}",
  "startTime": "{start_time}",
  "endTime": "{end_time}",
  "published": "{published}",
  "source": "{source}",
  "isResolved": {is_resolved},
  "milestoneTickets": {{
    "type": "Collection",
    "totalItems": 1,
    "items": [
      {{
        "type": "Ticket",
        "id": "{ticket_id}",
        "attributedTo": "{ticket_attributed_to}",
        "summary": "{ticket_summary}",
        "content": "{ticket_content}",
        "context": "{ticket_context}",
        "mediaType": "{ticket_media_type}",
        "source": {{
          "content": "{ticket_source_content}",
          "mediaType": "{ticket_source_type}"
        }},
        "assignments": "{ticket_assignments}",
        "isResolved": {ticket_is_resolved}
      }}
    ]
  }}
}}"#
        );

        let context_property = context::forgefed_context();

        let ticket_source = Content::new()
            .with_content(ticket_source_content)
            .with_media_type(ticket_source_type);

        let ticket = Ticket::new_inner()
            .with_id(ticket_id)
            .with_context(ticket_context)
            .with_attributed_to(ticket_attributed_to)
            .with_summary(ticket_summary)
            .with_content(ticket_content)
            .with_media_type(ticket_media_type)
            .with_source(ticket_source)
            .with_assignments(ticket_assignments)
            .with_is_resolved(ticket_is_resolved);

        let tickets = Collection::new_inner()
            .with_total_items(1u64)
            .with_items([ticket]);

        let milestone = Milestone::new()
            .with_context_property(context_property)
            .with_name(name)
            .with_content(content)
            .with_context(context)
            .with_start_time(start_time.parse::<DateTime>().unwrap())
            .with_end_time(end_time.parse::<DateTime>().unwrap())
            .with_published(published.parse::<DateTime>().unwrap())
            .with_source(source)
            .with_is_resolved(is_resolved)
            .with_milestone_tickets(tickets);

        assert_eq!(serde_json::to_string_pretty(&milestone).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Milestone>(json_str.as_str()).unwrap(),
            milestone
        );
    }
}
