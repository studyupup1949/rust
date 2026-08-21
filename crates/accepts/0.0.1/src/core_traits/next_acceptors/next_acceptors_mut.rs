/// Mutable counterpart to [`NextAcceptors`](super::NextAcceptors), yielding mutable references.
///
/// Implement this trait when the inner acceptors need to be configured or
/// mutated before accepting the next value.
pub trait NextAcceptorsMut {
    type Acceptor<'a>: ?Sized
    where
        Self: 'a;
    type Iter<'a>: Iterator<Item = &'a mut Self::Acceptor<'a>>
    where
        Self: 'a;

    fn next_acceptors_mut(&mut self) -> Self::Iter<'_>;
}
