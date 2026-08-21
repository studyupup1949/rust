use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct AsyncRoute<Value, NextAccepts, Predicate, PredicateFut>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
{
    predicate: Predicate,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts, Predicate, PredicateFut>
    AsyncRoute<Value, NextAccepts, Predicate, PredicateFut>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
{
    /// Creates a new `Route`.
    pub fn new(predicate: Predicate, next_acceptor: NextAccepts) -> Self {
        Self {
            predicate,
            next_acceptor,
            _marker: PhantomData,
        }
    }

    pub fn predicate(&self) -> &Predicate {
        &self.predicate
    }

    pub fn predicate_mut(&mut self) -> &mut Predicate {
        &mut self.predicate
    }
}

#[must_use = "RouteAsyncAcceptor must be used to evaluate routing predicates asynchronously"]
#[derive(Debug, Clone)]
pub struct RouteAsyncAcceptor<Value, NextAccepts, Predicate, PredicateFut, Routes>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    Routes: AsRef<[AsyncRoute<Value, NextAccepts, Predicate, PredicateFut>]>,
{
    routes: Routes,
    _marker: PhantomData<(Value, NextAccepts, Predicate)>,
}

impl<Value, NextAccepts, Predicate, PredicateFut, Routes>
    RouteAsyncAcceptor<Value, NextAccepts, Predicate, PredicateFut, Routes>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    Routes: AsRef<[AsyncRoute<Value, NextAccepts, Predicate, PredicateFut>]>,
{
    /// Creates a new `RouteAcceptor`.
    pub fn new(routes: Routes) -> Self {
        Self {
            routes,
            _marker: PhantomData,
        }
    }

    pub fn routes(&self) -> &Routes {
        &self.routes
    }

    pub fn routes_mut(&mut self) -> &mut Routes {
        &mut self.routes
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, NextAccepts, Predicate, PredicateFut, Routes> AsyncAccepts<Value>
    for RouteAsyncAcceptor<Value, NextAccepts, Predicate, PredicateFut, Routes>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    Routes: AsRef<[AsyncRoute<Value, NextAccepts, Predicate, PredicateFut>]>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            let routes = self.routes.as_ref();

            let mut pending_acceptor: Option<&NextAccepts> = None;

            for r in routes {
                if (r.predicate)(&value).await {
                    if let Some(previous) = pending_acceptor {
                        previous.accept_async(value.clone()).await;
                    }
                    pending_acceptor = Some(&r.next_acceptor);
                }
            }

            if let Some(last) = pending_acceptor {
                last.accept_async(value).await;
            }
        }
    }
}
