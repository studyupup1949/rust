use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<I>` implementation that forwards each item to the next acceptor.
#[must_use = "ForEach must be used to forward iterator items individually"]
#[derive(Debug, Clone)]
pub struct ForEach<Iter, NextAccepts> {
    next_acceptor: NextAccepts,
    _marker: PhantomData<Iter>,
}

impl<Iter, NextAccepts> ForEach<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: Accepts<Iter::Item>,
{
    /// Creates a new `ForEach`.
    pub fn new(next: NextAccepts) -> Self {
        Self {
            next_acceptor: next,
            _marker: PhantomData,
        }
    }
}

impl<Iter, NextAccepts> Accepts<Iter> for ForEach<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: Accepts<Iter::Item>,
{
    fn accept(&self, iter: Iter) {
        for item in iter {
            self.next_acceptor.accept(item);
        }
    }
}
