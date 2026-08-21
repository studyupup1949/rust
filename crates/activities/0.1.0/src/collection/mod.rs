use crate::object::ObjectType;

mod ordered;
mod orderedpage;
mod page;

pub enum ObjectOrLink {}

pub trait CollectionType: ObjectType {
    fn total_items(&self) -> usize;
    fn first(&self) -> ObjectOrLink;
    fn last(&self) -> ObjectOrLink;
    fn current(&self) -> ObjectOrLink;
    fn items(&self) -> dyn Iterator<Item = ObjectOrLink>;
}
