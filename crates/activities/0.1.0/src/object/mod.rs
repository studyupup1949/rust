use crate::link::{Link, Mime};
pub use article::Article;
pub use document::Document;
pub use document::DocumentType;
use serde::{Deserialize, Serialize};

use std::time::Duration;
pub use video::Video;

mod article;
// mod audio;
mod document;
// mod event;
// mod image;
// mod note;
// mod page;
// mod place;
// mod profile;
// mod relationship;
// mod tombstone;
mod video;

#[typetag::serde(tag = "type")]
pub trait TypedObjectType: ObjectType {}

pub trait ObjectType {
    fn attachment(&self) {}
    fn attributed_to(&self) {}
    fn audience(&self) {}
    fn content(&self) -> Option<&String> {
        None
    }
    fn context(&self) {}
    fn name(&self) -> Option<&String> {
        None
    }
    fn end_time(&self) {}
    fn generator(&self) {}
    fn icon(&self) {}
    fn image(&self) {}
    fn in_reply_to(&self) {}
    fn location(&self) {}
    fn preview(&self) {}
    fn published(&self) {}
    fn summary(&self) {}
    fn tag(&self) {}
    fn updated(&self) {}
    fn url(&self) -> Option<&Link> {
        None
    }
    fn to(&self) {}
    fn bto(&self) {}
    fn cc(&self) {}
    fn bcc(&self) {}
    fn media_type(&self) -> Option<Mime> {
        None
    }
    fn duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Deserialize, Serialize)]
pub enum LinkOrObject {
    Link(Link),
    Object(Box<dyn TypedObjectType>),
}

#[derive(Deserialize, Serialize)]
pub enum LinkOrList {
    Link(Link),
    List(Vec<Box<dyn TypedObjectType>>),
}

#[derive(Deserialize, Serialize)]
pub enum LinkObjectOrList {
    Link(Link),
    Object(Box<dyn TypedObjectType>),
    List(Vec<Box<dyn TypedObjectType>>),
}
