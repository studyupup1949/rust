//! actix-mock-helper is a set of helpers for creating mock actors using the `Mocker` type.
//! That type of actor can be  used to simulate any type of mock you could want, but it's very verbose to interact with.
//! actix-mock helper is especially useful in the case that you have multiple messages in a sequence that you want to mock

use actix::{Actor, Addr, actors::mocker::Mocker};
use std::any::Any;

/// A mock for a sequence of messages sent to the actor
/// Example:
/// ```
/// # use actix::prelude::*;
/// # use actix_mock_helper::MockActorSequence;
/// struct FakeActor;
/// impl Actor for FakeActor {
///     type Context = actix::Context<Self>;
/// }
/// struct Msg1;
/// struct Msg2;
/// impl Message for Msg1 {
///   type Result = i32;
/// }
/// impl Message for Msg2 {
///   type Result = bool;
/// }
///#[actix_rt::main]
/// async fn main() {
///   let mock_actor = MockActorSequence::new()
///     .msg(|_m: &Msg1| 5) // you can access the message sent.
///     .msg::<Msg2, _>(|_m| true) // alternate syntax to specify the message type
///     .build::<FakeActor>();
///   assert_eq!(mock_actor.send(Msg1).await.unwrap(), 5);
///   assert_eq!(mock_actor.send(Msg2).await.unwrap(), true);
/// }
/// ```
pub struct MockActorSequence {
    callbacks: Vec<Box<dyn FnMut(Box<dyn Any>) -> Box<dyn Any>>>,
    current: usize
}

impl MockActorSequence {
    pub fn new() -> Self {
        Self { callbacks: Vec::new(), current: 0 }
    }

    /// Add another message to be expected, and return the result of the callback.
    /// The type of the message is checked at runtime against the expectation,
    /// and the message itself is passed to the callback so that it can be used to build the result
    pub fn msg<Msg: actix::Message, Cb>(mut self, mut cb: Cb) -> Self
        where
        Msg: 'static ,
        Cb: FnMut(&Msg) -> Msg::Result,
        Cb: 'static {
        self.callbacks.push(Box::new(move |raw_msg| {
            let msg = raw_msg.downcast_ref::<Msg>().unwrap();
            let result: <Msg as actix::Message>::Result = cb(msg);
            Box::new(Some(result))
        }));
        self
    }

    /// Fnalize the sequence and build the actor. Returns an `Addr` to the actor.
    /// Must provide the actor type
    pub fn build<A: Actor>(mut self) -> Addr<Mocker<A>> {
        actix::actors::mocker::Mocker::mock(Box::new(move |raw_msg, _ctx| {
            let result = self.callbacks.get_mut(self.current).expect("unexpected message in MockActorSequence::build")(raw_msg);
            self.current += 1;
            result
        })).start()
    }
}

/// reduced boilerplate helper for if you have just a single message you expect.
pub fn simple_mock_actor<A: Actor, Msg: actix::Message, Cb: FnMut(&Msg) -> Msg::Result>(cb: Cb) -> Addr<Mocker<A>>
where
      Msg: 'static,
      Cb: 'static {
  MockActorSequence::new().msg(cb).build()
}

#[cfg(test)]
mod tests {

    use actix::{Actor, Message, actors::mocker::Mocker, Addr};
    use super::*;

    struct FakeActor;

    impl Actor for FakeActor {
        type Context = actix::Context<Self>;
    }

    struct Msg1;
    struct Msg2;

    struct UnknownMessage;

    impl Message for Msg1 {
    type Result = i32;
    }

    impl Message for Msg2 {
    type Result = bool;
    }

    impl Message for UnknownMessage {
    type Result = bool;
    }

    #[actix_rt::test]
    async fn can_mock_sequence() {
    let mock_actor = MockActorSequence::new()
        .msg(|_m: &Msg1| 5)
        .msg(|_m: &Msg2| true)
        .msg(|_m: &Msg1| 42)
        .build::<FakeActor>();

    assert_eq!(mock_actor.send(Msg1).await.unwrap(), 5);
    assert_eq!(mock_actor.send(Msg2).await.unwrap(), true);
    assert_eq!(mock_actor.send(Msg1).await.unwrap(), 42);
    assert!(mock_actor.send(Msg2).await.is_err());
    }

    #[actix_rt::test]
    async fn message_type_must_match() {
    let mock_actor = MockActorSequence::new()
        .msg(|_m: &Msg1| 5)
        .build::<FakeActor>();

    assert!(mock_actor.send(UnknownMessage).await.is_err());
    }

    #[actix_rt::test]
    async fn simple_works() {
    let mock_actor: Addr<Mocker<FakeActor>> = simple_mock_actor(|_m: &Msg1| 5);

    assert_eq!(mock_actor.send(Msg1).await.unwrap(), 5);
    }

}
