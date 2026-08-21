use crate::object::{ObjectType, TypedObjectType};

use crate::actor::TypedActorType;
use typetag::serde::Deserialize;
use typetag::serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Add {
    pub actor: Box<dyn TypedActorType>,
    pub object: Box<dyn TypedObjectType>,
    pub target: Option<Box<dyn TypedObjectType>>,
    pub origin: Option<Box<dyn TypedObjectType>>,
}

#[typetag::serde]
impl TypedObjectType for Add {}

impl ObjectType for Add {}
