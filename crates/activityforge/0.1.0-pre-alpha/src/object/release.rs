use activitystreams_vocabulary::{Collection, create_object, field_access};

create_object! {
    /// Describes a software release.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Milestone, Release, Ticket, context};
    /// use activitystreams_vocabulary::{Collection, Content, DateTime, Iri, MimeType, Name};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Release v0.1.0").unwrap();
    /// let is_pre_release = false;
    ///
    /// let milestone_name = Name::try_from("milestone #1").unwrap();
    /// let milestone_published = "2026-04-20T13:37:42Z";
    /// let milestone_is_resolved = false;
    ///
    /// let release_id = Iri::try_from("https://example.dev/alice/myrepo/releases/1").unwrap();
    ///
    /// let ticket_id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
    /// let ticket_context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();
    ///
    /// let ticket_source_content = "Please fix. *Everything* is broken!";
    /// let ticket_source_type = MimeType::TextMarkdownCommonMark;
    ///
    /// let ticket_is_resolved = false;
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Release",
    ///   "name": "{name}",
    ///   "isPreRelease": {is_pre_release},
    ///   "releaseMilestones": {{
    ///     "type": "Collection",
    ///     "totalItems": 1,
    ///     "items": [
    ///       {{
    ///         "type": "Milestone",
    ///         "name": "{milestone_name}",
    ///         "published": "{milestone_published}",
    ///         "isResolved": {milestone_is_resolved},
    ///         "milestoneReleases": {{
    ///           "type": "Collection",
    ///           "totalItems": 1,
    ///           "items": [
    ///             {{
    ///               "type": "Release",
    ///               "id": "{release_id}"
    ///             }}
    ///           ]
    ///         }},
    ///         "milestoneTickets": {{
    ///           "type": "Collection",
    ///           "totalItems": 1,
    ///           "items": [
    ///             {{
    ///               "type": "Ticket",
    ///               "id": "{ticket_id}",
    ///               "context": "{ticket_context}",
    ///               "source": {{
    ///                 "content": "{ticket_source_content}",
    ///                 "mediaType": "{ticket_source_type}"
    ///               }},
    ///               "isResolved": {ticket_is_resolved}
    ///             }}
    ///           ]
    ///         }}
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
    /// let milestone_ticket = Ticket::new_inner()
    ///     .with_id(ticket_id)
    ///     .with_context(ticket_context)
    ///     .with_source(ticket_source)
    ///     .with_is_resolved(ticket_is_resolved);
    ///
    /// let milestone_tickets = Collection::new_inner()
    ///     .with_total_items(1u64)
    ///     .with_items([milestone_ticket]);
    ///
    /// let milestone_release = Release::new_inner()
    ///     .with_id(release_id);
    ///
    /// let milestone_releases = Collection::new_inner()
    ///     .with_total_items(1u64)
    ///     .with_items([milestone_release]);
    ///
    /// let milestone = Milestone::new_inner()
    ///     .with_name(milestone_name)
    ///     .with_published(milestone_published.parse::<DateTime>().unwrap())
    ///     .with_is_resolved(milestone_is_resolved)
    ///     .with_milestone_releases(milestone_releases)
    ///     .with_milestone_tickets(milestone_tickets);
    ///
    /// let milestones = Collection::new_inner()
    ///     .with_total_items(1u64)
    ///     .with_items([milestone]);
    ///
    /// let release = Release::new()
    ///     .with_context_property(context_property)
    ///     .with_name(name)
    ///     .with_is_pre_release(is_pre_release)
    ///     .with_release_milestones(milestones);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&release).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Release>(json_str.as_str()).unwrap(),
    ///     release
    /// );
    /// # }
    /// ```
    Release: crate::ObjectType::Release {
        is_pre_release: Option<bool>,
        release_milestones: Option<Collection>,
    }
}

field_access! {
    Release {
        /// Whether this is a pre-release.
        is_pre_release: option { bool },
    }
}

field_access! {
    Release {
        /// [Collection] of [Milestone](crate::Milestone)s associated with this release.
        release_milestones: option_ref { Collection },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Milestone, Ticket, context};

    use activitystreams_vocabulary::{Content, DateTime, Iri, MimeType, Name};

    #[test]
    fn test_review() {
        let name = Name::try_from("Release v0.1.0").unwrap();
        let is_pre_release = false;

        let milestone_name = Name::try_from("milestone #1").unwrap();
        let milestone_published = "2026-04-20T13:37:42Z";
        let milestone_is_resolved = false;

        let release_id = Iri::try_from("https://example.dev/alice/myrepo/releases/1").unwrap();

        let ticket_id = Iri::try_from("https://example.dev/alice/myrepo/issues/42").unwrap();
        let ticket_context = Iri::try_from("https://example.dev/alice/myrepo").unwrap();

        let ticket_source_content = "Please fix. *Everything* is broken!";
        let ticket_source_type = MimeType::TextMarkdownCommonMark;

        let ticket_is_resolved = false;

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Release",
  "name": "{name}",
  "isPreRelease": {is_pre_release},
  "releaseMilestones": {{
    "type": "Collection",
    "totalItems": 1,
    "items": [
      {{
        "type": "Milestone",
        "name": "{milestone_name}",
        "published": "{milestone_published}",
        "isResolved": {milestone_is_resolved},
        "milestoneReleases": {{
          "type": "Collection",
          "totalItems": 1,
          "items": [
            {{
              "type": "Release",
              "id": "{release_id}"
            }}
          ]
        }},
        "milestoneTickets": {{
          "type": "Collection",
          "totalItems": 1,
          "items": [
            {{
              "type": "Ticket",
              "id": "{ticket_id}",
              "context": "{ticket_context}",
              "source": {{
                "content": "{ticket_source_content}",
                "mediaType": "{ticket_source_type}"
              }},
              "isResolved": {ticket_is_resolved}
            }}
          ]
        }}
      }}
    ]
  }}
}}"#
        );

        let context_property = context::forgefed_context();

        let ticket_source = Content::new()
            .with_content(ticket_source_content)
            .with_media_type(ticket_source_type);

        let milestone_ticket = Ticket::new_inner()
            .with_id(ticket_id)
            .with_context(ticket_context)
            .with_source(ticket_source)
            .with_is_resolved(ticket_is_resolved);

        let milestone_tickets = Collection::new_inner()
            .with_total_items(1u64)
            .with_items([milestone_ticket]);

        let milestone_release = Release::new_inner().with_id(release_id);

        let milestone_releases = Collection::new_inner()
            .with_total_items(1u64)
            .with_items([milestone_release]);

        let milestone = Milestone::new_inner()
            .with_name(milestone_name)
            .with_published(milestone_published.parse::<DateTime>().unwrap())
            .with_is_resolved(milestone_is_resolved)
            .with_milestone_releases(milestone_releases)
            .with_milestone_tickets(milestone_tickets);

        let milestones = Collection::new_inner()
            .with_total_items(1u64)
            .with_items([milestone]);

        let release = Release::new()
            .with_context_property(context_property)
            .with_name(name)
            .with_is_pre_release(is_pre_release)
            .with_release_milestones(milestones);

        assert_eq!(serde_json::to_string_pretty(&release).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Release>(json_str.as_str()).unwrap(),
            release
        );
    }
}
