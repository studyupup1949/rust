use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Represents the decoded JWT claims from a Firebase Authentication token.
///
/// This struct maps to the standard fields provided by Firebase ID tokens.
/// See: <https://firebase.google.com/docs/auth/admin/verify-id-tokens#verify_id_tokens_using_a_third-party_jwt_library>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirebaseUser {
    /// Issuer of the token (typically Firebase project URL)
    pub iss: String,

    /// Audience for the token (your Firebase project ID)
    pub aud: String,

    /// Subject — the unique identifier for the user (usually equals `user_id`)
    pub sub: String,

    /// Issued-at time (epoch seconds)
    pub iat: u64,

    /// Expiration time (epoch seconds)
    pub exp: u64,

    /// Time the user authenticated (epoch seconds)
    pub auth_time: u64,

    /// Firebase UID of the user
    pub user_id: String,

    /// The identity provider used to sign in (e.g., "google.com")
    pub provider_id: Option<String>,

    /// User's display name (if available)
    pub name: Option<String>,

    /// URL to the user's profile picture (if available)
    pub picture: Option<String>,

    /// User's email address
    pub email: Option<String>,

    /// Whether the user's email has been verified
    pub email_verified: Option<bool>,

    /// Additional Firebase-specific claims (provider info, linked accounts)
    pub firebase: FirebaseProvider,
}

/// Firebase-specific metadata included in the token under the `firebase` field.
///
/// This contains provider info and linked account identities (e.g., Google UID).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirebaseProvider {
    /// The main sign-in provider used (e.g., "google.com", "password")
    pub sign_in_provider: String,

    /// A map of identity providers to a list of unique IDs (e.g., `{ "google.com": ["1234567890"] }`)
    pub identities: Map<String, Value>,
}
