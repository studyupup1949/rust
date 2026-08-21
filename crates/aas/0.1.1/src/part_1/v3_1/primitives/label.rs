use crate::utilities::validate_text;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct Label(String);

#[derive(Error, Debug)]
pub enum LabelError {
    #[error("The label type needs at lest 1 character")]
    TooShort,

    #[error("The label type can hold at most 64 characters")]
    TooLong,

    #[error("Invalid character found")]
    InvalidCharacter,
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Label {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Label {
    type Error = LabelError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(LabelError::TooShort);
        }

        if value.chars().count() > 64 {
            return Err(LabelError::TooLong);
        }

        if !validate_text(&value) {
            return Err(LabelError::InvalidCharacter);
        }

        Ok(Label(value))
    }
}

impl TryFrom<&str> for Label {
    type Error = LabelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LabelVisitor;

        impl<'de> Visitor<'de> for LabelVisitor {
            type Value = Label;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a valid label string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Label, E>
            where
                E: de::Error,
            {
                Label::try_from(value)
                    .map_err(|err| de::Error::custom(format!("Invalid Label: {}", err)))
            }

            fn visit_string<E>(self, value: String) -> Result<Label, E>
            where
                E: de::Error,
            {
                Label::try_from(value)
                    .map_err(|err| de::Error::custom(format!("Invalid Label: {}", err)))
            }
        }

        deserializer.deserialize_string(LabelVisitor)
    }
}
