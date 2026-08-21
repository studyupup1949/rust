use crate::object::ObjectType;

mod add;
mod intransitive;

pub trait ActivityType: ObjectType {
    // Optional for questions (maybe more)
    fn actor(&self) -> Option<&dyn ObjectType> {
        None
    }
    fn object(&self) -> Option<&dyn ObjectType> {
        None
    }
    fn target(&self) -> Option<&dyn ObjectType> {
        None
    }
    fn result(&self) -> Option<&dyn ObjectType> {
        None
    }
    fn origin(&self) -> Option<&dyn ObjectType> {
        None
    }

    fn instrument(&self) -> Option<&dyn ObjectType> {
        None
    }
}

pub trait IntransitiveActivityType: ActivityType {
    fn object(&self) -> !;
}
