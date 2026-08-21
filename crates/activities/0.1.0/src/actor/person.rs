use crate::actor::Actor;
use crate::TypedType;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
pub struct Person(Actor);

#[typetag::serde]
impl TypedType for Person {}

impl Deref for Person {
    type Target = Actor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
