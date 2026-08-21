use activitystreams_vocabulary::{Iri, Item, Link, create_activity, create_item, field_access};

use crate::{Error, Result};

create_activity! {
    /// Indicates that `target` is being given (by the `actor`) access to a resource specified by `context` under the role/permission specified by `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Grant, context};
    /// use activitystreams_vocabulary::Iri;
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/aviva/outbox/reBGo").unwrap();
    /// let actor = Iri::try_from("http://example.dev/aviva").unwrap();
    ///
    /// let object = Iri::try_from("write").unwrap();
    /// let context = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
    /// let target = Iri::try_from("https://example.dev/bob").unwrap();
    ///
    /// let to0 = Iri::try_from("https://example.dev/aviva/followers").unwrap();
    /// let to1 = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
    /// let to2 = Iri::try_from("https://example.dev/aviva/myproject/followers").unwrap();
    /// let to3 = Iri::try_from("https://example.dev/bob").unwrap();
    /// let to4 = Iri::try_from("https://example.dev/bob/followers").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Grant",
    ///   "id": "{id}",
    ///   "context": "{context}",
    ///   "to": [
    ///     "{to0}",
    ///     "{to1}",
    ///     "{to2}",
    ///     "{to3}",
    ///     "{to4}"
    ///   ],
    ///   "actor": "{actor}",
    ///   "object": "{object}",
    ///   "target": "{target}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let to = [to0, to1, to2, to3, to4];
    ///
    /// let grant = Grant::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_context(context)
    ///     .with_target(target)
    ///     .with_to(to);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&grant).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Grant>(json_str.as_str()).unwrap(),
    ///     grant
    /// );
    /// # }
    /// ```
    Grant: crate::ActivityType::Grant {
        #[serde(skip_serializing_if = "Option::is_none")]
        fulfills: Option<Item>,
    }
}

field_access! {
    Grant {
        fulfills: option_ref { Item },
    }
}

create_item! {
    /// Represents the variants for property fields referencing a [Grant].
    GrantItem
        boxed
        default: Self::Grant(Box::default()),
    {
        Grant(Grant),
        Iri(Iri),
        Link(Link),
    }
}

impl GrantItem {
    /// Gets the [GrantItem] ID.
    pub fn id(&self) -> Result<&Iri> {
        match self {
            Self::Grant(grant) => grant.id().ok_or(Error::activity("grant: missing ID")),
            Self::Iri(iri) => Ok(iri.as_ref()),
            Self::Link(link) => Ok(link.href()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn test_grant() {
        let id = Iri::try_from("https://example.dev/aviva/outbox/reBGo").unwrap();
        let actor = Iri::try_from("http://example.dev/aviva").unwrap();

        let object = Iri::try_from("write").unwrap();
        let context = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
        let target = Iri::try_from("https://example.dev/bob").unwrap();

        let to0 = Iri::try_from("https://example.dev/aviva/followers").unwrap();
        let to1 = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
        let to2 = Iri::try_from("https://example.dev/aviva/myproject/followers").unwrap();
        let to3 = Iri::try_from("https://example.dev/bob").unwrap();
        let to4 = Iri::try_from("https://example.dev/bob/followers").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Grant",
  "id": "{id}",
  "context": "{context}",
  "to": [
    "{to0}",
    "{to1}",
    "{to2}",
    "{to3}",
    "{to4}"
  ],
  "actor": "{actor}",
  "object": "{object}",
  "target": "{target}"
}}"#
        );

        let context_property = context::forgefed_context();

        let to = [to0, to1, to2, to3, to4];

        let grant = Grant::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_actor(actor)
            .with_object(object)
            .with_context(context)
            .with_target(target)
            .with_to(to);

        assert_eq!(serde_json::to_string_pretty(&grant).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Grant>(json_str.as_str()).unwrap(),
            grant
        );
    }
}
