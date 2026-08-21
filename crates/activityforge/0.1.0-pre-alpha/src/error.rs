use activitystreams_vocabulary::Error as ActivityStreamsError;

/// Represents the error variants for the library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Vocabulary(String),
    Commit(String),
    Content(String),
    Hash(String),
    Activity(String),
    Object(String),
    ActivityStreams(ActivityStreamsError),
    Sql(String),
    Io(String),
    Uuid(String),
    Crypto(String),
}

impl Error {
    /// Creates a new [Vocabulary](Self::Vocabulary) error.
    pub fn vocabulary<I: Into<String>>(err: I) -> Self {
        Self::Vocabulary(err.into())
    }

    /// Creates a new [Commit](Self::Commit) error.
    pub fn commit<I: Into<String>>(err: I) -> Self {
        Self::Commit(err.into())
    }

    /// Creates a new [Content](Self::Content) error.
    pub fn content<I: Into<String>>(err: I) -> Self {
        Self::Content(err.into())
    }

    /// Creates a new [Hash](Self::Hash) error.
    pub fn hash<I: Into<String>>(err: I) -> Self {
        Self::Hash(err.into())
    }

    /// Creates a new [Activity](Self::Activity) error.
    pub fn activity<I: Into<String>>(err: I) -> Self {
        Self::Activity(err.into())
    }

    /// Creates a new [Object](Self::Object) error.
    pub fn object<I: Into<String>>(err: I) -> Self {
        Self::Object(err.into())
    }

    /// Creates a new [ActivityStreams](Self::ActivityStreams) error.
    pub fn activity_streams<I: Into<ActivityStreamsError>>(err: I) -> Self {
        Self::ActivityStreams(err.into())
    }

    /// Creates a new I/O error.
    pub fn io<I: Into<String>>(err: I) -> Self {
        Self::Io(err.into())
    }

    /// Creates a new SQL error.
    pub fn sql<I: Into<String>>(err: I) -> Self {
        Self::Sql(err.into())
    }

    /// Creates a new UUID error.
    pub fn uuid<I: Into<String>>(err: I) -> Self {
        Self::Uuid(err.into())
    }

    /// Creates a new cryptography error.
    pub fn crypto<I: Into<String>>(err: I) -> Self {
        Self::Crypto(err.into())
    }
}

impl From<ActivityStreamsError> for Error {
    fn from(err: ActivityStreamsError) -> Self {
        Self::ActivityStreams(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::sql(err.to_string())
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        Self::sql(err.to_string())
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        Self::uuid(err.to_string())
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::io(format!("async join error: {err}"))
    }
}

impl From<argon2::Error> for Error {
    fn from(err: argon2::Error) -> Self {
        Self::crypto(err.to_string())
    }
}

impl From<chacha20::Error> for Error {
    fn from(err: chacha20::Error) -> Self {
        Self::crypto(err.to_string())
    }
}

impl From<sha3::digest::InvalidLength> for Error {
    fn from(err: sha3::digest::InvalidLength) -> Self {
        Self::crypto(err.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Vocabulary(err) => write!(f, "vocabulary: {err}"),
            Self::Commit(err) => write!(f, "commit: {err}"),
            Self::Content(err) => write!(f, "content: {err}"),
            Self::Hash(err) => write!(f, "hash: {err}"),
            Self::Activity(err) => write!(f, "activity: {err}"),
            Self::Object(err) => write!(f, "object: {err}"),
            Self::ActivityStreams(err) => write!(f, "activitystreams_vocabulary: {err}"),
            Self::Sql(err) => write!(f, "sql: {err}"),
            Self::Io(err) => write!(f, "io: {err}"),
            Self::Uuid(err) => write!(f, "uuid: {err}"),
            Self::Crypto(err) => write!(f, "crypto: {err}"),
        }
    }
}

impl core::error::Error for Error {}

/// Convenience alias for the crate [Result](core::result::Result) type.
pub type Result<T> = core::result::Result<T, Error>;
