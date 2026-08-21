use core::marker::PhantomData;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Input>` implementation that maps the value before passing it on.
#[must_use = "MapAcceptor must be used to forward mapped values to the next acceptor"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct MapAcceptor<Input, Output, MapFn, NextAccepts>
where
    MapFn: Fn(Input) -> Output,
    NextAccepts: Accepts<Output>,
{
    map_fn: MapFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Input>,
}

impl<Input, Output, MapFn, NextAccepts> MapAcceptor<Input, Output, MapFn, NextAccepts>
where
    MapFn: Fn(Input) -> Output,
    NextAccepts: Accepts<Output>,
{
    /// Creates a new `MapAcceptor`.
    pub fn new(map_fn: MapFn, next_acceptor: NextAccepts) -> Self {
        Self {
            map_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Input, Output, MapFn, NextAccepts> Accepts<Input>
    for MapAcceptor<Input, Output, MapFn, NextAccepts>
where
    MapFn: Fn(Input) -> Output,
    NextAccepts: Accepts<Output>,
{
    fn accept(&self, value: Input) {
        let mapped = (self.map_fn)(value);
        self.next_acceptor.accept(mapped);
    }
}
