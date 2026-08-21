use crate::actor::Actor;
use crate::TypedType;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
pub struct Group(Actor);

#[typetag::serde]
impl TypedType for Group {}

impl Deref for Group {
    type Target = Actor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
