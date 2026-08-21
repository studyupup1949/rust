use super::NextAcceptorsMut;

/// Marker trait implemented only for `core::iter::Once<Option<&'a mut A>>`.
///
/// It keeps the blanket implementation scoped to iterators that yield at most
/// one mutable acceptor wrapped in an [`Option`].
trait OnceOptionIterMut<'a, A: 'a + ?Sized>: Iterator<Item = Option<&'a mut A>> {}
impl<'a, A: 'a> OnceOptionIterMut<'a, A> for core::iter::Once<Option<&'a mut A>> {}

/// Provides optional mutable access to a next acceptor.
///
/// The blanket implementation unwraps the iterator, panicking when the iterator
/// is empty.  Consumers can branch on the returned [`Option`] to decide whether
/// forwarding should take place.
pub trait MaybeNextAcceptorMut: NextAcceptorsMut {
    fn maybe_next_acceptor_mut(&mut self) -> Option<&mut Self::Acceptor<'_>>;
}

impl<T> MaybeNextAcceptorMut for T
where
    T: NextAcceptorsMut,
    for<'a> <T as NextAcceptorsMut>::Iter<'a>: OnceOptionIterMut<'a, T::Acceptor<'a>>,
{
    fn maybe_next_acceptor_mut(&mut self) -> Option<&mut Self::Acceptor<'_>> {
        // `next_acceptors_mut()` returns `core::iter::Once<Option<&A>>`,
        // so `next()` never yields `None` on the first call.
        self.next_acceptors_mut().next().unwrap()
    }
}
