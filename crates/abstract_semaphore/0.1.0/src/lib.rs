mod error;
mod platform;

pub use error::SemaphoreError;
/// A portable abstraction over native operating system semaphores.
///
/// This type provides a common interface over platform-specific semaphore
/// implementations.
///
/// The underlying synchronization primitive is selected automatically based
/// on the target operating system.
pub use platform::Semaphore;

pub type Result<T> = std::result::Result<T, SemaphoreError>;
