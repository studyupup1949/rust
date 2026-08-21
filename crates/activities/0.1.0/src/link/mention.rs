use crate::link::{LinkType, Url};
use crate::TypedType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mention {
    href: Url,
    name: Option<String>,
}

#[typetag::serde]
impl TypedType for Mention {}

impl LinkType for Mention {
    fn href(&self) -> &Url {
        &self.href
    }
    fn name(&self) -> Option<&String> {
        match &self.name {
            Some(val) => Some(val),
            _ => None,
        }
    }
}
