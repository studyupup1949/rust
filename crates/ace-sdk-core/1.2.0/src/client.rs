//! ACE HTTP client with 3-tier cache (RAM -> SQLite -> Server).
//!
//! Provides authenticated HTTP client for all ACE API operations
//! including playbook retrieval, pattern search, learning traces,
//! and server configuration.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::cache::{GraphCache, LocalCacheService};
use crate::errors::AceError;
use crate::logger::{ILogger, NoopLogger};
use crate::types::*;
use crate::usage::{UsageHistoryResponse, UsageWindow};

/// Default throttle window for the fire-and-forget graph-edge refresh
/// (1 hour in milliseconds — matches the TS reference implementation).
pub const DEFAULT_GRAPH_EDGES_THROTTLE_MS: u64 = 3_600_000;

/// Options for creating an ACE client.
#[derive(Debug, Clone, Default)]
pub struct AceClientOptions {
    /// Enable auto-refresh of expired tokens (default: true for user tokens).
    pub auto_refresh: Option<bool>,
    /// Custom headers to include in all requests.
    pub custom_headers: Option<std::collections::HashMap<String, String>>,
    /// Minimum interval between `GET /patterns/graph` refresh calls
    /// (milliseconds). Defaults to `DEFAULT_GRAPH_EDGES_THROTTLE_MS` (1 h).
    /// Set to `0` to disable throttling (fires on every search).
    pub graph_edges_throttle_ms: Option<u64>,
}

/// ACE API client with 3-tier caching.
pub struct AceClient {
    config: AceConfig,
    http_client: Client,
    local_cache: Option<LocalCacheService>,
    memory_cache: Arc<Mutex<Option<StructuredPlaybook>>>,
    logger: Arc<dyn ILogger>,
    auto_refresh: bool,
    custom_headers: std::collections::HashMap<String, String>,
    config_cache: Arc<Mutex<Option<(ServerConfig, std::time::Instant)>>>,
    /// Most recent usage info parsed from response headers (v0.4.2+).
    /// Parity with TS `getLastUsage()` / Kotlin `lastUsage` / Go `GetLastUsage()`.
    last_usage: Arc<Mutex<Option<UsageInfo>>>,
    /// ACE 1.5 SQLite graph cache (patterns + co-application edges, 7d TTL).
    /// Optional — graceful absent when `org_id` is unknown at construction time.
    graph_cache: Option<Arc<std::sync::Mutex<GraphCache>>>,
    /// Minimum ms between fire-and-forget graph-edge refresh calls.
    graph_edges_throttle_ms: u64,
}

impl AceClient {
    /// Create a new ACE client.
    ///
    /// # Arguments
    /// * `config` - ACE configuration with server URL, token, and project ID.
    /// * `options` - Optional client configuration.
    ///
    /// # Example
    /// ```rust,no_run
    /// use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
    ///
    /// let config = AceConfig {
    ///     server_url: "https://ace-api.code-engine.app".to_string(),
    ///     api_token: "ace_user_xxx".to_string(),
    ///     project_id: "my-project".to_string(),
    ///     ..Default::default()
    /// };
    ///
    /// let client = AceClient::new(config, AceClientOptions::default()).unwrap();
    /// ```
    pub fn new(config: AceConfig, options: AceClientOptions) -> Result<Self, AceError> {
        let logger: Arc<dyn ILogger> = Arc::new(NoopLogger);
        let auto_refresh = options
            .auto_refresh
            .unwrap_or_else(|| is_user_token(&config.api_token));
        let custom_headers = options.custom_headers.unwrap_or_default();
        let graph_edges_throttle_ms = options
            .graph_edges_throttle_ms
            .unwrap_or(DEFAULT_GRAPH_EDGES_THROTTLE_MS);

        let http_client = Client::new();

        // Initialize local SQLite cache (optional - graceful fallback)
        let local_cache = LocalCacheService::new(
            "default",
            &config.project_id,
            config.cache_ttl_minutes,
            None,
        )
        .ok();

        if local_cache.is_some() {
            logger.debug(&format!(
                "SQLite cache enabled (TTL: {} minutes)",
                config.cache_ttl_minutes
            ));
        } else {
            logger.debug("SQLite cache disabled, using RAM-only cache");
        }

        logger.debug(&format!("ACE Server: {}", config.server_url));
        logger.debug(&format!("Project: {}", config.project_id));

        // ACE 1.5 graph cache — use org_id if available, else "default".
        // Creation failures are swallowed; cache is simply absent.
        // `config.graph_cache_dir` is respected so tests can inject a TempDir.
        let org_id = config
            .default_org_id
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let graph_cache =
            GraphCache::new(&org_id, &config.project_id, config.graph_cache_dir.clone())
                .ok()
                .map(|gc| Arc::new(std::sync::Mutex::new(gc)));

        Ok(Self {
            config,
            http_client,
            local_cache,
            memory_cache: Arc::new(Mutex::new(None)),
            logger,
            auto_refresh,
            custom_headers,
            config_cache: Arc::new(Mutex::new(None)),
            last_usage: Arc::new(Mutex::new(None)),
            graph_cache,
            graph_edges_throttle_ms,
        })
    }

