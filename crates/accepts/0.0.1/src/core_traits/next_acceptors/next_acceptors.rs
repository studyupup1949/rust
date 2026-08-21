/// Provides iteration over acceptors that should receive the next value.
///
/// The trait associates both the acceptor type and the iterator used to expose
/// it, allowing implementors to decide how many downstream acceptors are
/// available.  Adapters typically implement this trait to forward incoming
/// values to an inner collection of acceptors.
pub trait NextAcceptors {
    type Acceptor<'a>: ?Sized
    where
        Self: 'a;
    type Iter<'a>: Iterator<Item = &'a Self::Acceptor<'a>>
    where
        Self: 'a;

    fn next_acceptors(&self) -> Self::Iter<'_>;
}
