//! Async client for joohoi/acme-dns.
//!
//! Wraps the HTTP API described in the acme-dns README: `/register`,
//! `/update`, and `/health`. :contentReference[oaicite:0]{index=0}
//!
//! Typical flow:
//!   1. Call [`AcmeDnsClient::register`] once to get [`Credentials`].
//!   2. Create `_acme-challenge.<yourdomain>` CNAME -> `fulldomain`.
//!   3. On each DNS-01 challenge, call [`AcmeDnsClient::update_txt`]
//!      with those credentials and the new token.

mod error;

pub use crate::error::Error;

use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

/// Credentials returned by `/register` and required for `/update`.
///
/// Example JSON from the acme-dns README: :contentReference[oaicite:1]{index=1}
///
/// ```json
/// {
///   "allowfrom": ["192.168.100.1/24"],
///   "fulldomain": "8e57...dcc6a.auth.acme-dns.io",
///   "password": "htB9mR9D...",
///   "subdomain": "8e57...dcc6a",
///   "username": "c36f50e8-..."
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub subdomain: String,
    pub fulldomain: String,
    #[serde(default)]
    pub allowfrom: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct RegistrationRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    allowfrom: Option<&'a [String]>,
}

#[derive(Debug, Clone, Serialize)]
struct UpdateRequest<'a> {
    subdomain: &'a str,
    txt: &'a str,
}

/// Minimal async client for the acme-dns HTTP API.
///
/// It's intentionally tiny: you configure it with the API base URL,
/// then call `register`, `update_txt`, and `health`.
#[derive(Clone, Debug)]
pub struct AcmeDnsClient {
    base_url: Url,
    http: HttpClient,
}

impl AcmeDnsClient {
    /// Create a new client from the API base URL, e.g. `https://auth.example.org/`.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, Error> {
        let base = Url::parse(base_url.as_ref())?;
        let http = HttpClient::builder().build()?;
        Ok(Self {
            base_url: base,
            http,
        })
    }

    /// Create a client from `ACME_DNS_API_BASE`.
    pub fn from_env() -> Result<Self, Error> {
        let base = std::env::var("ACME_DNS_API_BASE")
            .map_err(|_| Error::MissingEnv("ACME_DNS_API_BASE"))?;
        Self::new(base)
    }

    /// Register a new acme-dns account.
    ///
    /// If `allow_from` is provided, it configures CIDR ranges allowed to call `/update`.
    /// If `None`, the server default is used (often "no restriction" or “caller’s IP”). :contentReference[oaicite:2]{index=2}
    pub async fn register(&self, allow_from: Option<&[String]>) -> Result<Credentials, Error> {
        let url = self.base_url.join("register")?;

        let body = RegistrationRequest {
            allowfrom: allow_from,
        };

        let resp = self.http.post(url).json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if status != StatusCode::CREATED {
            return Err(Error::UnexpectedStatus { status, body: text });
        }

        let creds: Credentials = serde_json::from_str(&text)?;
        Ok(creds)
    }

    /// Update the TXT value associated with the given credentials.
    ///
    /// This is the call your ACME client makes every time the CA
    /// asks you to prove control via DNS-01. :contentReference[oaicite:3]{index=3}
    pub async fn update_txt(&self, creds: &Credentials, txt: &str) -> Result<(), Error> {
        let url = self.base_url.join("update")?;

        let body = UpdateRequest {
            subdomain: &creds.subdomain,
            txt,
        };

        let resp = self
            .http
            .post(url)
            .header("X-Api-User", &creds.username)
            .header("X-Api-Key", &creds.password)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if status != StatusCode::OK {
            return Err(Error::UnexpectedStatus { status, body: text });
        }

        Ok(())
    }

    /// Simple health check (`GET /health`).
    pub async fn health(&self) -> Result<(), Error> {
        let url = self.base_url.join("health")?;
        let resp = self.http.get(url).send().await?;
        let status = resp.status();

        if status != StatusCode::OK {
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::UnexpectedStatus { status, body });
        }

        Ok(())
    }
}

impl Credentials {
    /// Load credentials from environment variables.
    ///
    /// This mirrors the LEGO provider style a bit (API base is separate). :contentReference[oaicite:4]{index=4}
    ///
    /// Required:
    ///   - `ACME_DNS_USERNAME`
    ///   - `ACME_DNS_PASSWORD`
    ///   - `ACME_DNS_SUBDOMAIN`
    ///   - `ACME_DNS_FULLDOMAIN`
    ///
    /// Optional:
    ///   - `ACME_DNS_ALLOWFROM` (comma-separated CIDRs)
    pub fn from_env() -> Result<Self, Error> {
        use std::env;

        let username =
            env::var("ACME_DNS_USERNAME").map_err(|_| Error::MissingEnv("ACME_DNS_USERNAME"))?;
        let password =
            env::var("ACME_DNS_PASSWORD").map_err(|_| Error::MissingEnv("ACME_DNS_PASSWORD"))?;
        let subdomain =
            env::var("ACME_DNS_SUBDOMAIN").map_err(|_| Error::MissingEnv("ACME_DNS_SUBDOMAIN"))?;
        let fulldomain = env::var("ACME_DNS_FULLDOMAIN")
            .map_err(|_| Error::MissingEnv("ACME_DNS_FULLDOMAIN"))?;

        let allowfrom = env::var("ACME_DNS_ALLOWFROM")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            username,
            password,
            subdomain,
            fulldomain,
            allowfrom,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn register_parses_response() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/register");
            then.status(201).json_body(json!({
                "allowfrom": ["192.168.100.1/24"],
                "fulldomain": "8e57.auth.acme-dns.io",
                "password": "pw",
                "subdomain": "8e57",
                "username": "user-uuid"
            }));
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let creds = client.register(None).await.unwrap();

        mock.assert();
        assert_eq!(creds.username, "user-uuid");
        assert_eq!(creds.password, "pw");
        assert_eq!(creds.subdomain, "8e57");
        assert_eq!(creds.fulldomain, "8e57.auth.acme-dns.io");
        assert_eq!(creds.allowfrom, vec!["192.168.100.1/24"]);
    }

