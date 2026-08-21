use thiserror::Error;

/// The adsbx_browser error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error returned by the Chrome browser.
    #[error("Browser error: {0}")]
    ChromeError(String),
}

impl From<failure::Error> for Error
where
    failure::Error: std::fmt::Debug,
{
    fn from(error: failure::Error) -> Self {
        Error::ChromeError(format!("{:?}", error))
    }
}
