#![doc = include_str!("../README.md")]

extern crate alloc;

pub use heck;
pub use paste::paste;
pub use serde;
pub use serde_json;

pub use alloc::collections::BTreeMap as Map;

pub mod activity;
pub mod actor;
mod content;
mod context;
mod duration;
mod error;
mod generic;
mod iri;
mod item;
mod langtag;
pub mod link;
mod list;
mod macros;
mod mime;
mod name;
pub mod object;
pub mod security;
mod time;
pub mod vocabulary;

pub use activity::*;
pub use actor::*;
pub use content::{Content, ContentItem};
pub use context::Context;
pub use duration::Duration;
pub use error::{Error, Result};
pub use generic::GenericType;
pub use iri::*;
pub use item::{Item, Items, OrderedItems};
pub use langtag::{LanguageMap, LanguageTag};
pub use link::*;
pub use list::OrderedList;
pub use mime::MimeType;
pub use name::{Name, NameMap};
pub use object::*;
pub use security::*;
pub use time::DateTime;
pub use vocabulary::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::{ActivityVocabulary, Error};

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize)]
    pub struct TestType<T: ActivityVocabulary>(T);
    impl<T: ActivityVocabulary> TestType<T> {
        pub fn as_type(&self) -> Result<T::Type, Error> {
            self.0.as_type()
        }
    }

    impl<'de, T: ActivityVocabulary> serde::de::Deserialize<'de> for TestType<T> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            T::deserialize(deserializer).map(TestType)
        }
    }
}
