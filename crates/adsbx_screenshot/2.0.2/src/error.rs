use thiserror::Error;

/// The adsbx_browser error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error returned by the Chrome browser.
    #[error("Browser error: {0}")]
    BrowserError(String),

    /// URL could not be parsed into a valid set of options
    #[error("URL structure error: {0}")]
    UrlStructureError(String),

    /// Error related to configuration options.
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

impl From<failure::Error> for Error
where
    failure::Error: std::fmt::Debug,
{
    fn from(error: failure::Error) -> Self {
        Error::BrowserError(format!("{:?}", error))
    }
}
