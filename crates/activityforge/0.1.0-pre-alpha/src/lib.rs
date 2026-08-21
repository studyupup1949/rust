#![doc = include_str!("../README.md")]

pub use paste::paste;

pub mod context;

pub mod activity;
pub mod actor;
pub mod db;
mod error;
mod hash;
pub mod object;
pub mod roadmap;
pub mod vocabulary;

pub(crate) mod util;

pub use activity::*;
pub use actor::*;
pub use error::{Error, Result};
pub use hash::{Hash, Sha1Hash, Sha256Hash};
pub use object::*;
pub use vocabulary::*;

#[cfg(test)]
pub(crate) mod tests {
    use activitystreams_vocabulary::{ActivityVocabulary, Error};

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
