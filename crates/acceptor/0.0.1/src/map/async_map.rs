use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// `AsyncAccepts<Input>` implementation that maps the value before passing it on.
#[must_use = "AsyncMap must be used to forward mapped async results"]
#[derive(Debug, Clone)]
pub struct AsyncMap<Input, Output, MapFn, MapFut, NextAccepts> {
    map_fn: MapFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Input, Output, MapFut)>,
}

impl<Input, Output, MapFn, MapFut, NextAccepts> AsyncMap<Input, Output, MapFn, MapFut, NextAccepts>
where
    MapFn: Fn(Input) -> MapFut,
    MapFut: Future<Output = Output>,
    NextAccepts: AsyncAccepts<Output>,
{
    /// Creates a new `AsyncMap`.
    pub fn new(map_fn: MapFn, next_acceptor: NextAccepts) -> Self {
        Self {
            map_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Input, Output, MapFn, MapFut, NextAccepts> AsyncAccepts<Input>
    for AsyncMap<Input, Output, MapFn, MapFut, NextAccepts>
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
