use activitystreams_vocabulary::{Iri, Link, create_activity, impl_default, impl_display};
use serde::{Deserialize, Serialize};

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
    Grant: crate::ActivityType::Grant {}
}

/// Represents the variants for property fields referencing a [Grant].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GrantItem {
    Grant(Box<Grant>),
    Iri(Box<Iri>),
    Link(Box<Link>),
}

impl GrantItem {
    /// Creates a new [GrantItem].
    pub fn new() -> Self {
        Self::Grant(Box::default())
    }

    /// Creates a new [Grant](Self::Grant) variant.
    pub fn grant<I: Into<Grant>>(val: I) -> Self {
        Self::Grant(Box::new(val.into()))
    }

    /// Gets whether the [GrantItem] contains a [Grant](Self::Grant) variant.
    pub const fn is_grant(&self) -> bool {
        matches!(self, Self::Grant(_))
    }

    /// Gets a reference to the [GrantItem] containing a [Grant](Self::Grant) variant.
    pub fn as_grant(&self) -> Result<&Grant> {
        match self {
            Self::Grant(grant) => Ok(grant),
            _ => Err(Error::activity("invalid grant item variant")),
        }
    }

    /// Converts the [GrantItem] into a [Grant](Self::Grant) variant.
    pub fn to_grant(self) -> Result<Grant> {
        match self {
            Self::Grant(grant) => Ok(*grant),
            _ => Err(Error::activity("invalid grant item variant")),
        }
    }

    /// Creates a new [Iri](Self::Iri) variant.
    pub fn iri<I: Into<Iri>>(val: I) -> Self {
        Self::Iri(Box::new(val.into()))
    }

    /// Gets whether the [GrantItem] contains a [Iri](Self::Iri) variant.
    pub const fn is_iri(&self) -> bool {
        matches!(self, Self::Iri(_))
    }

    /// Gets a reference to the [GrantItem] containing a [Iri](Self::Iri) variant.
    pub fn as_iri(&self) -> Result<&Iri> {
        match self {
            Self::Iri(iri) => Ok(iri),
            _ => Err(Error::activity("invalid iri item variant")),
        }
    }

    /// Converts the [GrantItem] into a [Iri](Self::Iri) variant.
    pub fn to_iri(self) -> Result<Iri> {
        match self {
            Self::Iri(iri) => Ok(*iri),
            _ => Err(Error::activity("invalid iri item variant")),
        }
    }

    /// Creates a new [Link](Self::Link) variant.
    pub fn link<I: Into<Link>>(val: I) -> Self {
        Self::Link(Box::new(val.into()))
    }

    /// Gets whether the [GrantItem] contains a [Link](Self::Link) variant.
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Gets a reference to the [GrantItem] containing a [Link](Self::Link) variant.
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(link) => Ok(link),
            _ => Err(Error::activity("invalid link item variant")),
        }
    }

    /// Converts the [GrantItem] into a [Link](Self::Link) variant.
    pub fn to_link(self) -> Result<Link> {
        match self {
            Self::Link(link) => Ok(*link),
            _ => Err(Error::activity("invalid link item variant")),
        }
    }
}

impl_default!(GrantItem);
impl_display!(GrantItem, json);

impl<I: Into<Grant>> From<I> for GrantItem {
    fn from(val: I) -> Self {
        Self::grant(val)
    }
}

impl From<Iri> for GrantItem {
    fn from(val: Iri) -> Self {
        Self::iri(val)
    }
}

impl From<Link> for GrantItem {
    fn from(val: Link) -> Self {
        Self::link(val)
    }
}

impl<'a> TryFrom<&'a GrantItem> for &'a Grant {
    type Error = Error;

    fn try_from(val: &'a GrantItem) -> Result<Self> {
        val.as_grant()
    }
}

impl TryFrom<GrantItem> for Grant {
    type Error = Error;

    fn try_from(val: GrantItem) -> Result<Self> {
        val.to_grant()
    }
}

impl<'a> TryFrom<&'a GrantItem> for &'a Iri {
    type Error = Error;

    fn try_from(val: &'a GrantItem) -> Result<Self> {
        val.as_iri()
    }
}

impl TryFrom<GrantItem> for Iri {
    type Error = Error;

    fn try_from(val: GrantItem) -> Result<Self> {
        val.to_iri()
    }
}

impl<'a> TryFrom<&'a GrantItem> for &'a Link {
    type Error = Error;

    fn try_from(val: &'a GrantItem) -> Result<Self> {
        val.as_link()
    }
}

impl TryFrom<GrantItem> for Link {
    type Error = Error;

    fn try_from(val: GrantItem) -> Result<Self> {
        val.to_link()
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
