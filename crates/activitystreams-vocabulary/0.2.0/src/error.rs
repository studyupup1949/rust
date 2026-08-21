/// Represents the error variants for the library.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    Json(String),
    Iri(String),
    Mime(String),
    LanguageTag(String),
    Duration(String),
    Item(String),
    Vocabulary(String),
    Name(String),
    NameMap(String),
    Question(String),
    Place(String),
    Object(String),
    Tombstone(String),
    Multikey(String),
}

impl Error {
    /// Creates a new JSON [Error].
    pub fn json<S: Into<String>>(err: S) -> Self {
        Self::Json(err.into())
    }

    /// Creates a new IRI [Error].
    pub fn iri<S: Into<String>>(err: S) -> Self {
        Self::Iri(err.into())
    }

    /// Creates a new MimeType [Error].
    pub fn mime<S: Into<String>>(err: S) -> Self {
        Self::Mime(err.into())
    }

    /// Creates a new LanguageTag [Error].
    pub fn language_tag<S: Into<String>>(err: S) -> Self {
        Self::LanguageTag(err.into())
    }

    /// Creates a new Duration [Error].
    pub fn duration<S: Into<String>>(err: S) -> Self {
        Self::Duration(err.into())
    }

    /// Creates a new Vocabulary [Error].
    pub fn vocabulary<S: Into<String>>(err: S) -> Self {
        Self::Vocabulary(err.into())
    }

    /// Creates a new Item [Error].
    pub fn item<S: Into<String>>(err: S) -> Self {
        Self::Item(err.into())
    }

    /// Creates a new Name [Error].
    pub fn name<S: Into<String>>(err: S) -> Self {
        Self::Name(err.into())
    }

    /// Creates a new NameMap [Error].
    pub fn name_map<S: Into<String>>(err: S) -> Self {
        Self::NameMap(err.into())
    }

    /// Creates a new Question [Error].
    pub fn question<S: Into<String>>(err: S) -> Self {
        Self::Question(err.into())
    }

    /// Creates a new Place [Error].
    pub fn place<S: Into<String>>(err: S) -> Self {
        Self::Place(err.into())
    }

    /// Creates a new Object [Error].
    pub fn object<S: Into<String>>(err: S) -> Self {
        Self::Object(err.into())
    }

    /// Creates a new Tombstone [Error].
    pub fn tombstone<S: Into<String>>(err: S) -> Self {
        Self::Tombstone(err.into())
    }

    /// Creates a new Multikey [Error].
    pub fn multikey<S: Into<String>>(err: S) -> Self {
        Self::Multikey(err.into())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Json(err) => write!(f, "json: {err}"),
            Self::Iri(err) => write!(f, "IRI: {err}"),
            Self::Mime(err) => write!(f, "mime-type: {err}"),
            Self::LanguageTag(err) => write!(f, "language tag: {err}"),
            Self::Duration(err) => write!(f, "duration: {err}"),
            Self::Vocabulary(err) => write!(f, "vocabulary: {err}"),
            Self::Item(err) => write!(f, "item: {err}"),
            Self::Name(err) => write!(f, "name: {err}"),
            Self::NameMap(err) => write!(f, "name map: {err}"),
            Self::Question(err) => write!(f, "question: {err}"),
            Self::Place(err) => write!(f, "place: {err}"),
            Self::Object(err) => write!(f, "object: {err}"),
            Self::Tombstone(err) => write!(f, "tombstone: {err}"),
            Self::Multikey(err) => write!(f, "multikey: {err}"),
        }
    }
}

impl core::error::Error for Error {}

/// Convenience alias for the library [Result](core::result::Result) type.
pub type Result<T> = core::result::Result<T, Error>;
