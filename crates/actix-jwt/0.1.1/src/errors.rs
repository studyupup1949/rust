//! JWT error types.
//!
//! All errors are represented by the [`JwtError`] enum which implements
//! [`std::error::Error`], [`std::fmt::Display`] and
//! [`actix_web::ResponseError`].  Error messages match the
//! [`echo-jwt`](https://github.com/LdDl/echo-jwt) (Go implementation) sentinel
//! errors for API-level compatibility.

use std::fmt;

use actix_web::HttpResponse;
use actix_web::http::StatusCode;

/// Unified error type for the JWT middleware.
///
/// Each variant maps to a specific HTTP status code via the
/// [`actix_web::ResponseError`] implementation.
///
/// # Examples
///
/// ```
/// use actix_jwt::JwtError;
///
/// let err = JwtError::ExpiredToken;
/// assert_eq!(err.to_string(), "token is expired");
/// ```
///
/// Variants that carry a message reproduce it verbatim:
///
/// ```
/// use actix_jwt::JwtError;
///
/// let err = JwtError::TokenParsing("invalid signature".into());
/// assert_eq!(err.to_string(), "invalid signature");
/// assert!(err.is_token_parsing());
/// ```
#[derive(Debug)]
pub enum JwtError {
    /// HMAC secret key was not provided.
    MissingSecretKey,
    /// The authorizer callback rejected the request.
    Forbidden,
    /// No authenticator callback was configured.
    MissingAuthenticator,
    /// The request body could not be parsed into login credentials.
    MissingLoginValues,
    /// The authenticator callback returned an error.
    FailedAuthentication,
    /// JWT signing or encoding failed.
    FailedTokenCreation,
    /// The access token's `exp` claim is in the past.
    ExpiredToken,
    /// The `Authorization` header is present but empty.
    EmptyAuthHeader,
    /// The `exp` claim is missing from the token.
    MissingExpField,
    /// The `exp` claim is not a numeric value.
    WrongFormatOfExp,
    /// The `Authorization` header does not match the expected format.
    InvalidAuthHeader,
    /// The query-string token source was empty.
    EmptyQueryToken,
    /// The cookie token source was empty.
    EmptyCookieToken,
    /// The path-parameter token source was empty.
    EmptyParamToken,
    /// The configured signing algorithm is not supported.
    InvalidSigningAlgorithm,
    /// The private key file could not be read.
    NoPrivKeyFile,
    /// The public key file could not be read.
    NoPubKeyFile,
    /// The private key could not be parsed.
    InvalidPrivKey,
    /// The public key could not be parsed.
    InvalidPubKey,
    /// No `refresh_token` was found in the request.
    MissingRefreshToken,
    /// The refresh token failed validation.
    InvalidRefreshToken,
    /// The refresh token was not found in the store.
    RefreshTokenNotFound,
    /// The refresh token has expired.
    RefreshTokenExpired,
    /// An empty token string was passed to a store method.
    TokenEmpty,
    /// The supplied expiry timestamp is in the past.
    ExpiryInPast,
    /// A token parsing / validation error with a free-form message.
    TokenParsing(String),
    /// A token extraction error with a free-form message.
    TokenExtraction(String),
    /// An internal / infrastructure error with a free-form message.
    Internal(String),
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSecretKey => write!(f, "secret key is required"),
            Self::Forbidden => {
                write!(f, "you don't have permission to access this resource")
            }
            Self::MissingAuthenticator => write!(f, "authenticator func is undefined"),
            Self::MissingLoginValues => write!(f, "missing Username or Password"),
            Self::FailedAuthentication => write!(f, "incorrect Username or Password"),
            Self::FailedTokenCreation => write!(f, "failed to create JWT Token"),
            Self::ExpiredToken => write!(f, "token is expired"),
            Self::EmptyAuthHeader => write!(f, "auth header is empty"),
            Self::MissingExpField => write!(f, "missing exp field"),
            Self::WrongFormatOfExp => write!(f, "exp must be float64 format"),
            Self::InvalidAuthHeader => write!(f, "auth header is invalid"),
            Self::EmptyQueryToken => write!(f, "query token is empty"),
            Self::EmptyCookieToken => write!(f, "cookie token is empty"),
            Self::EmptyParamToken => write!(f, "parameter token is empty"),
            Self::InvalidSigningAlgorithm => write!(f, "invalid signing algorithm"),
            Self::NoPrivKeyFile => write!(f, "private key file unreadable"),
            Self::NoPubKeyFile => write!(f, "public key file unreadable"),
            Self::InvalidPrivKey => write!(f, "private key invalid"),
            Self::InvalidPubKey => write!(f, "public key invalid"),
            Self::MissingRefreshToken => write!(f, "missing refresh_token parameter"),
            Self::InvalidRefreshToken => write!(f, "invalid or expired refresh token"),
            Self::RefreshTokenNotFound => write!(f, "refresh token not found"),
            Self::RefreshTokenExpired => write!(f, "refresh token expired"),
            Self::TokenEmpty => write!(f, "token cannot be empty"),
            Self::ExpiryInPast => write!(f, "token expiry time must be in the future"),
            Self::TokenParsing(msg) => write!(f, "{msg}"),
            Self::TokenExtraction(msg) => write!(f, "{msg}"),
            Self::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for JwtError {}

impl JwtError {
    /// Returns `true` for the [`TokenParsing`](Self::TokenParsing) variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_jwt::JwtError;
    ///
    /// assert!(JwtError::TokenParsing("bad".into()).is_token_parsing());
    /// assert!(!JwtError::Forbidden.is_token_parsing());
    /// ```
    pub fn is_token_parsing(&self) -> bool {
        matches!(self, Self::TokenParsing(_))
    }