    /// Create a new client with a custom logger.
    pub fn with_logger(
        config: AceConfig,
        options: AceClientOptions,
        logger: Arc<dyn ILogger>,
    ) -> Result<Self, AceError> {
        let auto_refresh = options
            .auto_refresh
            .unwrap_or_else(|| is_user_token(&config.api_token));
        let custom_headers = options.custom_headers.unwrap_or_default();
        let graph_edges_throttle_ms = options
            .graph_edges_throttle_ms
            .unwrap_or(DEFAULT_GRAPH_EDGES_THROTTLE_MS);
        let http_client = Client::new();

        let local_cache = LocalCacheService::new(
            "default",
            &config.project_id,
            config.cache_ttl_minutes,
            None,
        )
        .ok();

        let org_id = config
            .default_org_id
            .clone()
            .unwrap_or_else(|| "default".to_string());
        let graph_cache =
            GraphCache::new(&org_id, &config.project_id, config.graph_cache_dir.clone())
                .ok()
                .map(|gc| Arc::new(std::sync::Mutex::new(gc)));

        Ok(Self {
            config,
            http_client,
            local_cache,
            memory_cache: Arc::new(Mutex::new(None)),
            logger,
            auto_refresh,
            custom_headers,
            config_cache: Arc::new(Mutex::new(None)),
            last_usage: Arc::new(Mutex::new(None)),
            graph_cache,
            graph_edges_throttle_ms,
        })
    }

    /// Get the most recent usage info parsed from response headers.
    ///
    /// Returns `None` if no authenticated request has been made yet, or if
    /// the most recent response did not include the `X-ACE-Plan` header.
    ///
    /// Parity: TS `getLastUsage()`, Kotlin `getLastUsage()`,
    /// Go `GetLastUsage()`, Python `get_last_usage()`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use ace_sdk_core::{AceClient, AceClientOptions, AceConfig};
    /// # async fn example(client: &AceClient) {
    /// let _ = client.get_playbook(false).await;
    /// if let Some(usage) = client.get_last_usage().await {
    ///     println!("Plan: {}", usage.plan);
    ///     println!("Patterns used: {}/{}", usage.patterns.used, usage.patterns.limit);
    /// }
    /// # }
    /// ```
    pub async fn get_last_usage(&self) -> Option<UsageInfo> {
        self.last_usage.lock().await.clone()
    }

    /// Parse `X-ACE-*` usage headers from a response into a `UsageInfo`.
    ///
    /// Returns `None` when the `X-ACE-Plan` header is missing — matches
    /// the TS reference behaviour (no spurious updates on free/legacy
    /// responses).
    fn parse_usage_headers(headers: &reqwest::header::HeaderMap) -> Option<UsageInfo> {
        let plan = headers.get("X-ACE-Plan")?.to_str().ok()?.to_string();
        let (sub_type, plan_tier) = parse_plan(&plan);

        let status = match headers
            .get("X-ACE-Status")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("active")
        {
            "trialing" => SubscriptionStatus::Trialing,
            "read_only" => SubscriptionStatus::ReadOnly,
            "blocked" => SubscriptionStatus::Blocked,
            _ => SubscriptionStatus::Active,
        };

        fn parse_metric(headers: &reqwest::header::HeaderMap, name: &str) -> UsageMetric {
            let raw = match headers.get(name).and_then(|v| v.to_str().ok()) {
                Some(s) => s,
                None => return UsageMetric { used: 0, limit: 0 },
            };
            let parts: Vec<&str> = raw.split('/').collect();
            if parts.len() != 2 {
                return UsageMetric { used: 0, limit: -1 };
            }
            let used = match parts[0].parse::<u32>() {
                Ok(n) => n,
                Err(_) => return UsageMetric { used: 0, limit: -1 },
            };
            let limit = match parts[1].parse::<i32>() {
                Ok(n) => n,
                Err(_) => return UsageMetric { used: 0, limit: -1 },
            };
            UsageMetric { used, limit }
        }

        Some(UsageInfo {
            plan,
            subscription_type: sub_type,
            plan_tier,
            status,
            patterns: parse_metric(headers, "X-ACE-Patterns"),
            patterns_total: parse_metric(headers, "X-ACE-Patterns-Total"),
            projects: parse_metric(headers, "X-ACE-Projects"),
            domains: parse_metric(headers, "X-ACE-Domains"),
            templates: parse_metric(headers, "X-ACE-Templates"),
            api_calls: parse_metric(headers, "X-ACE-API-Calls"),
            traces_today: parse_metric(headers, "X-ACE-Traces"),
            subscription_updated_at: headers
                .get("X-ACE-Subscription-Updated-At")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
        })
    }

