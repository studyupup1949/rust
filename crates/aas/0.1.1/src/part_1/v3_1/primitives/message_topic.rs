use crate::utilities::validate_text;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use thiserror::Error;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize)]
pub struct MessageTopic(String);

#[derive(Error, Debug)]
pub enum MessageTopicError {
    #[error("The label type needs at lest 1 character")]
    TooShort,

    #[error("The label type can hold at most 255 characters")]
    TooLong,

    #[error("Invalid character found")]
    InvalidCharacter,
}

impl Display for MessageTopic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl MessageTopic {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for MessageTopic {
    type Error = MessageTopicError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(MessageTopicError::TooShort);
        }

        if value.chars().count() > 64 {
            return Err(MessageTopicError::TooLong);
        }

        if !validate_text(&value) {
            return Err(MessageTopicError::InvalidCharacter);
        }

        Ok(MessageTopic(value))
    }
}

impl TryFrom<&str> for MessageTopic {
    type Error = MessageTopicError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for MessageTopic {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for MessageTopic {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for MessageTopic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MessageTopicVisitor;

        impl<'de> Visitor<'de> for MessageTopicVisitor {
            type Value = MessageTopic;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a valid message topic string")
            }

            fn visit_str<E>(self, value: &str) -> Result<MessageTopic, E>
            where
                E: de::Error,
            {
                MessageTopic::try_from(value)
                    .map_err(|err| de::Error::custom(format!("Invalid MessageTopic: {}", err)))
            }

            fn visit_string<E>(self, value: String) -> Result<MessageTopic, E>
            where
                E: de::Error,
            {
                MessageTopic::try_from(value)
                    .map_err(|err| de::Error::custom(format!("Invalid MessageTopic: {}", err)))
            }
        }

        deserializer.deserialize_string(MessageTopicVisitor)
    }
}
