use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct Route<Value, NextAccepts, Predicate>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
{
    predicate: Predicate,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts, Predicate> Route<Value, NextAccepts, Predicate>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
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

#[must_use = "RouteAcceptor must be used to evaluate routing predicates"]
#[derive(Debug, Clone)]
pub struct RouteAcceptor<Value, NextAccepts, Predicate, Routes>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
    Routes: AsRef<[Route<Value, NextAccepts, Predicate>]>,
{
    routes: Routes,
    _marker: PhantomData<(Value, NextAccepts, Predicate)>,
}

impl<Value, NextAccepts, Predicate, Routes> RouteAcceptor<Value, NextAccepts, Predicate, Routes>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
    Routes: AsRef<[Route<Value, NextAccepts, Predicate>]>,
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

impl<Value, NextAccepts, Predicate, Routes> Accepts<Value>
    for RouteAcceptor<Value, NextAccepts, Predicate, Routes>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
    Routes: AsRef<[Route<Value, NextAccepts, Predicate>]>,
{
    fn accept(&self, value: Value) {
        let routes = self.routes.as_ref();

        let mut pending_acceptor: Option<&NextAccepts> = None;

        for r in routes {
            if (r.predicate)(&value) {
                if let Some(previous) = pending_acceptor {
                    previous.accept(value.clone());
                }
                pending_acceptor = Some(&r.next_acceptor);
            }
        }

        if let Some(last) = pending_acceptor {
            last.accept(value);
        }
    }
}
