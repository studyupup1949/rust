use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

#[derive(Debug, Clone)]
pub struct AsyncRouterEntry<Value, NextAccepts, Predicate, PredicateFut> {
    predicate: Predicate,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, PredicateFut)>,
}

impl<Value, NextAccepts, Predicate, PredicateFut>
    AsyncRouterEntry<Value, NextAccepts, Predicate, PredicateFut>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
{
    /// Creates a new `AsyncRouterEntry`.
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

#[must_use = "AsyncRouter must be used to evaluate routing predicates asynchronously"]
#[derive(Debug, Clone)]
pub struct AsyncRouter<Value, NextAccepts, Predicate, PredicateFut, Routes> {
    routes: Routes,
    _marker: PhantomData<(Value, NextAccepts, Predicate, PredicateFut)>,
}

impl<Value, NextAccepts, Predicate, PredicateFut, Routes>
    AsyncRouter<Value, NextAccepts, Predicate, PredicateFut, Routes>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    Routes: AsRef<[AsyncRouterEntry<Value, NextAccepts, Predicate, PredicateFut>]>,
{
    /// Creates a new `AsyncRouter`.
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

impl<Value, NextAccepts, Predicate, PredicateFut, Routes> AsyncAccepts<Value>
    for AsyncRouter<Value, NextAccepts, Predicate, PredicateFut, Routes>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    Routes: AsRef<[AsyncRouterEntry<Value, NextAccepts, Predicate, PredicateFut>]>,
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