    #[tokio::test]
    async fn register_unexpected_status_errors() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/register");
            then.status(400).body("bad request");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let err = client.register(None).await.unwrap_err();

        mock.assert();

        let Error::UnexpectedStatus { status, body } = err else {
            panic!("expected UnexpectedStatus, got {err:?}");
        };

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, "bad request");
    }

    #[tokio::test]
    async fn update_sends_headers_and_body() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/update")
                .header("X-Api-User", "user-uuid")
                .header("X-Api-Key", "pw")
                .json_body(json!({
                    "subdomain": "8e57",
                    "txt": "token123"
                }));

            then.status(200).body("OK");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let creds = Credentials {
            username: "user-uuid".into(),
            password: "pw".into(),
            subdomain: "8e57".into(),
            fulldomain: "8e57.auth.acme-dns.io".into(),
            allowfrom: vec![],
        };

        client.update_txt(&creds, "token123").await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn update_unexpected_status_errors() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/update");
            then.status(400).body("bad_txt");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let creds = Credentials {
            username: "user-uuid".into(),
            password: "pw".into(),
            subdomain: "8e57".into(),
            fulldomain: "8e57.auth.acme-dns.io".into(),
            allowfrom: vec![],
        };

        let err = client.update_txt(&creds, "token123").await.unwrap_err();

        mock.assert();

        let Error::UnexpectedStatus { status, body } = err else {
            panic!("expected UnexpectedStatus, got {err:?}");
        };

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body, "bad_txt");
    }

    #[tokio::test]
    async fn health_ok() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/health");
            then.status(200).body("OK");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        client.health().await.unwrap();
        mock.assert();
    }

    #[tokio::test]
    async fn health_unexpected_status() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/health");
            then.status(500).body("boom");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let err = client.health().await.unwrap_err();

        mock.assert();

        let Error::UnexpectedStatus { status, body } = err else {
            panic!("expected UnexpectedStatus, got {err:?}");
        };

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, "boom");
    }

    #[test]
    fn client_from_env_works() {
        use std::env;

        // `set_var` is unsafe in Rust 2024.
        unsafe {
            env::set_var("ACME_DNS_API_BASE", "https://example.invalid");
        }

        let client = AcmeDnsClient::from_env().unwrap();
        // Just make sure it parses and constructs; we don't need to use it further.
        let _ = client;
    }

    #[test]
    fn client_from_env_missing_env_errors() {
        use std::env;

        // Remove the var to ensure the MissingEnv branch is hit.
        unsafe {
            env::remove_var("ACME_DNS_API_BASE");
        }

        let err = AcmeDnsClient::from_env().unwrap_err();

        let Error::MissingEnv(name) = err else {
            panic!("expected MissingEnv, got {err:?}");
        };
        assert_eq!(name, "ACME_DNS_API_BASE");
    }

    #[test]
    fn credentials_from_env_works() {
        use std::env;

        unsafe {
            env::set_var("ACME_DNS_USERNAME", "u");
            env::set_var("ACME_DNS_PASSWORD", "p");
            env::set_var("ACME_DNS_SUBDOMAIN", "s");
            env::set_var("ACME_DNS_FULLDOMAIN", "s.auth.example.org");
            env::set_var("ACME_DNS_ALLOWFROM", "1.2.3.4/32, 10.0.0.0/8");
        }

        let creds = Credentials::from_env().unwrap();
        assert_eq!(creds.username, "u");
        assert_eq!(creds.password, "p");
        assert_eq!(creds.subdomain, "s");
        assert_eq!(creds.fulldomain, "s.auth.example.org");
        assert_eq!(
            creds.allowfrom,
            vec!["1.2.3.4/32".to_string(), "10.0.0.0/8".to_string()]
        );
    }

    #[test]
    fn new_with_invalid_url_errors() {
        let err = AcmeDnsClient::new("not a url").unwrap_err();

        let Error::Url(_) = err else {
            panic!("expected Error::Url, got {err:?}");
        };
    }

    #[tokio::test]
    async fn register_invalid_json_errors() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/register");
            // 201 but body is not valid JSON
            then.status(201).body("this is not json");
        });

        let client = AcmeDnsClient::new(server.base_url()).unwrap();
        let err = client.register(None).await.unwrap_err();

        mock.assert();

        let Error::Json(_) = err else {
            panic!("expected Error::Json, got {err:?}");
        };
    }
}
