use core::{future::Future, hash::Hash};
use std::collections::HashSet;

use tokio::sync::Mutex;

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<Value>` implementation that forwards each unique value only once.
#[must_use = "DeduplicateAsyncAcceptor must be used to suppress duplicate async values"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct DeduplicateAsyncAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: AsyncAccepts<Value>,
{
    seen: Mutex<HashSet<Value>>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<Value, NextAccepts> DeduplicateAsyncAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `DeduplicateAsyncAcceptor`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self::with_seen(Mutex::new(HashSet::new()), next_acceptor)
    }

    pub fn with_seen(seen: Mutex<HashSet<Value>>, next_acceptor: NextAccepts) -> Self {
        Self {
            seen,
            next_acceptor,
        }
    }
}

#[auto_impl_dyn_internal]
impl<Value, NextAccepts> AsyncAccepts<Value> for DeduplicateAsyncAcceptor<Value, NextAccepts>
where
    Value: Clone + Eq + Hash,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            let mut seen = self.seen.lock().await;
            let insert = seen.insert(value.clone());

            drop(seen);

            if insert {
                self.next_acceptor.accept_async(value).await;
            }
        }
    }
}
