use serde::{Deserialize, Serialize};

use crate::link::Link;
use crate::object::ObjectType;
use crate::TypedType;

pub trait DocumentType: ObjectType {}

#[derive(Serialize, Deserialize)]
pub struct Document {
    url: Link,
    name: String,
}

#[typetag::serde]
impl TypedType for Document {}

impl ObjectType for Document {
    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }

    fn url(&self) -> Option<&Link> {
        Some(&self.url)
    }
}

impl DocumentType for Document {}
