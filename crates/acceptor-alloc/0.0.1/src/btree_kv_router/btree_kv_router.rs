use alloc::collections::BTreeMap;
use core::marker::PhantomData;

use accepts::{Accepts, AsyncAccepts};

/// Routes `(Key, Value)` pairs to acceptors stored in a `BTreeMap`.
///
/// If no route exists for the provided key, the fallback acceptor receives the
/// original pair. Downstream acceptors also receive the original `(Key, Value)`
/// so they can keep using the key.
#[must_use = "BTreeKVRouter must be used to route values"]
#[derive(Debug, Clone)]
pub struct BTreeKVRouter<Key, Value, RouteAccepts, Fallback = ()> {
    routes: BTreeMap<Key, RouteAccepts>,
    fallback: Fallback,
    _marker: PhantomData<Value>,
}

impl<Key, Value, RouteAccepts> BTreeKVRouter<Key, Value, RouteAccepts, ()> {
    /// Creates a `BTreeKVRouter` that drops pairs when no route matches.
    pub fn new(routes: BTreeMap<Key, RouteAccepts>) -> Self {
        Self::with_fallback(routes, ())
    }
}

impl<Key, Value, RouteAccepts, Fallback> BTreeKVRouter<Key, Value, RouteAccepts, Fallback> {
    /// Creates a `BTreeKVRouter` with a custom fallback acceptor.
    pub fn with_fallback(routes: BTreeMap<Key, RouteAccepts>, fallback: Fallback) -> Self {
        Self {
            routes,
            fallback,
            _marker: PhantomData,
        }
    }

    pub fn routes(&self) -> &BTreeMap<Key, RouteAccepts> {
        &self.routes
    }

    pub fn routes_mut(&mut self) -> &mut BTreeMap<Key, RouteAccepts> {
        &mut self.routes
    }

    pub fn fallback(&self) -> &Fallback {
        &self.fallback
    }

    pub fn fallback_mut(&mut self) -> &mut Fallback {
        &mut self.fallback
    }
}

impl<Key, Value, RouteAccepts, Fallback> Accepts<(Key, Value)>
    for BTreeKVRouter<Key, Value, RouteAccepts, Fallback>
where
    Key: Ord,
    RouteAccepts: Accepts<(Key, Value)>,
    Fallback: Accepts<(Key, Value)>,
{
    fn accept(&self, pair: (Key, Value)) {
        let (key, value) = pair;

        if let Some(route) = self.routes.get(&key) {
            route.accept((key, value));
            return;
        }

        self.fallback.accept((key, value));
    }
}

impl<Key, Value, RouteAccepts, Fallback> AsyncAccepts<(Key, Value)>
    for BTreeKVRouter<Key, Value, RouteAccepts, Fallback>
where
    Key: Ord,
    RouteAccepts: AsyncAccepts<(Key, Value)>,
    Fallback: AsyncAccepts<(Key, Value)>,
{
    fn accept_async<'a>(&'a self, pair: (Key, Value)) -> impl core::future::Future<Output = ()> + 'a
    where
        Key: 'a,
        Value: 'a,
    {
        async move {
            let (key, value) = pair;

            if let Some(route) = self.routes.get(&key) {
                route.accept_async((key, value)).await;
                return;
            }

            self.fallback.accept_async((key, value)).await;
        }
    }
}
