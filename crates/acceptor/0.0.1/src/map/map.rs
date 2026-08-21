use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<Input>` implementation that maps the value before passing it on.
#[must_use = "Map must be used to forward mapped values to the next acceptor"]
#[derive(Debug, Clone)]
pub struct Map<Input, Output, MapFn, NextAccepts> {
    map_fn: MapFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output, MapFn, NextAccepts> Map<Input, Output, MapFn, NextAccepts>
where
    MapFn: Fn(Input) -> Output,
    NextAccepts: Accepts<Output>,
{
    /// Creates a new `Map`.
    pub fn new(map_fn: MapFn, next_acceptor: NextAccepts) -> Self {
        Self {
            map_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Input, Output, MapFn, NextAccepts> Accepts<Input> for Map<Input, Output, MapFn, NextAccepts>
where
    MapFn: Fn(Input) -> Output,
    NextAccepts: Accepts<Output>,
{
    fn accept(&self, value: Input) {
        let mapped = (self.map_fn)(value);
        self.next_acceptor.accept(mapped);
    }
}
