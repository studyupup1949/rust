use crate::link::Link;
use crate::object::{DocumentType, ObjectType, TypedObjectType};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Video {
    url: Link,
    name: String,
}

impl Video {
    pub fn new(url: Link, name: String) -> Self {
        Self { url, name }
    }
}
#[typetag::serde]
impl TypedObjectType for Video {}

impl ObjectType for Video {
    fn name(&self) -> Option<&String> {
        Some(&self.name)
    }
    fn url(&self) -> Option<&Link> {
        Some(&self.url)
    }
}

impl DocumentType for Video {}
