use super::NextAcceptorsMut;

/// Marker trait implemented only for `core::iter::Once<&'a mut A>`.
///
/// The bound ensures that the blanket implementation below only triggers when
/// the iterator produced by
/// [`NextAcceptorsMut::Iter`](super::NextAcceptorsMut::Iter) yields a single
/// mutable reference.
trait OnceIterMut<'a, A: 'a + ?Sized>: Iterator<Item = &'a mut A> {}
impl<'a, A: 'a> OnceIterMut<'a, A> for core::iter::Once<&'a mut A> {}

/// Provides mutable access to exactly one next acceptor.
///
/// The blanket implementation panics if no acceptor is available, mirroring the
/// behaviour of [`NextAcceptor`](super::NextAcceptor) for the mutable case.
pub trait NextAcceptorMut: NextAcceptorsMut {
    fn next_acceptor_mut(&mut self) -> &mut Self::Acceptor<'_>;
}

impl<T> NextAcceptorMut for T
where
    T: NextAcceptorsMut,
    for<'a> <T as NextAcceptorsMut>::Iter<'a>: OnceIterMut<'a, T::Acceptor<'a>>,
{
    fn next_acceptor_mut(&mut self) -> &mut Self::Acceptor<'_> {
        self.next_acceptors_mut().next().unwrap()
    }
}
