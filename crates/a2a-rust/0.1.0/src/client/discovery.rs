use std::collections::BTreeMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use reqwest::Url;

use crate::A2AError;
use crate::jsonrpc::PROTOCOL_VERSION;
use crate::types::AgentCard;

/// Discovery cache configuration for remote agent cards.
#[derive(Debug, Clone, Copy)]
pub struct AgentCardDiscoveryConfig {
    /// Maximum time to reuse a cached discovery response.
    pub ttl: Duration,
}

impl Default for AgentCardDiscoveryConfig {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
struct CachedAgentCard {
    card: AgentCard,
    fetched_at: Instant,
}

/// Discovers and caches remote A2A agent cards.
#[derive(Debug)]
pub struct AgentCardDiscovery {
    client: reqwest::Client,
    config: AgentCardDiscoveryConfig,
    cache: RwLock<BTreeMap<String, CachedAgentCard>>,
}

impl Default for AgentCardDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentCardDiscovery {
    /// Create a discovery client with default caching behavior.
    pub fn new() -> Self {
        Self::with_config(AgentCardDiscoveryConfig::default())
    }

    /// Create a discovery client with explicit cache settings.
    pub fn with_config(config: AgentCardDiscoveryConfig) -> Self {
        Self::with_http_client(reqwest::Client::new(), config)
    }

    /// Create a discovery client with a caller-provided HTTP client.
    pub fn with_http_client(client: reqwest::Client, config: AgentCardDiscoveryConfig) -> Self {
        Self {
            client,
            config,
            cache: RwLock::new(BTreeMap::new()),
        }
    }

    /// Discover an agent card, using the cache when still fresh.
    pub async fn discover(&self, base_url: &str) -> Result<AgentCard, A2AError> {
        let base_url = normalize_base_url(base_url)?;
        let cache_key = cache_key(&base_url);

        if let Some(card) = self.cached_card(&cache_key)? {
            return Ok(card);
        }

        self.fetch_and_store(cache_key, base_url).await
    }

    /// Force a fresh agent-card fetch and replace any cached entry.
    pub async fn refresh(&self, base_url: &str) -> Result<AgentCard, A2AError> {
        let base_url = normalize_base_url(base_url)?;
        self.fetch_and_store(cache_key(&base_url), base_url).await
    }

    fn cached_card(&self, cache_key: &str) -> Result<Option<AgentCard>, A2AError> {
        let cache = self
            .cache
            .read()
            .map_err(|_| A2AError::Internal("discovery cache lock poisoned".to_owned()))?;

        let Some(cached) = cache.get(cache_key) else {
            return Ok(None);
        };

        if cached.fetched_at.elapsed() >= self.config.ttl {
            return Ok(None);
        }

        Ok(Some(cached.card.clone()))
    }

    async fn fetch_and_store(
        &self,
        cache_key: String,
        base_url: Url,
    ) -> Result<AgentCard, A2AError> {
        let discovery_url = well_known_agent_card_url(&base_url)?;
        let response = self
            .client
            .get(discovery_url)
            .header("A2A-Version", PROTOCOL_VERSION)
            .send()
            .await?;

        let status = response.status();
        let bytes = response.bytes().await?;
        if !status.is_success() {
            return Err(A2AError::InvalidAgentResponse(format!(
                "agent discovery returned HTTP {}",
                status
            )));
        }

        let card: AgentCard = serde_json::from_slice(&bytes)
            .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()))?;
        let mut cache = self
            .cache
            .write()
            .map_err(|_| A2AError::Internal("discovery cache lock poisoned".to_owned()))?;
        cache.insert(
            cache_key,
            CachedAgentCard {
                card: card.clone(),
                fetched_at: Instant::now(),
            },
        );

        Ok(card)
    }
}

pub(crate) fn normalize_base_url(base_url: &str) -> Result<Url, A2AError> {
    let mut url =
        Url::parse(base_url).map_err(|error| A2AError::InvalidRequest(error.to_string()))?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

pub(crate) fn resolve_interface_url(base_url: &Url, interface_url: &str) -> Result<Url, A2AError> {
    Url::parse(interface_url)
        .or_else(|_| base_url.join(interface_url))
        .map_err(|error| A2AError::InvalidAgentResponse(error.to_string()))
}

pub(crate) fn ensure_trailing_slash(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        let path = format!("{}/", url.path());
        url.set_path(&path);
    }

    url
}

fn cache_key(base_url: &Url) -> String {
    let mut normalized = base_url.clone();
    if normalized.path() != "/" {
        let trimmed = normalized.path().trim_end_matches('/').to_owned();
        normalized.set_path(&trimmed);
    }

    normalized.to_string()
}

fn well_known_agent_card_url(base_url: &Url) -> Result<Url, A2AError> {
    ensure_trailing_slash(base_url.clone())
        .join(".well-known/agent-card.json")
        .map_err(|error| A2AError::InvalidRequest(error.to_string()))
}
