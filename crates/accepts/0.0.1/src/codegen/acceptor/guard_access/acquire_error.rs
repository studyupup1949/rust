/// Error returned by [`MutGuardAccess::acquire`].
pub struct AcquireError<Error, Guard> {
    /// The error that occurred while acquiring the guard.
    pub error: Error,
    /// The guard owned despite the acquisition failure.
    pub guard: Option<Guard>,
}
impl<Error, Guard> AcquireError<Error, Guard> {
    pub fn new(error: Error, guard: Option<Guard>) -> Self {
        Self { error, guard }
    }

    #[allow(dead_code)]
    pub fn from_error(error: Error) -> Self {
        Self::new(error, None)
    }
}
