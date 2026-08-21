#![doc = include_str!("../README.md")]

extern crate alloc;

pub use heck;
pub use paste::paste;
pub use serde;
pub use serde_json;

pub use alloc::collections::BTreeMap as Map;

mod activity;
mod application;
mod article;
mod audio;
mod collection;
mod content;
mod context;
mod data_integrity_proof;
mod document;
mod duration;
mod endpoints;
mod error;
mod event;
mod generic;
mod group;
mod image;
mod iri;
mod item;
mod key;
mod langtag;
mod link;
mod list;
mod macros;
mod mention;
mod mime;
mod multikey;
mod name;
mod note;
mod object;
mod organization;
mod page;
mod person;
mod place;
mod profile;
mod relationship;
mod service;
mod time;
mod tombstone;
mod video;
mod vocabulary;

pub use activity::{
    Accept, Activity, Add, Announce, Arrive, Block, Closed, Create, Delete, Dislike, Flag, Follow,
    Ignore, IntransitiveActivity, Invite, Join, Leave, Like, Listen, Move, Offer, Question, Read,
    Reject, Remove, TentativeAccept, TentativeReject, Travel, Undo, Update, View,
};
pub use application::Application;
pub use article::Article;
pub use audio::Audio;
pub use collection::{
    Collection, CollectionItem, CollectionPage, CollectionPageItem, OrderedCollection,
    OrderedCollectionPage, OrderedCollectionPageItem,
};
pub use content::{Content, ContentItem};
pub use context::Context;
pub use data_integrity_proof::{Cryptosuite, DataIntegrityProof, DataIntegrityProofBytes};
pub use document::Document;
pub use duration::Duration;
pub use endpoints::Endpoints;
pub use error::{Error, Result};
pub use event::Event;
pub use generic::GenericType;
pub use group::Group;
pub use image::{Image, ImageItem};
pub use iri::{Iri, IriItem};
pub use item::{Item, Items, OrderedItems};
pub use key::{Key, PrivateKeyPem, PublicKeyPem};
pub use langtag::{LanguageMap, LanguageTag};
pub use link::{Link, Links};
pub use list::OrderedList;
pub use mention::Mention;
pub use mime::MimeType;
pub use multikey::{
    MultibaseData, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Multikeys,
};
pub use name::{Name, NameMap};
pub use note::Note;
pub use object::{Object, Objects};
pub use organization::Organization;
pub use page::Page;
pub use person::Person;
pub use place::{Accuracy, Float, Place, Radius, Unit, Units};
pub use profile::Profile;
pub use relationship::Relationship;
pub use service::Service;
pub use time::DateTime;
pub use tombstone::{Deleted, Tombstone};
pub use video::Video;
pub use vocabulary::{
    ActivityType, ActivityVocabulary, ActorType, CoreType, LinkType, ObjectType, SecurityType,
    VocabularyType, VocabularyTypes,
};

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
