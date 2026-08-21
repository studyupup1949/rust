use accepts::Accepts;
use core::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct RouterEntry<Value, NextAccepts, Predicate> {
    predicate: Predicate,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts, Predicate> RouterEntry<Value, NextAccepts, Predicate>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
{
    /// Creates a new `RouterEntry`.
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

#[must_use = "Router must be used to evaluate routing predicates"]
#[derive(Debug, Clone)]
pub struct Router<Value, NextAccepts, Predicate, Routes> {
    routes: Routes,
    _marker: PhantomData<(Value, NextAccepts, Predicate)>,
}

impl<Value, NextAccepts, Predicate, Routes> Router<Value, NextAccepts, Predicate, Routes>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
    Routes: AsRef<[RouterEntry<Value, NextAccepts, Predicate>]>,
{
    /// Creates a new `Router`.
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
    for Router<Value, NextAccepts, Predicate, Routes>
where
    Value: Clone,
    NextAccepts: Accepts<Value>,
    Predicate: Fn(&Value) -> bool,
    Routes: AsRef<[RouterEntry<Value, NextAccepts, Predicate>]>,
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
