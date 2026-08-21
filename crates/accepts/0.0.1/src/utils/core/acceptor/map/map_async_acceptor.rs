use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<Input>` implementation that maps the value before passing it on.
#[must_use = "MapAsyncAcceptor must be used to forward mapped async results"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct MapAsyncAcceptor<Input, Output, MapFn, MapFut, NextAccepts>
where
    MapFn: Fn(Input) -> MapFut,
    MapFut: Future<Output = Output>,
    NextAccepts: AsyncAccepts<Output>,
{
    map_fn: MapFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Input>,
}

impl<Input, Output, MapFn, MapFut, NextAccepts>
    MapAsyncAcceptor<Input, Output, MapFn, MapFut, NextAccepts>
where
    MapFn: Fn(Input) -> MapFut,
    MapFut: Future<Output = Output>,
    NextAccepts: AsyncAccepts<Output>,
{
    /// Creates a new `MapAsyncAcceptor`.
    pub fn new(map_fn: MapFn, next_acceptor: NextAccepts) -> Self {
        Self {
            map_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Input, Output, MapFn, MapFut, NextAccepts> AsyncAccepts<Input>
    for MapAsyncAcceptor<Input, Output, MapFn, MapFut, NextAccepts>
where
    MapFn: Fn(Input) -> MapFut,
    MapFut: Future<Output = Output>,
    NextAccepts: AsyncAccepts<Output>,
{
    fn accept_async<'a>(&'a self, value: Input) -> impl Future<Output = ()> + 'a
    where
        Input: 'a,
    {
        async {
            let mapped = (self.map_fn)(value).await;
            self.next_acceptor.accept_async(mapped).await;
        }
    }
}
