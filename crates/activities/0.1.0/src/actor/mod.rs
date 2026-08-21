use crate::object::ObjectType;
use serde::{Deserialize, Serialize};

mod application;
mod group;
mod organization;
mod person;
mod service;

pub trait ActorType: ObjectType {}

#[typetag::serde(tag = "type")]
pub trait TypedActorType: ActorType {}

#[derive(Serialize, Deserialize)]
pub struct Actor {
    // todo
}
