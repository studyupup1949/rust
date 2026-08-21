use crate::activity::{ActivityType, IntransitiveActivityType};
use crate::link::Link;
use crate::object::{LinkObjectOrList, ObjectType, TypedObjectType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct AnswerOptions(LinkObjectOrList);

#[derive(Serialize, Deserialize)]
pub enum Answer {
    OneOf(AnswerOptions),
    AnyOf(AnswerOptions),
}

#[derive(Serialize, Deserialize)]
pub enum Closed {
    Boolean(bool),
    DateTime,
    Object,
    Link(Link),
}

#[derive(Serialize, Deserialize)]
pub struct Question {
    closed: Closed,
}

impl ObjectType for Question {}
#[typetag::serde]
impl TypedObjectType for Question {}

impl ActivityType for Question {}
impl IntransitiveActivityType for Question {
    fn object(&self) -> ! {
        unimplemented!()
    }
}
