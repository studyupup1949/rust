//! Wrapper type over two different error types
use std::error::Error;

use snafu::Snafu;

/// An error that can be one of two types
#[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum EitherError<A: Error + 'static, B: Error + 'static> {
    /// Left
    Left {
        /// Underlying error
        source: A,
    },
    /// Right
    Right {
        /// Underyling error
        source: B,
    },
}
