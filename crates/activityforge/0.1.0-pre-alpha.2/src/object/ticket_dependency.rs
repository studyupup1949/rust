use activitystreams_vocabulary::{Item, create_object, field_access};

use crate::DependsOn;

create_object! {
    /// Represents a relationship between 2 [Ticket](crate::Ticket)s, in which the resolution of one ticket requires the other ticket to be resolved too.
    ///
    /// It **MUST** specify the `subject`, `object` and `relationship` properties.
    ///
    /// The `relationship` property **MUST** be [dependsOn](DependsOn).
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{DependsOn, TicketDependency, context};
    /// use activitystreams_vocabulary::{DateTime, Iri};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/ticket-deps/2342593").unwrap();
    /// let attributed_to = Iri::try_from("https://example.dev/alice").unwrap();
    /// let summary = "Alice's ticket depends on Bob's ticket";
    /// let published = "2019-07-11T12:34:56Z";
    /// let subject = Iri::try_from("https://example.dev/alice/myproj/issues/42").unwrap();
    /// let relationship = DependsOn::new();
    /// let object = Iri::try_from("https://dev.community/bob/coolproj/issues/85").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "TicketDependency",
    ///   "id": "{id}",
    ///   "attributedTo": "{attributed_to}",
    ///   "summary": "{summary}",
    ///   "published": "{published}",
    ///   "subject": "{subject}",
    ///   "relationship": "{relationship}",
    ///   "object": "{object}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let ticket_dep = TicketDependency::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_attributed_to(attributed_to)
    ///     .with_published(published.parse::<DateTime>().unwrap())
    ///     .with_summary(summary)
    ///     .with_subject(subject)
    ///     .with_relationship(relationship)
    ///     .with_object(object);
    ///
    /// assert_eq!(
    ///     serde_json::to_string_pretty(&ticket_dep).unwrap(),
    ///     json_str
    /// );
    /// assert_eq!(
    ///     serde_json::from_str::<TicketDependency>(json_str.as_str()).unwrap(),
    ///     ticket_dep
    /// );
    /// # }
    /// ```
    TicketDependency: crate::ObjectType::TicketDependency {
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<Box<Item>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationship: Option<Box<DependsOn>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<Box<Item>>,
    }
}

field_access! {
    TicketDependency<Vocab> {
        /// On a [TicketDependency] object, the `subject` property identifies one of the connected individuals.
        ///
        /// For instance, for a [TicketDependency] object describing "John is related to Sally", `subject` would refer to John.
        subject: option_box_deref { Item },
        /// On a [TicketDependency] object, the `relationship` property identifies the kind of relationship that exists between `subject` and `object`.
        relationship: option_box_deref { DependsOn },
        /// When used within a [TicketDependency], `object` describes the entity to which the `subject` is related.
        object: option_box_deref { Item },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;
    use activitystreams_vocabulary::{DateTime, Iri};

    #[test]
    fn test_ticket_dependency() {
        let id = Iri::try_from("https://example.dev/ticket-deps/2342593").unwrap();
        let attributed_to = Iri::try_from("https://example.dev/alice").unwrap();
        let summary = "Alice's ticket depends on Bob's ticket";
        let published = "2019-07-11T12:34:56Z";
        let subject = Iri::try_from("https://example.dev/alice/myproj/issues/42").unwrap();
        let relationship = DependsOn::new();
        let object = Iri::try_from("https://dev.community/bob/coolproj/issues/85").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "TicketDependency",
  "id": "{id}",
  "attributedTo": "{attributed_to}",
  "summary": "{summary}",
  "published": "{published}",
  "subject": "{subject}",
  "relationship": "{relationship}",
  "object": "{object}"
}}"#
        );

        let context = context::forgefed_context();

        let ticket_dep = TicketDependency::new()
            .with_context_property(context)
            .with_id(id)
            .with_attributed_to(attributed_to)
            .with_published(published.parse::<DateTime>().unwrap())
            .with_summary(summary)
            .with_subject(subject)
            .with_relationship(relationship)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&ticket_dep).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<TicketDependency>(json_str.as_str()).unwrap(),
            ticket_dep
        );
    }
}
