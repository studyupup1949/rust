use serde::{Deserialize, Serialize};

use crate::{
    DateTime, Error, Items, Link, Object, Result, create_intransitive_activity, field_access,
    impl_default, impl_display,
};

create_intransitive_activity! {
    /// Represents a question being asked.
    ///
    /// Question objects are an extension of [IntransitiveActivity](crate::IntransitiveActivity).
    ///
    /// That is, the [Question] object is an [Activity](crate::Activity), but the direct object is the question itself and therefore it would not contain an `object` property.
    ///
    /// Either of the `anyOf` and `oneOf` properties **MAY** be used to express possible answers,
    /// but a [Question] object **MUST NOT** have both properties.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Closed, Name, Note, Question};
    ///
    /// # fn main() {
    /// let name = Name::try_from("What is the answer?").unwrap();
    /// let a_name = Name::try_from("Option A").unwrap();
    /// let b_name = Name::try_from("Option B").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Question",
    ///   "name": "{name}",
    ///   "oneOf": [
    ///     {{
    ///       "type": "Note",
    ///       "name": "{a_name}"
    ///     }},
    ///     {{
    ///       "type": "Note",
    ///       "name": "{b_name}"
    ///     }}
    ///   ]
    /// }}"#);
    ///
    /// let option_a = Note::new_inner().with_name(a_name);
    /// let option_b = Note::new_inner().with_name(b_name);
    /// let question = Question::new()
    ///     .with_name(name)
    ///     .with_one_of([option_a, option_b]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&question).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Question>(json_str.as_str()).unwrap(),
    ///     question
    /// );
    ///
    /// let name = Name::try_from("What is the answer?").unwrap();
    /// let closed = Closed::date_time_str("2016-05-10T00:00:00Z").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Question",
    ///   "name": "What is the answer?",
    ///   "closed": {closed}
    /// }}"#);
    ///
    /// let question = Question::new()
    ///     .with_name(name)
    ///     .with_closed(closed);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&question).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Question>(json_str.as_str()).unwrap(),
    ///     question
    /// );
    /// # }
    /// ```
    Question {
        #[serde(skip_serializing_if = "Option::is_none")]
        one_of: Option<Items>,
        #[serde(skip_serializing_if = "Option::is_none")]
        any_of: Option<Items>,
        #[serde(skip_serializing_if = "Option::is_none")]
        closed: Option<Closed>,
    }
}

impl Question {
    /// Gets the [Question] `oneOf` field.
    ///
    /// Identifies an exclusive option for a Question.
    ///
    /// Use of `oneOf` implies that the [Question] can have only a single answer.
    ///
    /// To indicate that a [Question] can have multiple answers, use `anyOf`.
    pub fn one_of(&self) -> Option<&Items> {
        self.one_of.as_ref()
    }

    /// Sets the [Question] `oneOf` field.
    ///
    /// Identifies an exclusive option for a Question.
    ///
    /// Use of `oneOf` implies that the [Question] can have only a single answer.
    ///
    /// To indicate that a [Question] can have multiple answers, use `anyOf`.
    pub fn set_one_of<I: Into<Items>>(&mut self, val: I) {
        self.one_of = Some(val.into());
        self.any_of = None;
    }

    /// Builder function that sets the [Question] `oneOf` field.
    ///
    /// Identifies an exclusive option for a Question.
    ///
    /// Use of `oneOf` implies that the [Question] can have only a single answer.
    ///
    /// To indicate that a [Question] can have multiple answers, use `anyOf`.
    pub fn with_one_of<I: Into<Items>>(self, val: I) -> Self {
        Self {
            one_of: Some(val.into()),
            any_of: None,
            ..self
        }
    }

    /// Gets the [Question] `anyOf` field.
    ///
    /// Identifies an inclusive option for a [Question].
    ///
    /// Use of `anyOf` implies that the [Question] can have multiple answers.
    ///
    /// To indicate that a [Question] can have only one answer, use `oneOf`.
    pub fn any_of(&self) -> Option<&Items> {
        self.any_of.as_ref()
    }

    /// Sets the [Question] `anyOf` field.
    ///
    /// Identifies an inclusive option for a [Question].
    ///
    /// Use of `anyOf` implies that the [Question] can have multiple answers.
    ///
    /// To indicate that a [Question] can have only one answer, use `oneOf`.
    pub fn set_any_of<I: Into<Items>>(&mut self, val: I) {
        self.any_of = Some(val.into());
        self.one_of = None;
    }

    /// Builder function that sets the [Question] `anyOf` field.
    ///
    /// Identifies an inclusive option for a [Question].
    ///
    /// Use of `anyOf` implies that the [Question] can have multiple answers.
    ///
    /// To indicate that a [Question] can have only one answer, use `oneOf`.
    pub fn with_any_of<I: Into<Items>>(self, val: I) -> Self {
        Self {
            any_of: Some(val.into()),
            one_of: None,
            ..self
        }
    }
}

