use crate::actor::Actor;
use crate::TypedType;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Serialize, Deserialize)]
pub struct Organization(Actor);

#[typetag::serde]
impl TypedType for Organization {}

impl Deref for Organization {
    type Target = Actor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
