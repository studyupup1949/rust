/// Carries an error together with the source value that triggered it.
pub struct GuardErrorWithSource<Error, Source> {
    /// The error that occurred
    pub error: Error,
    /// The source associated with the error
    pub source: Source,
}

impl<Error, Source> GuardErrorWithSource<Error, Source> {
    /// Creates a new guard that preserves the error and its originating source.
    pub fn new(error: Error, source: Source) -> Self {
        Self { error, source }
    }
}
