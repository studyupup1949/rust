use super::FirebaseIdpError;
use crate::firebase::user::FirebaseUser;

use serde::{Deserialize, Serialize};
use std::ops::Deref;

const GOOGLE_IDP_NAME: &str = "Google";

/// Identity Provider ID used by `Firebase` for Google sign-ins.
pub const GOOGLE_IDP_ID: &str = "google.com";

/// Wrapper for a Google-specific user ID, extracted from a `Firebase` ID token.
///
/// This is the unique identifier assigned by Google for the user (e.g., their Google account UID).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GoogleUserId(String);

impl GoogleUserId {
    /// Returns the inner Google ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for GoogleUserId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TryFrom<&FirebaseUser> for GoogleUserId {
    type Error = crate::Error;

    /// Attempts to extract the Google user ID from a `FirebaseUser`.
    ///
    /// # Errors
    /// Returns `FirebaseIdpError` if no Google identity is linked to the user.
    fn try_from(firebase_user: &FirebaseUser) -> Result<Self, Self::Error> {
        // Look for "google.com" identity
        let google_user_id = firebase_user
            .firebase
            .identities
            .get(GOOGLE_IDP_ID)
            .and_then(serde_json::Value::as_array)
            .and_then(|arr| arr.first())
            .and_then(serde_json::Value::as_str)
            .ok_or(FirebaseIdpError::MissingIdpClaims {
                provider: GOOGLE_IDP_NAME,
            })
            .map_err(crate::Error::IdpError)
            .map(|s| GoogleUserId(s.to_owned()))?;

        Ok(google_user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::firebase::user::{FirebaseProvider, FirebaseUser};
    use serde_json::{json, Map};

    fn mock_firebase_user_with_google_id(google_uid: &str) -> FirebaseUser {
        let mut identities = Map::new();
        identities.insert(GOOGLE_IDP_ID.to_string(), json!([google_uid]));

        FirebaseUser {
            iss: "https://securetoken.google.com/my-project".into(),
            aud: "my-project".into(),
            sub: "firebase-subject-id".into(),
            iat: 1_600_000_000,
            exp: 1_600_000_360,
            auth_time: 1_600_000_000,
            user_id: "firebase-uid".into(),
            provider_id: Some(GOOGLE_IDP_ID.to_string()),
            name: Some("Test User".into()),
            picture: Some("https://example.com/avatar.png".parse().unwrap()),
            email: Some("test@example.com".parse().unwrap()),
            email_verified: Some(true),
            firebase: FirebaseProvider {
                sign_in_provider: GOOGLE_IDP_ID.to_string(),
                identities,
            },
        }
    }

    #[test]
    fn extracts_google_user_id_successfully() {
        let user = mock_firebase_user_with_google_id("google-uid-123");
        let result = GoogleUserId::try_from(&user);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "google-uid-123");
    }

    #[test]
    fn returns_error_if_google_id_missing() {
        let user = mock_firebase_user_with_google_id("google-uid-123");
        let mut no_google_id_user = user.clone();
        no_google_id_user.firebase.identities.remove(GOOGLE_IDP_ID);

        let result = GoogleUserId::try_from(&no_google_id_user);

        assert!(matches!(
            result,
            Err(crate::Error::IdpError(FirebaseIdpError::MissingIdpClaims { provider }))
                if provider == GOOGLE_IDP_NAME
        ));
    }

    #[test]
    fn returns_error_if_google_id_is_empty_array() {
        let mut user = mock_firebase_user_with_google_id("google-uid-123");
        user.firebase
            .identities
            .insert(GOOGLE_IDP_ID.to_string(), json!([]));

        let result = GoogleUserId::try_from(&user);

        assert!(matches!(
            result,
            Err(crate::Error::IdpError(FirebaseIdpError::MissingIdpClaims { provider }))
                if provider == GOOGLE_IDP_NAME
        ));
    }

    #[test]
    fn returns_error_if_google_id_not_string() {
        let mut user = mock_firebase_user_with_google_id("google-uid-123");
        user.firebase
            .identities
            .insert(GOOGLE_IDP_ID.to_string(), json!([123]));

        let result = GoogleUserId::try_from(&user);

        assert!(matches!(
            result,
            Err(crate::Error::IdpError(FirebaseIdpError::MissingIdpClaims { provider }))
                if provider == GOOGLE_IDP_NAME
        ));
    }
}
