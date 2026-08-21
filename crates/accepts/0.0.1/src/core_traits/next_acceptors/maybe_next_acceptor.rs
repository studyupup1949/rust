use super::NextAcceptors;

/// Marker trait implemented only for `core::iter::Once<Option<&'a A>>`.
///
/// This bound allows the blanket implementation to activate exclusively when
/// [`NextAcceptors::Iter`](super::NextAcceptors::Iter) yields at most one
/// optional acceptor.
trait OnceOptionIter<'a, A: 'a + ?Sized>: Iterator<Item = Option<&'a A>> {}
impl<'a, A: 'a> OnceOptionIter<'a, A> for core::iter::Once<Option<&'a A>> {}

/// Provides optional access to a next acceptor.
///
/// The blanket implementation unwraps the inner iterator, panicking if it is
/// unexpectedly empty.  Consumers can use the returned [`Option`] to decide
/// whether there is an acceptor to forward to.
pub trait MaybeNextAcceptor: NextAcceptors {
    fn maybe_next_acceptor(&self) -> Option<&Self::Acceptor<'_>>;
}

impl<T> MaybeNextAcceptor for T
where
    T: NextAcceptors,
    for<'a> <T as NextAcceptors>::Iter<'a>: OnceOptionIter<'a, T::Acceptor<'a>>,
{
    fn maybe_next_acceptor(&self) -> Option<&Self::Acceptor<'_>> {
        // `next_acceptors()` returns `core::iter::Once<Option<&A>>`,
        // so `next()` never yields `None` on the first call.
        self.next_acceptors().next().unwrap()
    }
}
