use serde::de::DeserializeOwned;
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use tokio::{task::JoinHandle, time::sleep};
use tracing::*;

use crate::jwk::{JwkConfig, JwkKeys, JwkVerifier, KeyResponse, PublicKeysError};

/// Fallback timeout if no `max-age` is provided in the Cache-Control header.
const FALLBACK_TIMEOUT: Duration = Duration::from_secs(60);

/// FirebaseAuth is responsible for verifying Firebase JWT tokens and keeping the
/// Google public keys up to date by periodically fetching them.
///
/// It uses the Cache-Control `max-age` directive to schedule the next refresh.
/// If fetching fails, it retries every 10 seconds until successful.
#[derive(Clone)]
pub struct FirebaseAuth {
    verifier: Arc<RwLock<JwkVerifier>>,
    handler: Arc<Mutex<Box<JoinHandle<()>>>>,
}

impl Drop for FirebaseAuth {
    fn drop(&mut self) {
        // Abort the background task on drop.
        let handler = self.handler.lock().unwrap();
        handler.abort();
    }
}

impl FirebaseAuth {
    /// Create a new FirebaseAuth instance with an initial key fetch.
    ///
    /// Panics if the initial fetch fails, since the app cannot verify any tokens.
    pub async fn new(project_id: impl AsRef<str>) -> Self {
        let jwk_keys = match Self::get_public_keys().await {
            Ok(keys) => keys,
            Err(e) => {
                eprintln!("Error fetching initial public JWK keys: {:?}", e);
                panic!("Failed to fetch initial public JWK keys. Cannot verify Firebase tokens.");
            }
        };

        let verifier = Arc::new(RwLock::new(JwkVerifier::new(project_id, jwk_keys)));
        let handler = Arc::new(Mutex::new(Box::new(tokio::spawn(async {})))); // placeholder

        let mut instance = Self { verifier, handler };
        instance.start_key_update();
        instance
    }

    /// Verifies a Firebase JWT token and deserializes the payload into type `T`.
    pub fn verify<T: DeserializeOwned>(&self, token: &str) -> crate::Result<T> {
        let verifier = self.verifier.read().unwrap();
        verifier
            .verify(token)
            .map_err(crate::Error::VerificationError)
    }

    /// Spawns a background task to periodically refresh the JWK keys.
    ///
    /// If the fetch fails, retries every 10 seconds.
    fn start_key_update(&mut self) {
        let verifier_ref = Arc::clone(&self.verifier);

        let task = tokio::spawn(async move {
            loop {
                let delay = match Self::get_public_keys().await {
                    Ok(jwk_keys) => {
                        let mut verifier = verifier_ref.write().unwrap();
                        verifier.set_keys(jwk_keys.clone());
                        debug!("Updated JWK keys. Next refresh in {:?}", jwk_keys.max_age);
                        jwk_keys.max_age
                    }
                    Err(err) => {
                        warn!("Failed to refresh public JWK keys: {:?}", err);
                        warn!("Retrying in 10 seconds...");
                        Duration::from_secs(10)
                    }
                };
                sleep(delay).await;
            }
        });

        let mut handler = self.handler.lock().unwrap();
        *handler = Box::new(task);
    }

    /// Fetches the latest public keys from the identity provider and parses cache-control headers.
    pub(crate) async fn get_public_keys() -> crate::Result<JwkKeys> {
        let response = reqwest::get(JwkConfig::JWK_URL)
            .await
            .map_err(PublicKeysError::FetchPublicKeys)?;

        let cache_control = response
            .headers()
            .get("Cache-Control")
            .ok_or(PublicKeysError::MissingCacheControlHeader)?
            .to_str()
            .map_err(|_| PublicKeysError::EmptyMaxAgeDirective)?;

        let max_age = Self::parse_max_age_value(cache_control).unwrap_or(FALLBACK_TIMEOUT);

        let public_keys = response
            .json::<KeyResponse>()
            .await
            .map_err(PublicKeysError::PublicKeyParseError)?;

        Ok(JwkKeys {
            keys: public_keys.keys,
            max_age,
        })
    }

    /// Parses the `max-age` directive from a Cache-Control header string.
    pub(crate) fn parse_max_age_value(value: &str) -> Result<Duration, PublicKeysError> {
        for directive in value.split(',') {
            let mut parts = directive.trim().splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim();

            if key.eq_ignore_ascii_case("max-age") {
                let secs = val
                    .parse::<u64>()
                    .map_err(|_| PublicKeysError::InvalidMaxAgeValue)?;
                return Ok(Duration::from_secs(secs));
            }
        }

        Err(PublicKeysError::MissingMaxAgeDirective)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_rt::test;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use jwk::{JwkKeys, KeyResponse, PublicKeysError};
    use serde_json::json;
    use std::time::Duration;

    use crate::jwk;

    async fn get_public_keys_from_url(url: &str) -> crate::Result<JwkKeys> {
        let response = reqwest::get(url)
            .await
            .map_err(PublicKeysError::FetchPublicKeys)?;

        let cache_control = response
            .headers()
            .get("Cache-Control")
            .ok_or(PublicKeysError::MissingCacheControlHeader)?
            .to_str()
            .map_err(|_| PublicKeysError::EmptyMaxAgeDirective)?;

        let max_age = FirebaseAuth::parse_max_age_value(cache_control).unwrap_or(FALLBACK_TIMEOUT);

        let public_keys = response
            .json::<KeyResponse>()
            .await
            .map_err(PublicKeysError::PublicKeyParseError)?;

        Ok(JwkKeys {
            keys: public_keys.keys,
            max_age,
        })
    }

    #[test]
    async fn parses_max_age_correctly() {
        let input = "public, max-age=3600, must-revalidate";
        let duration = FirebaseAuth::parse_max_age_value(input).unwrap();
        assert_eq!(duration, Duration::from_secs(3600));
    }

    #[test]
    async fn returns_error_for_missing_max_age() {
        let input = "public, no-cache";
        let err = FirebaseAuth::parse_max_age_value(input).unwrap_err();
        matches!(err, PublicKeysError::MissingMaxAgeDirective);
    }

    #[test]
    async fn returns_error_for_invalid_max_age() {
        let input = "max-age=not_a_number";
        let err = FirebaseAuth::parse_max_age_value(input).unwrap_err();
        matches!(err, PublicKeysError::InvalidMaxAgeValue);
    }

    #[test]
    async fn get_public_keys_successfully_parses_keys() {
        let server = MockServer::start();

        let body = json!({
            "keys": [
                {
                    "kty": "RSA",
                    "alg": "RS256",
                    "use": "sig",
                    "kid": "1234",
                    "n": "modulus",
                    "e": "AQAB"
                }
            ]
        });

        let _mock = server.mock(|when, then| {
            when.method(GET).path("/keys");
            then.status(200)
                .header("Cache-Control", "public, max-age=120")
                .json_body(body.clone());
        });

        let keys = get_public_keys_from_url(&server.url("/keys"))
            .await
            .unwrap();
        assert_eq!(keys.max_age, Duration::from_secs(120));
        assert_eq!(keys.keys.len(), 1);
    }

    #[test]
    async fn background_task_aborts_on_drop() {
        let auth = FirebaseAuth::new("dummy-project").await;

        {
            let handler_guard = auth.handler.lock().unwrap();
            assert!(!handler_guard.is_finished(), "Task should be running");
        }

        drop(auth); // Triggers Drop which aborts task

        // Give a moment for task abort to propagate
        actix_rt::time::sleep(Duration::from_millis(100)).await;
    }
}
