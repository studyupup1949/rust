pub mod activity;
mod actor;
pub mod collection;
pub mod link;
pub mod object;
mod util;

#[typetag::serde(tag = "type")]
pub trait TypedType {}
