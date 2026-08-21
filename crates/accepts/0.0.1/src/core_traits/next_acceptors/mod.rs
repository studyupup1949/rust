//! Traits for accessing one or more *next* acceptors from composite types.
//!
//! These helpers are primarily intended for adapter implementations that need
//! to forward values to another acceptor.  They provide ergonomic accessors for
//! obtaining immutable or mutable references while expressing intent in the type
//! system.

mod maybe_next_acceptor;
mod maybe_next_acceptor_mut;
mod next_acceptor;
mod next_acceptor_mut;
mod next_acceptors;
mod next_acceptors_mut;

pub use maybe_next_acceptor::MaybeNextAcceptor;
pub use maybe_next_acceptor_mut::MaybeNextAcceptorMut;
pub use next_acceptor::NextAcceptor;
pub use next_acceptor_mut::NextAcceptorMut;
pub use next_acceptors::NextAcceptors;
pub use next_acceptors_mut::NextAcceptorsMut;