field_access! {
    Question {
        /// Indicates that a question has been closed, and answers are no longer accepted.
        closed: option_ref { Closed },
    }
}

/// Indicates that a question has been closed, and answers are no longer accepted.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Closed {
    Object(Box<Object>),
    Link(Box<Link>),
    DateTime(Box<DateTime>),
    Bool(bool),
}

impl Closed {
    /// Creates a new [Closed].
    pub const fn new() -> Self {
        Self::Bool(false)
    }

    /// Creates a new [Closed] [Object](Self::Object) variant.
    pub fn object<I: Into<Object>>(val: I) -> Self {
        Self::Object(Box::new(val.into()))
    }

    /// Gets whether [Closed] is a [Object](Self::Object) variant.
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Attempts to get [Closed] as reference to a [Link](Self::Link) variant.
    pub fn as_object(&self) -> Result<&Object> {
        match self {
            Self::Object(object) => Ok(object.as_ref()),
            _ => Err(Error::question("invalid closed variant")),
        }
    }

    /// Creates a new [Closed] [Link](Self::Link) variant.
    pub fn link<I: Into<Link>>(val: I) -> Self {
        Self::Link(Box::new(val.into()))
    }

    /// Gets whether [Closed] is a [Link](Self::Link) variant.
    pub const fn is_link(&self) -> bool {
        matches!(self, Self::Link(_))
    }

    /// Attempts to get [Closed] as reference to a [Link](Self::Link) variant.
    pub fn as_link(&self) -> Result<&Link> {
        match self {
            Self::Link(link) => Ok(link.as_ref()),
            _ => Err(Error::question("invalid closed variant")),
        }
    }

    /// Creates a new [Closed] [DateTime](Self::DateTime) variant.
    pub fn date_time<E, I>(val: I) -> Result<Self>
    where
        E: core::error::Error,
        I: TryInto<DateTime, Error = E>,
    {
        val.try_into()
            .map_err(|err| Error::question(format!("invalid closed date-time: {err}")))
            .map(Box::new)
            .map(Self::DateTime)
    }

    /// Creates a new [Closed] [DateTime](Self::DateTime) variant from a string.
    pub fn date_time_str(val: &str) -> Result<Self> {
        val.parse::<DateTime>()
            .map_err(|err| Error::question(format!("invalid closed date-time: {err}")))
            .map(Box::new)
            .map(Self::DateTime)
    }

    /// Gets whether [Closed] is a [DateTime](Self::DateTime) variant.
    pub const fn is_date_time(&self) -> bool {
        matches!(self, Self::DateTime(_))
    }

    /// Attempts to get [Closed] as reference to a [DateTime](Self::DateTime) variant.
    pub fn as_date_time(&self) -> Result<&DateTime> {
        match self {
            Self::DateTime(date) => Ok(date.as_ref()),
            _ => Err(Error::question("invalid closed variant")),
        }
    }

    /// Creates a new [Closed] [Bool](Self::Bool) variant.
    pub const fn bool(val: bool) -> Self {
        Self::Bool(val)
    }

    /// Gets whether [Closed] is a [Bool](Self::Bool) variant.
    pub const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Attempts to get [Closed] as reference to a [Bool](Self::Bool) variant.
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Self::Bool(b) => Ok(*b),
            _ => Err(Error::question("invalid closed variant")),
        }
    }
}

impl_default!(Closed);
impl_display!(Closed, json);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Note};

    #[test]
    fn test_activity() {
        let name = Name::try_from("What is the answer?").unwrap();
        let a_name = Name::try_from("Option A").unwrap();
        let b_name = Name::try_from("Option B").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Question",
  "name": "{name}",
  "oneOf": [
    {{
      "type": "Note",
      "name": "{a_name}"
    }},
    {{
      "type": "Note",
      "name": "{b_name}"
    }}
  ]
}}"#
        );

        let option_a = Note::new_inner().with_name(a_name);
        let option_b = Note::new_inner().with_name(b_name);
        let question = Question::new()
            .with_name(name)
            .with_one_of([option_a, option_b]);

        assert_eq!(serde_json::to_string_pretty(&question).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Question>(json_str.as_str()).unwrap(),
            question
        );

        let name = Name::try_from("What is the answer?").unwrap();
        let closed = Closed::date_time_str("2016-05-10T00:00:00Z").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Question",
  "name": "What is the answer?",
  "closed": {closed}
}}"#
        );

        let question = Question::new().with_name(name).with_closed(closed);

        assert_eq!(serde_json::to_string_pretty(&question).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Question>(json_str.as_str()).unwrap(),
            question
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Question>(json_str).is_err());
    }
}
