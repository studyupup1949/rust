//! Adapter for changing the type of a consumed value when registering a consumer

use async_trait::async_trait;

use crate::traits::{Event, EventConsumer};

/// Adaptor that allows connecting an [`EventConsumer`] to an
/// [`EventProducer`](crate::traits::EventProducer) of a different type using a conversion closure
pub struct Adaptor<I: Event, O: Event, C: EventConsumer<O>> {
    /// The closure used to map the call
    closure: Box<dyn Fn(I) -> Option<O> + Send + Sync>,
    /// The wrapped consumer
    consumer: C,
}

impl<I: Event, O: Event, C: EventConsumer<O>> Adaptor<I, O, C> {
    /// Create a new [`EventConsumer`], using the provided mapping function on the values passing
    /// through.
    ///
    /// The provided closure should return `Some(O)` if the conversion is possible, and `None`
    /// otherwise. [`Event`]s for which `None` is returned are ignored and not passed through to the
    /// underlying consumer.
    pub fn wrap(closure: impl Fn(I) -> Option<O> + Send + Sync + 'static, consumer: C) -> Self {
        Self {
            closure: Box::new(closure),
            consumer,
        }
    }
}

#[async_trait]
impl<I: Event, O: Event, C: EventConsumer<O>> EventConsumer<I> for Adaptor<I, O, C> {
    type Error = <C as EventConsumer<O>>::Error;
    async fn accept(&self, event: I) -> Result<(), Self::Error> {
        match (self.closure)(event) {
            Some(x) => self.consumer.accept(x).await,
            None => Ok(()),
        }
    }
    fn accept_sync(&self, event: I) -> Result<(), Self::Error> {
        match (self.closure)(event) {
            Some(x) => self.consumer.accept_sync(x),
            None => Ok(()),
        }
    }
}