    /// Returns `true` for the [`TokenExtraction`](Self::TokenExtraction) variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_jwt::JwtError;
    ///
    /// assert!(JwtError::TokenExtraction("empty".into()).is_token_extraction());
    /// assert!(!JwtError::ExpiredToken.is_token_extraction());
    /// ```
    pub fn is_token_extraction(&self) -> bool {
        matches!(self, Self::TokenExtraction(_))
    }

    /// Returns `true` for the [`Forbidden`](Self::Forbidden) variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use actix_jwt::JwtError;
    ///
    /// assert!(JwtError::Forbidden.is_forbidden());
    /// assert!(!JwtError::ExpiredToken.is_forbidden());
    /// ```
    pub fn is_forbidden(&self) -> bool {
        matches!(self, Self::Forbidden)
    }
}

impl actix_web::ResponseError for JwtError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Forbidden => StatusCode::FORBIDDEN,

            Self::MissingSecretKey
            | Self::MissingAuthenticator
            | Self::FailedTokenCreation
            | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,

            Self::MissingLoginValues
            | Self::EmptyAuthHeader
            | Self::MissingExpField
            | Self::WrongFormatOfExp
            | Self::InvalidAuthHeader
            | Self::EmptyQueryToken
            | Self::EmptyCookieToken
            | Self::EmptyParamToken
            | Self::MissingRefreshToken
            | Self::TokenExtraction(_) => StatusCode::BAD_REQUEST,

            Self::FailedAuthentication
            | Self::ExpiredToken
            | Self::InvalidSigningAlgorithm
            | Self::NoPrivKeyFile
            | Self::NoPubKeyFile
            | Self::InvalidPrivKey
            | Self::InvalidPubKey
            | Self::InvalidRefreshToken
            | Self::RefreshTokenNotFound
            | Self::RefreshTokenExpired
            | Self::TokenEmpty
            | Self::ExpiryInPast
            | Self::TokenParsing(_) => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(serde_json::json!({
            "code": self.status_code().as_u16(),
            "message": self.to_string(),
        }))
    }
}
