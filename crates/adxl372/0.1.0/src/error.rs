//! Error handling primitives for the ADXL372 driver.

/// Crate-wide result type alias.
pub type Result<T, E> = core::result::Result<T, Error<E>>;

/// Error variants produced by the driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// Any error reported by the underlying bus interface.
    Interface(E),
    /// The provided configuration parameters are invalid.
    InvalidConfig,
    /// The self-test did not pass.
    SelfTestFailed,
    /// The requested operation is not available yet.
    NotReady,
    /// The peripheral did not report the expected identification values.
    DeviceIdMismatch,
}

impl<E> From<E> for Error<E> {
    fn from(err: E) -> Self {
        Self::Interface(err)
    }
}
