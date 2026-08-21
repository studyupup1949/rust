use crate::actor::{Actor, ActorType, TypedActorType};

use crate::object::ObjectType;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
pub struct Application(Actor);

impl ObjectType for Application {}
impl ActorType for Application {}

#[typetag::serde]
impl TypedActorType for Application {}

impl Deref for Application {
    type Target = Actor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
