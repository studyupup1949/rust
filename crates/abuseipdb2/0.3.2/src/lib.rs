use chrono::{DateTime, Duration, Utc};
use reqwest::{header::HeaderMap, IntoUrl};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

pub mod endpoints;
pub mod types;

const DEFAULT_BASE_URL: &str = "https://api.abuseipdb.com/api/v2/";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Rate limit exceeded: retry after {retry_after:?}, limit {limit}, remaining {remaining}, reset {reset:?}")]
    RateLimit {
        retry_after: Duration,
        limit: u32,
        remaining: u32,
        reset: DateTime<Utc>,
    },
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API errors: {0:?}")]
    Other(Vec<types::Error>),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    http: reqwest::Client,
    base: Url,
}

impl Client {
    #[must_use]
    pub fn new<T: ToString>(key: T) -> Self {
        Self::new_with_base(key, DEFAULT_BASE_URL).unwrap()
    }

    #[must_use]
    pub fn new_with_base<TK: ToString, TB: IntoUrl>(key: TK, base_url: TB) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("key", key.to_string().parse().unwrap());
        headers.insert("accept", "application/json".parse().unwrap());

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent("abuseipdb-rs")
            .build()
            .unwrap();

        Ok(Self {
            http,
            base: base_url.into_url()?,
        })
    }

    pub fn base(&self) -> &Url {
        &self.base
    }

    pub fn set_base<T: IntoUrl>(&mut self, base_url: T) -> Result<()> {
        self.base = base_url.into_url()?;
        Ok(())
    }

    fn make_ratelimit_error(&self, headers: &HeaderMap) -> Error {
        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .map(Duration::seconds)
            .unwrap_or_else(|| Duration::seconds(0));

        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let reset = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .map(|v: i64| DateTime::from_timestamp(v, 0).unwrap())
            .unwrap_or_else(|| Utc::now());

        Error::RateLimit {
            retry_after,
            limit,
            remaining,
            reset,
        }
    }

    async fn make_error(&self, response: reqwest::Response) -> Error {
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return self.make_ratelimit_error(response.headers());
        } else {
            #[derive(Debug, Deserialize)]
            struct Response {
                errors: Vec<types::Error>,
            }
            let response = response.json::<Response>().await.unwrap();
            Error::Other(response.errors)
        }
    }

    async fn get<TU, TI, TO>(&self, url: TU, query: TI) -> Result<TO>
    where
        TU: IntoUrl,
        TI: Serialize,
        TO: for<'de> Deserialize<'de>,
    {
        let response = self.http.get(url).query(&query).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(self.make_error(response).await)
        }
    }

    async fn post<TU, TI, TO>(&self, url: TU, body: TI) -> Result<TO>
    where
        TU: IntoUrl,
        TI: Serialize,
        TO: for<'de> Deserialize<'de>,
    {
        let body = serde_urlencoded::to_string(&body).unwrap();
        let response = self
            .http
            .post(url)
            .header("content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(self.make_error(response).await)
        }
    }
}