    /// Store parsed usage info as the latest snapshot.
    async fn process_usage(&self, usage: UsageInfo) {
        let mut slot = self.last_usage.lock().await;
        *slot = Some(usage);
    }

    /// Build request headers.
    fn build_headers(&self) -> Result<HeaderMap, AceError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.api_token))
                .map_err(|e| AceError::Other(e.to_string()))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&format!("ace-sdk-rust/{}", CORE_VERSION))
                .map_err(|e| AceError::Other(e.to_string()))?,
        );
        headers.insert(
            "X-ACE-Project",
            HeaderValue::from_str(&self.config.project_id)
                .map_err(|e| AceError::Other(e.to_string()))?,
        );

        // Add org header for user tokens
        if let Some(ref org_id) = self.config.default_org_id {
            headers.insert(
                "X-ACE-Org",
                HeaderValue::from_str(org_id).map_err(|e| AceError::Other(e.to_string()))?,
            );
        }

        // Add custom headers
        for (key, value) in &self.custom_headers {
            if let (Ok(name), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(name, val);
            }
        }

        Ok(headers)
    }

    /// Make an authenticated request to the ACE API.
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        method: reqwest::Method,
        body: Option<&serde_json::Value>,
    ) -> Result<T, AceError> {
        let url = format!("{}{}", self.config.server_url, endpoint);
        let headers = self.build_headers()?;

        self.logger.debug(&format!("-> {} {}", method, endpoint));

        let mut request = self.http_client.request(method, &url).headers(headers);

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status().as_u16();

        // Parse `X-ACE-*` usage headers on every response (parity with TS, Kotlin, Go, Python).
        if let Some(usage) = Self::parse_usage_headers(response.headers()) {
            self.process_usage(usage).await;
        }

        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(AceError::from_http_response(status, &body_text));
        }

        let result = response.json::<T>().await?;
        Ok(result)
    }

    /// Get structured playbook (3-tier cache: RAM -> SQLite -> Server).
    pub async fn get_playbook(
        &self,
        force_refresh: bool,
    ) -> Result<PlaybookResponseWithMetadata, AceError> {
        // 1. Check RAM cache
        if !force_refresh {
            let cache = self.memory_cache.lock().await;
            if let Some(ref playbook) = *cache {
                self.logger.debug("Cache hit (RAM)");
                return Ok(PlaybookResponseWithMetadata {
                    playbook: playbook.clone(),
                    total_bullets: count_bullets(playbook),
                    metadata: None,
                });
            }
        }

        // 2. Check SQLite cache
        if !force_refresh {
            if let Some(ref cache) = self.local_cache {
                if !cache.needs_sync() {
                    if let Some(playbook) = cache.get_playbook() {
                        self.logger.debug("Cache hit (SQLite)");
                        let mut mem = self.memory_cache.lock().await;
                        *mem = Some(playbook.clone());
                        return Ok(PlaybookResponseWithMetadata {
                            playbook: playbook.clone(),
                            total_bullets: count_bullets(&playbook),
                            metadata: None,
                        });
                    }
                }
            }
        }

        // 3. Fetch from server
        self.logger.info("Fetching playbook from server...");
        let result: PlaybookResponseWithMetadata = self
            .request(
                "/playbook?include_metadata=true",
                reqwest::Method::GET,
                None,
            )
            .await?;

        // Update caches
        let mut mem = self.memory_cache.lock().await;
        *mem = Some(result.playbook.clone());
        if let Some(ref cache) = self.local_cache {
            cache.save_playbook(&result.playbook);
        }

        Ok(result)
    }

    /// Semantic search for relevant patterns.
    ///
    /// `threshold` is `Option<f64>` — when `None`, the field is omitted from
    /// the request body and the server default takes effect (CONTRACT §2).
    pub async fn search_patterns(
        &self,
        query: &str,
        threshold: Option<f64>,
        top_k: Option<u32>,
        section: Option<&str>,
        agent_type: Option<&str>,
    ) -> Result<SearchResponseWithMetadata, AceError> {
        let now = chrono::Utc::now().to_rfc3339();
        let temp_id = format!("temp_search_{}", chrono::Utc::now().timestamp_millis());

        let mut body = serde_json::json!({
            "pattern": {
                "id": temp_id,
                "content": query,
                "confidence": 0.8,
                "created_at": now,
                "section": section.unwrap_or("general")
            }
        });

        if let Some(t) = threshold {
            body["threshold"] = serde_json::json!(t);
        }

        if let Some(k) = top_k {
            body["top_k"] = serde_json::json!(k);
        }

        if let Some(at) = agent_type {
            body["agent_type"] = serde_json::json!(at);
        }

        self.request(
            "/patterns/search?include_metadata=true",
            reqwest::Method::POST,
            Some(&body),
        )
        .await
    }

    /// ACE 1.5 native pattern search — returns `SearchResponse15` with typed
    /// reward fields, `match_factors`, and top-level `retrieval_id` (F-080).
    ///
    /// Populate path (CONTRACT §5c — three best-effort steps each wrapped
    /// so a cache failure NEVER breaks search):
    ///
    /// 1. **Upsert patterns** — each returned pattern is written to the local
    ///    `GraphCache` with a 7-day TTL so `get_pattern(id)` works for cached
    ///    neighbors (token-efficient id-only path).
    ///
    /// 2. **Throttled edge refresh** — calls `GET /patterns/graph?min_weight=5`
    ///    at most once per `graph_edges_throttle_ms` window (default 1 h),
    ///    gated on `sync_state('graph_edges_synced_at')`.  Fire-and-forget:
    ///    the async task is spawned and the method returns immediately after
    ///    writing the optimistic marker.
    ///
    /// 3. **Expand neighbors** — runs the 2-hop neighbor query and attaches
    ///    results as `SearchResponse15::expanded`.  Deduped against primary.
    ///    Disabled when `SearchRequest15::expand_neighbors = Some(false)`.
    ///
    /// The three optional body fields (`task_intent`, `exploration_enabled`,
    /// `exploration_rate`) are omitted when `None` — server default preserved.
    pub async fn search_patterns15(
        &self,
        request: SearchRequest15,
    ) -> Result<SearchResponse15, AceError> {
        let expand_neighbors = request.expand_neighbors.unwrap_or(true);
        let body = serde_json::to_value(&request)?;
        let mut response: SearchResponse15 = self
            .request(
                "/patterns/search?include_metadata=true",
                reqwest::Method::POST,
                Some(&body),
            )
            .await?;

        if let Some(ref gc_mutex) = self.graph_cache {
            // ── Step 1: Upsert returned patterns (CONTRACT §5c) ──────────────
            // Best-effort: swallow all errors.
            // Determine whether to fire the throttled refresh BEFORE
            // dropping the lock so we can stamp the marker atomically.
            let (needs_refresh, expanded_neighbors) = {
                match gc_mutex.lock() {
                    Err(_) => (false, vec![]),
                    Ok(gc) => {
                        let now_ms = chrono::Utc::now().timestamp_millis();

                        // Upsert all returned patterns.
                        for pattern in &response.similar_patterns {
                            if let Ok(payload_json) = serde_json::to_string(pattern) {
                                let reward = pattern.cumulative_v15_reward.unwrap_or(0.0);
                                let _ = gc.upsert_pattern_at(
                                    &pattern.id,
                                    &payload_json,
                                    reward,
                                    now_ms,
                                );
                            }
                        }

                        // ── Step 2: Throttle gate check ───────────────────────
                        let needs = match gc.get_sync_state("graph_edges_synced_at") {
                            Ok(Some(ref ts)) => chrono::DateTime::parse_from_rfc3339(ts)
                                .map(|last| {
                                    let elapsed = chrono::Utc::now()
                                        .signed_duration_since(last)
                                        .num_milliseconds()
                                        as u64;
                                    elapsed >= self.graph_edges_throttle_ms
                                })
                                .unwrap_or(true),
                            Ok(None) => true,
                            Err(_) => true,
                        };

                        if needs {
                            // Optimistically stamp now (inside lock = atomic).
                            let marker = chrono::Utc::now().to_rfc3339();
                            gc.set_sync_state("graph_edges_synced_at", &marker);
                        }

                        // ── Step 3: Expand neighbors ──────────────────────────
                        let neighbors_out = if expand_neighbors {
                            let primary_ids: std::collections::HashSet<String> = response
                                .similar_patterns
                                .iter()
                                .map(|p| p.id.clone())
                                .collect();

                            let mut acc: Vec<crate::types::ExpandedNeighborEntry> = Vec::new();
                            for pattern in &response.similar_patterns {
                                if let Ok(nbrs) = gc.neighbors(&pattern.id, 2, 5) {
                                    for (nb_id, _payload, reward) in nbrs {
                                        if primary_ids.contains(&nb_id) {
                                            continue;
                                        }
                                        if acc.iter().any(|e| e.pattern_id == nb_id) {
                                            continue;
                                        }
                                        acc.push(crate::types::ExpandedNeighborEntry {
                                            pattern_id: nb_id,
                                            cumulative_reward: reward,
                                            cached: true,
                                            payload_json: None,
                                        });
                                    }
                                }
                            }
                            acc
                        } else {
                            vec![]
                        };

                        // MutexGuard dropped here when `gc` goes out of scope
                        (needs, neighbors_out)
                    }
                }
            };

            // ── Step 2 continued: spawn fire-and-forget refresh ───────────────
            // Lock is fully released above. Use the Arc-level wrapper so no
            // MutexGuard is alive across the HTTP await inside the task.
            if needs_refresh {
                let http = self.http_client.clone();
                let base_url = self.config.server_url.clone();
                let gc_arc = gc_mutex.clone();
                let headers = self.build_headers().unwrap_or_default();

                tokio::spawn(async move {
                    let _ = GraphCache::refresh_from_server_arc(
                        gc_arc,
                        http,
                        base_url,
                        headers,
                        Some(5),
                        None,
                    )
                    .await;
                });
            }

            // Attach expanded field to response.
            if !expanded_neighbors.is_empty() {
                response.expanded = Some(expanded_neighbors);
            }
        }

        Ok(response)
    }

    /// Get a reference to the ACE 1.5 graph cache (if initialized).
    ///
    /// Returns `None` when the cache could not be created (e.g. filesystem error).
    pub fn get_graph_cache(&self) -> Option<&Arc<std::sync::Mutex<GraphCache>>> {
        self.graph_cache.as_ref()
    }

    /// Get top patterns by helpful score.
    pub async fn get_top_patterns(
        &self,
        section: Option<&str>,
        limit: Option<u32>,
        min_helpful: Option<i32>,
    ) -> Result<Vec<PlaybookBullet>, AceError> {
        let mut params = Vec::new();
        if let Some(s) = section {
            params.push(format!("section={}", s));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(m) = min_helpful {
            params.push(format!("min_helpful={}", m));
        }

        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        let result: serde_json::Value = self
            .request(
                &format!("/patterns/top{}", query),
                reqwest::Method::GET,
                None,
            )
            .await?;

        let patterns: Vec<PlaybookBullet> =
            serde_json::from_value(result["top_patterns"].clone()).unwrap_or_default();
        Ok(patterns)
    }

    /// Get playbook analytics.
    pub async fn get_analytics(&self) -> Result<PlaybookStats, AceError> {
        self.request("/analytics", reqwest::Method::GET, None).await
    }

    /// Store execution trace for server-side analysis.
    pub async fn store_execution_trace(
        &self,
        trace: &ExecutionTrace,
    ) -> Result<LearningResponse, AceError> {
        let body = serde_json::to_value(trace)?;
        self.request("/traces", reqwest::Method::POST, Some(&body))
            .await
    }

    /// Apply a delta operation (ADD/UPDATE/DELETE).
    pub async fn apply_delta(&self, operation: &DeltaOperation) -> Result<(), AceError> {
        let body = serde_json::to_value(operation)?;
        let _: serde_json::Value = self
            .request("/delta", reqwest::Method::POST, Some(&body))
            .await?;
        self.invalidate_cache().await;
        Ok(())
    }

    /// Clear entire playbook.
    pub async fn clear_playbook(&self) -> Result<(), AceError> {
        let _: serde_json::Value = self
            .request("/patterns?confirm=true", reqwest::Method::DELETE, None)
            .await?;
        self.invalidate_cache().await;
        Ok(())
    }

    /// Bootstrap playbook from extracted code blocks.
    pub async fn bootstrap(
        &self,
        mode: &BootstrapMode,
        code_blocks: Vec<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<BootstrapResponse, AceError> {
        let body = serde_json::json!({
            "mode": mode,
            "code_blocks": code_blocks,
            "metadata": metadata.unwrap_or(serde_json::json!({}))
        });

        self.request("/bootstrap", reqwest::Method::POST, Some(&body))
            .await
    }

    /// Get server configuration with caching.
    pub async fn get_config(&self, use_cache: bool) -> Result<ServerConfig, AceError> {
        if use_cache {
            let cache = self.config_cache.lock().await;
            if let Some((ref config, instant)) = *cache {
                if instant.elapsed() < std::time::Duration::from_secs(3600) {
                    self.logger.debug("Config cache hit (RAM)");
                    return Ok(config.clone());
                }
            }
        }

        self.logger.info("Fetching config from server...");
        let config: ServerConfig = self
            .request("/api/v1/config", reqwest::Method::GET, None)
            .await?;

        let mut cache = self.config_cache.lock().await;
        *cache = Some((config.clone(), std::time::Instant::now()));

        Ok(config)
    }

    /// Verify API token and fetch organization info.
    pub async fn verify_token(&self) -> Result<serde_json::Value, AceError> {
        self.request("/api/v1/config/verify", reqwest::Method::GET, None)
            .await
    }

    /// Invalidate all caches (force refresh on next call).
    pub async fn invalidate_cache(&self) {
        let mut mem = self.memory_cache.lock().await;
        *mem = None;
        self.logger.debug("Cache invalidated");
    }

    /// Save structured playbook to server.
    pub async fn save_playbook(&self, playbook: &StructuredPlaybook) -> Result<(), AceError> {
        let all_bullets: Vec<&PlaybookBullet> = playbook
            .strategies_and_hard_rules
            .iter()
            .chain(playbook.useful_code_snippets.iter())
            .chain(playbook.troubleshooting_and_pitfalls.iter())
            .chain(playbook.apis_to_use.iter())
            .collect();

        let body = serde_json::json!({ "patterns": all_bullets });
        let _: serde_json::Value = self
            .request("/patterns", reqwest::Method::POST, Some(&body))
            .await?;
        Ok(())
    }

    /// Apply multiple delta operations in batch.
    pub async fn apply_deltas(&self, operations: &[DeltaOperation]) -> Result<(), AceError> {
        for op in operations {
            self.apply_delta(op).await?;
        }
        Ok(())
    }

    /// Compute embeddings for texts.
    pub async fn compute_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, AceError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let body = serde_json::json!({ "texts": texts });
        let result: serde_json::Value = self
            .request("/embeddings", reqwest::Method::POST, Some(&body))
            .await?;

        let embeddings: Vec<Vec<f64>> =
            serde_json::from_value(result["embeddings"].clone()).unwrap_or_default();
        Ok(embeddings)
    }

    /// Get usage history with time-windowed buckets.
    pub async fn get_usage_history(
        &self,
        window: Option<&UsageWindow>,
        project_id: Option<&str>,
    ) -> Result<UsageHistoryResponse, AceError> {
        let mut params = Vec::new();
        if let Some(w) = window {
            params.push(format!("window={}", w));
        }
        if let Some(pid) = project_id {
            params.push(format!("project_id={}", pid));
        }
        let query = if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        };

        self.request(
            &format!("/api/v1/usage/history{}", query),
            reqwest::Method::GET,
            None,
        )
        .await
    }

    /// Fetch hourly/daily usage history for a specific organization.
    ///
    /// Issues `GET /api/v1/usage/history?window={window}[&project_id={id}]`
    /// with the `X-ACE-Org` header set to `org_id`. Windows of 12h or
    /// less return hourly buckets; larger windows return daily buckets.
    ///
    /// # Arguments
    /// * `org_id` - Organization ID to scope the query to (sent via `X-ACE-Org`).
    /// * `window` - Time window for the query.
    /// * `project_id` - Optional project filter.
    pub async fn get_org_usage_hourly(
        &self,
        org_id: &str,
        window: UsageWindow,
        project_id: Option<&str>,
    ) -> Result<UsageHistoryResponse, AceError> {
        let mut url = format!(
            "{}/api/v1/usage/history?window={}",
            self.config.server_url, window
        );
        if let Some(pid) = project_id {
            url.push_str(&format!("&project_id={}", pid));
        }

        let mut headers = self.build_headers()?;
        headers.insert(
            "X-ACE-Org",
            reqwest::header::HeaderValue::from_str(org_id)
                .map_err(|e| AceError::Other(e.to_string()))?,
        );

        self.logger
            .debug(&format!("-> GET /api/v1/usage/history?window={}", window));

        let response = self.http_client.get(&url).headers(headers).send().await?;
        let status = response.status().as_u16();

        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(AceError::from_http_response(status, &body_text));
        }

        let result = response.json::<UsageHistoryResponse>().await?;
        Ok(result)
    }

    /// Batch retrieve patterns by ID.
    pub async fn batch_get_patterns(
        &self,
        pattern_ids: &[String],
    ) -> Result<BatchGetPatternsResponse, AceError> {
        if pattern_ids.is_empty() {
            return Ok(BatchGetPatternsResponse {
                patterns: Vec::new(),
                found_count: 0,
                not_found: Vec::new(),
            });
        }

        const MAX_BATCH: usize = 50;
        let mut all_patterns = Vec::new();
        let mut total_found = 0u32;
        let mut all_not_found = Vec::new();

        for chunk in pattern_ids.chunks(MAX_BATCH) {
            let body = serde_json::json!({ "pattern_ids": chunk });
            let result: BatchGetPatternsResponse = self
                .request("/patterns/batch", reqwest::Method::POST, Some(&body))
                .await?;
            all_patterns.extend(result.patterns);
            total_found += result.found_count;
            all_not_found.extend(result.not_found);
        }

        Ok(BatchGetPatternsResponse {
            patterns: all_patterns,
            found_count: total_found,
            not_found: all_not_found,
        })
    }

    /// Update server configuration.
    pub async fn update_config(
        &self,
        settings: serde_json::Value,
        scope: &str,
    ) -> Result<ServerConfig, AceError> {
        let config: ServerConfig = self
            .request(
                &format!("/api/v1/config?scope={}", scope),
                reqwest::Method::PUT,
                Some(&settings),
            )
            .await?;

        // Invalidate config cache
        let mut cache = self.config_cache.lock().await;
        *cache = None;

        Ok(config)
    }

    /// Reset server configuration to defaults.
    pub async fn reset_config(&self, scope: &str) -> Result<serde_json::Value, AceError> {
        let result: serde_json::Value = self
            .request(
                &format!("/api/v1/config/reset?scope={}", scope),
                reqwest::Method::POST,
                None,
            )
            .await?;

        // Invalidate config cache
        let mut cache = self.config_cache.lock().await;
        *cache = None;

        Ok(result)
    }

    /// Clear config cache.
    pub async fn clear_config_cache(&self) {
        let mut cache = self.config_cache.lock().await;
        *cache = None;
    }

    /// Initialize playbook from git repository (server-side).
    pub async fn initialize_from_repo(
        &self,
        repo_path: &str,
        commit_limit: u32,
        days_back: u32,
        merge_with_existing: bool,
    ) -> Result<serde_json::Value, AceError> {
        let body = serde_json::json!({
            "repo_path": repo_path,
            "commit_limit": commit_limit,
            "days_back": days_back,
            "merge_with_existing": merge_with_existing
        });
        self.request("/init", reqwest::Method::POST, Some(&body))
            .await
    }

    /// Get playbook status/statistics (alias for get_analytics).
    pub async fn get_status(&self) -> Result<PlaybookStats, AceError> {
        self.get_analytics().await
    }

    /// Store execution trace with SSE streaming (with fallback to /traces).
    ///
    /// Sends trace to `/traces/stream` endpoint for real-time learning events.
    /// Falls back to `store_execution_trace` on error when `fallback_on_error` is true.
    pub async fn store_execution_trace_stream(
        &self,
        trace: &ExecutionTrace,
        on_event: impl Fn(&LearningStreamEvent),
        fallback_on_error: bool,
    ) -> Result<LearningResponse, AceError> {
        let url = format!("{}/traces/stream", self.config.server_url);
        let headers = self.build_headers()?;

        self.logger.debug("-> POST /traces/stream (SSE)");

        let body = serde_json::to_value(trace)?;

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .header("Accept", "text/event-stream")
            .json(&body)
            .send()
            .await;

        let response = match response {
            Ok(r) => r,
            Err(e) => {
                if fallback_on_error {
                    self.logger
                        .info("SSE stream failed, falling back to /traces endpoint");
                    return self.store_execution_trace(trace).await;
                }
                return Err(AceError::Network(e));
            }
        };

        let status = response.status().as_u16();
        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();

            // Check for quota exceeded - return graceful result instead of error
            if status == 429 {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body_text) {
                    if data.get("error").and_then(|v| v.as_str()) == Some("quota_exceeded") {
                        return Ok(LearningResponse {
                            stored: false,
                            task: None,
                            timestamp: None,
                            analysis_performed: false,
                            server_learning_enabled: None,
                            learning_statistics: None,
                            learning_queued: Some(false),
                            quota_exceeded: Some(true),
                            quota_error_code: data
                                .get("code")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            message: data
                                .get("message")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        });
                    }
                }
            }

            if fallback_on_error {
                self.logger
                    .info("SSE stream failed, falling back to /traces endpoint");
                return self.store_execution_trace(trace).await;
            }
            return Err(AceError::from_http_response(status, &body_text));
        }

        // Parse SSE events from response body
        let body_text = response.text().await.unwrap_or_default();
        let mut last_response: Option<LearningResponse> = None;

        for line in body_text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(event) = serde_json::from_str::<LearningStreamEvent>(data) {
                    on_event(&event);

                    // Extract final result from 'done' stage
                    if event.stage == crate::types::LearningStreamStage::Done {
                        if let Some(ref event_data) = event.data {
                            if let Ok(lr) =
                                serde_json::from_value::<LearningResponse>(event_data.clone())
                            {
                                last_response = Some(lr);
                            }
                        }
                    }
                }
            }
        }

        // Return last response or build one from trace
        Ok(last_response.unwrap_or(LearningResponse {
            stored: true,
            task: Some(trace.task.clone()),
            timestamp: Some(trace.timestamp.clone()),
            analysis_performed: true,
            server_learning_enabled: None,
            learning_statistics: None,
            learning_queued: None,
            quota_exceeded: None,
            quota_error_code: None,
            message: None,
        }))
    }

    /// List execution traces (spec-21, root prefix `/traces`).
    ///
    /// Filters via query string: project_id (required), start, end, status,
    /// agent_type, session_id, git_branch, limit, cursor.
    /// Sends Authorization, X-ACE-Org, X-ACE-Project headers.
    /// Maps 400/401/403/422/503 to typed errors.
    pub async fn list_traces(&self, filters: TraceFilters) -> Result<TraceListResponse, AceError> {
        let mut params: Vec<(String, String)> = Vec::new();
        params.push(("project_id".to_string(), filters.project_id.clone()));
        if let Some(v) = filters.start {
            params.push(("start".to_string(), v));
        }
        if let Some(v) = filters.end {
            params.push(("end".to_string(), v));
        }
        if let Some(v) = filters.status {
            params.push(("status".to_string(), v));
        }
        if let Some(v) = filters.agent_type {
            params.push(("agent_type".to_string(), v));
        }
        if let Some(v) = filters.session_id {
            params.push(("session_id".to_string(), v));
        }
        if let Some(v) = filters.git_branch {
            params.push(("git_branch".to_string(), v));
        }
        if let Some(v) = filters.limit {
            params.push(("limit".to_string(), v.to_string()));
        }
        if let Some(v) = filters.cursor {
            params.push(("cursor".to_string(), v));
        }

        let qs = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}/traces?{}", self.config.server_url, qs);

        let mut headers = self.build_headers()?;
        // Override X-ACE-Project to the filter's project_id (call-site truth)
        headers.insert(
            "X-ACE-Project",
            reqwest::header::HeaderValue::from_str(&filters.project_id)
                .map_err(|e| AceError::Other(e.to_string()))?,
        );

        self.logger.debug("-> GET /traces");
        let response = self.http_client.get(&url).headers(headers).send().await?;
        let status = response.status().as_u16();

        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(AceError::from_http_response(status, &body_text));
        }

        let result = response.json::<TraceListResponse>().await?;
        Ok(result)
    }

    /// Get full trace detail (spec-21, root prefix `/traces/{trace_id}`).
    ///
    /// Sends Authorization, X-ACE-Org, X-ACE-Project headers + project_id query.
    /// Maps 410 -> `AceError::TraceUnavailable` (pre-migration ghost trace,
    /// safe to ignore without retry).
    pub async fn get_trace(
        &self,
        trace_id: &str,
        project_id: &str,
    ) -> Result<TraceDetail, AceError> {
        let url = format!(
            "{}/traces/{}?project_id={}",
            self.config.server_url,
            urlencoding::encode(trace_id),
            urlencoding::encode(project_id),
        );

        let mut headers = self.build_headers()?;
        headers.insert(
            "X-ACE-Project",
            reqwest::header::HeaderValue::from_str(project_id)
                .map_err(|e| AceError::Other(e.to_string()))?,
        );

        self.logger.debug(&format!("-> GET /traces/{}", trace_id));
        let response = self.http_client.get(&url).headers(headers).send().await?;
        let status = response.status().as_u16();

        if status == 410 {
            let body_text = response.text().await.unwrap_or_default();
            let msg = serde_json::from_str::<serde_json::Value>(&body_text)
                .ok()
                .and_then(|v| {
                    v.get("message")
                        .and_then(|m| m.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| body_text.clone());
            return Err(AceError::TraceUnavailable(msg));
        }

        if status >= 400 {
            let body_text = response.text().await.unwrap_or_default();
            return Err(AceError::from_http_response(status, &body_text));
        }

        let result = response.json::<TraceDetail>().await?;
        Ok(result)
    }

    /// Get reference to the local cache service.
    pub fn get_local_cache(&self) -> Option<&LocalCacheService> {
        self.local_cache.as_ref()
    }

    /// Check if auto-refresh is enabled.
    pub fn is_auto_refresh(&self) -> bool {
        self.auto_refresh
    }
}

/// Count total bullets across all playbook sections.
fn count_bullets(playbook: &StructuredPlaybook) -> u32 {
    (playbook.strategies_and_hard_rules.len()
        + playbook.useful_code_snippets.len()
        + playbook.troubleshooting_and_pitfalls.len()
        + playbook.apis_to_use.len()) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_bullets() {
        let playbook = StructuredPlaybook::default();
        assert_eq!(count_bullets(&playbook), 0);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = AceConfig {
            server_url: "https://test.example.com".to_string(),
            api_token: "ace_user_test123".to_string(),
            project_id: "test-project".to_string(),
            ..Default::default()
        };

        let client = AceClient::new(config, AceClientOptions::default()).unwrap();
        assert!(client.is_auto_refresh());
    }

    #[tokio::test]
    async fn test_client_no_auto_refresh_for_org_tokens() {
        let config = AceConfig {
            server_url: "https://test.example.com".to_string(),
            api_token: "ace_12345678test".to_string(),
            project_id: "test-project".to_string(),
            ..Default::default()
        };

        let client = AceClient::new(config, AceClientOptions::default()).unwrap();
        assert!(!client.is_auto_refresh());
    }
}
