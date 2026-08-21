use super::NextAcceptors;

/// Marker trait implemented only for `core::iter::Once<&'a A>`.
///
/// The bound is used so that the blanket implementation below only applies when
/// [`NextAcceptors::Iter`](super::NextAcceptors::Iter) yields exactly one
/// acceptor.
trait OnceIter<'a, A: 'a + ?Sized>: Iterator<Item = &'a A> {}
impl<'a, A: 'a> OnceIter<'a, A> for core::iter::Once<&'a A> {}

/// Provides access to exactly one next acceptor.
///
/// Implementations are usually derived from [`NextAcceptors`](super::NextAcceptors).
/// The provided
/// blanket implementation panics if the iterator does not contain an element,
/// making misconfigured adapters fail fast during development.
pub trait NextAcceptor: NextAcceptors {
    fn next_acceptor(&self) -> &Self::Acceptor<'_>;
}

impl<T> NextAcceptor for T
where
    T: NextAcceptors,
    for<'a> <T as NextAcceptors>::Iter<'a>: OnceIter<'a, T::Acceptor<'a>>,
{
    fn next_acceptor(&self) -> &Self::Acceptor<'_> {
        self.next_acceptors().next().unwrap()
    }
}
