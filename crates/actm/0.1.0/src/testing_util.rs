#![allow(dead_code)]
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use proptest_derive::Arbitrary;

use crate::{
    async_actor,
    executor::Executor,
    sync_actor,
    traits::{Actor, Event, EventConsumer, EventProducer},
    types::{CompletionToken, Waiter},
    util::{AsyncActor, AsyncActorError, SyncActor, SyncActorError, WrappedEvent},
    wrapped_event,
};

// Define the Event types, here the context type is an i64

/// Dummy Event implementation for use in unit testing
#[enum_dispatch]
pub trait Math {
    // Operate on a value
    fn operate(&self, value: i64) -> i64;
}

#[derive(Arbitrary, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Default)]
pub struct Add(pub(crate) i64);
impl Math for Add {
    fn operate(&self, value: i64) -> i64 {
        value.overflowing_add(self.0).0
    }
}

#[derive(Arbitrary, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Default)]
pub struct Subtract(pub(crate) i64);
impl Math for Subtract {
    fn operate(&self, value: i64) -> i64 {
        value.overflowing_sub(self.0).0
    }
}

#[enum_dispatch(Math)]
#[derive(Arbitrary, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub enum MathEventType {
    Add,
    Subtract,
}

#[derive(Arbitrary, Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub struct Output {
    pub before: i64,
    pub after: i64,
    pub input: MathEventType,
}

wrapped_event!(MathEvent, MathEventType);
wrapped_event!(OutputEvent, Output);

// Async actor definition

#[allow(clippy::unused_async)]
async fn async_math_event_handler(mut value: i64, math: MathEvent) -> (i64, Option<OutputEvent>) {
    // Perform the operation
    let old_value = value;
    let math = math.into_inner();
    value = math.operate(value);
    // Check to see if there was a completion token, if so, send back an Output
    // Make our output
    let output = Output {
        before: old_value,
        after: value,
        input: math,
    };
    // Wrap it up
    let output = OutputEvent::from(output);
    // Send it up
    (value, Some(output))
}

async_actor!(
    AsyncMathActor,
    MathEvent,
    OutputEvent,
    i64,
    async_math_event_handler
);

// Sync Actor Definition

#[allow(clippy::unnecessary_wraps)]
fn math_event_handler(value: &mut i64, math: MathEvent) -> Option<OutputEvent> {
    // Perform the operation
    let old_value = *value;
    let math = math.into_inner();
    *value = math.operate(*value);
    // Check to see if there was a completion token, if so, send back an Output
    // Make our output
    let output = Output {
        before: old_value,
        after: *value,
        input: math,
    };
    // Wrap it up
    let output = OutputEvent::from(output);
    // Send it up
    Some(output)
}

sync_actor!(
    SyncMathActor,
    MathEvent,
    OutputEvent,
    i64,
    math_event_handler
);

mod tests {
    use super::*;
    use crate::traits::ActorExt;

    #[cfg(feature = "async-std")]
    mod async_math_actor {

        use futures::StreamExt;

        use super::*;
        use crate::executor::AsyncStd;
        #[async_std::test]
        async fn smoke() {
            let actor = AsyncMathActor::<AsyncStd>::new(0, Some(1));
            let output = actor.stream(None).await.unwrap();
            // Wait for the subscribe to happen before feeding in events
            assert!(actor.catchup().wait().await);
            // Add some numbers to our internal count
            for i in 1..=10 {
                let event: MathEvent = MathEventType::Add(Add(i)).into();
                actor.inbox().accept(event).await.unwrap();
            }
            // Make sure we have the correct count
            let count_event = actor
                .call(MathEventType::Add(Add(0)).into())
                .await
                .unwrap()
                .unwrap()
                .into_inner();
            assert_eq!(count_event.after, 55);
            println!("Past first assert");
            // Make sure our events are as expected
            let output_events = output
                .stream()
                .take(11)
                .map(|x| x.into_inner().input)
                .collect::<Vec<_>>()
                .await;
            println!("Events in output: {:?}", output_events);
            assert_eq!(
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0]
                    .into_iter()
                    .map(|x| MathEventType::Add(Add(x)))
                    .collect::<Vec<_>>(),
                output_events
            );
            // Make sure the actor shuts down
            assert!(actor.shutdown().wait().await);
        }
    }

    mod sync_math_actor {
        use super::*;
        use crate::executor::Threads;

        #[test]
        fn smoke() {
            let actor = SyncMathActor::<Threads>::new(0, Some(1));
            let output = actor.stream_sync(None).unwrap();
            // Sleep for a bit to let the subscription take
            assert!(actor.catchup().wait_sync());
            println!("Started up actor");
            // Add some numbers to our internal count
            for i in 1..=10 {
                println!("Sending event {i}");
                let event: MathEvent = MathEventType::Add(Add(i)).into();
                actor.inbox().accept_sync(event).unwrap();
                println!("Sent event {i}");
            }
            // Make sure we have the correct count
            println!("Getting counter");
            let count_event = actor
                .call_sync(MathEventType::Add(Add(0)).into())
                .unwrap()
                .unwrap()
                .into_inner();
            println!("Got Counter");
            assert_eq!(count_event.after, 55);
            println!("Past first assert");
            // Make sure our events are as expected
            let output_events = output
                .iter()
                .take(11)
                .map(|x| x.into_inner().input)
                .collect::<Vec<_>>();
            println!("Events in output: {:?}", output_events);
            assert_eq!(
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0]
                    .into_iter()
                    .map(|x| MathEventType::Add(Add(x)))
                    .collect::<Vec<_>>(),
                output_events
            );
            // Make sure the actor shuts down properly
            assert!(actor.shutdown().wait_sync());
        }
    }
}
