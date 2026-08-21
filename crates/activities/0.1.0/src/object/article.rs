use crate::object::{ObjectType, TypedObjectType};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Article {
    name: String,
    content: String,
}

#[typetag::serde]
impl TypedObjectType for Article {}
impl ObjectType for Article {
    fn content(&self) -> Option<&String> {
        Some(&self.content)
    }

    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }
}
