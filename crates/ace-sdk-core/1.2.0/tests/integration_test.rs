//! Comprehensive integration tests for ace-sdk-core.
//!
//! 120+ tests covering all modules: types, client, auth, config, utils,
//! services, and errors.

use ace_sdk_core::*;
use std::collections::HashMap;

// =============================================================================
// Types: JSON Roundtrip Tests
// =============================================================================

#[test]
fn test_playbook_bullet_json_roundtrip() {
    let bullet = PlaybookBullet {
        id: "ctx-1234-abc".to_string(),
        section: BulletSection::StrategiesAndHardRules,
        content: "Always handle errors explicitly".to_string(),
        domain: Some("error-handling".to_string()),
        concrete_domain: Some("src/errors/".to_string()),
        helpful: 10.0,
        harmful: 1.0,
        confidence: 0.85,
        observations: 20.0,
        evidence: vec!["src/main.rs:42".to_string()],
        created_at: "2025-01-01T00:00:00Z".to_string(),
        last_used: Some("2025-04-01T00:00:00Z".to_string()),
        root_cause: String::new(),
        error_context: String::new(),
    };

    let json = serde_json::to_string(&bullet).unwrap();
    let deserialized: PlaybookBullet = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "ctx-1234-abc");
    // ACE 1.5: helpful/harmful are deprecated read-only fields — they are NOT
    // serialized into write/POST bodies (skip_serializing). The round-trip via
    // serialize→deserialize therefore produces the default value (0.0).
    // Callers must read helpful/harmful from server responses, never from local state.
    assert_eq!(deserialized.helpful, 0.0); // skip_serializing: not in write body
    assert_eq!(deserialized.confidence, 0.85);
    assert_eq!(deserialized.evidence.len(), 1);
    assert_eq!(deserialized.domain, Some("error-handling".to_string()));
}

#[test]
fn test_playbook_bullet_minimal_json() {
    // Test with minimal fields (all optional fields omitted in JSON)
    let json = r#"{
        "id": "ctx-min-1",
        "section": "apis_to_use",
        "content": "Use reqwest for HTTP",
        "created_at": "2025-01-01T00:00:00Z",
        "last_used": "2025-01-01T00:00:00Z"
    }"#;
    let bullet: PlaybookBullet = serde_json::from_str(json).unwrap();
    assert_eq!(bullet.id, "ctx-min-1");
    assert_eq!(bullet.section, BulletSection::ApisToUse);
    assert_eq!(bullet.helpful, 0.0); // default
    assert_eq!(bullet.harmful, 0.0); // default
    assert_eq!(bullet.confidence, 0.0); // default
    assert!(bullet.domain.is_none());
    assert!(bullet.evidence.is_empty());
}

#[test]
fn test_structured_playbook_json_roundtrip() {
    let playbook = StructuredPlaybook {
        strategies_and_hard_rules: vec![PlaybookBullet {
            id: "strat-1".to_string(),
            section: BulletSection::StrategiesAndHardRules,
            content: "Use Result<T, E>".to_string(),
            domain: None,
            concrete_domain: None,
            helpful: 5.0,
            harmful: 0.0,
            confidence: 0.9,
            observations: 10.0,
            evidence: vec![],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: Some("2025-01-01T00:00:00Z".to_string()),
            root_cause: String::new(),
            error_context: String::new(),
        }],
        useful_code_snippets: vec![],
        troubleshooting_and_pitfalls: vec![],
        apis_to_use: vec![],
    };

    let json = serde_json::to_string(&playbook).unwrap();
    let deserialized: StructuredPlaybook = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.strategies_and_hard_rules.len(), 1);
    assert_eq!(
        deserialized.strategies_and_hard_rules[0].content,
        "Use Result<T, E>"
    );
}

#[test]
fn test_structured_playbook_default() {
    let playbook = StructuredPlaybook::default();
    assert!(playbook.strategies_and_hard_rules.is_empty());
    assert!(playbook.useful_code_snippets.is_empty());
    assert!(playbook.troubleshooting_and_pitfalls.is_empty());
    assert!(playbook.apis_to_use.is_empty());
}

#[test]
fn test_execution_trace_json_roundtrip() {
    let trace = ExecutionTrace {
        task: "Fix authentication bug".to_string(),
        trajectory: vec![serde_json::json!({"step": 1, "action": "read_file"})],
        result: ExecutionResult {
            success: true,
            output: "Bug fixed".to_string(),
            error: None,
            summary: Some("Fixed auth token validation".to_string()),
        },
        playbook_used: vec!["ctx-123".to_string()],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        git: Some(GitContext {
            commit_hash: "abc123".to_string(),
            branch: "main".to_string(),
            files_changed: vec!["src/auth.rs".to_string()],
            commit_message: Some("fix: auth token validation".to_string()),
            timestamp: None,
            author: Some("dev".to_string()),
            author_email: None,
            repository_url: None,
            insertions: Some(10),
            deletions: Some(3),
            parent_commits: None,
        }),
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id: None,
        applied_log_ids: None,
    };

    let json = serde_json::to_string(&trace).unwrap();
    let deserialized: ExecutionTrace = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.task, "Fix authentication bug");
    assert!(deserialized.result.success);
    assert!(deserialized.git.is_some());
    assert_eq!(deserialized.git.unwrap().commit_hash, "abc123");
}

#[test]
fn test_execution_trace_without_git() {
    let trace = ExecutionTrace {
        task: "Simple task".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: false,
            output: String::new(),
            error: Some("Failed".to_string()),
            summary: None,
        },
        playbook_used: vec![],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        git: None,
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id: None,
        applied_log_ids: None,
    };

    let json = serde_json::to_string(&trace).unwrap();
    assert!(!json.contains("\"git\""));
    assert!(!json.contains("\"session_id\""));
    assert!(!json.contains("\"agent_id\""));
    assert!(!json.contains("\"agent_type\""));
    assert!(!json.contains("\"parent_agent_id\""));
}

#[test]
fn test_execution_trace_multi_agent_fields() {
    let trace = ExecutionTrace {
        task: "sub-agent task".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output: "ok".to_string(),
            error: None,
            summary: None,
        },
        playbook_used: vec![],
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        git: None,
        session_id: Some("sess_123".to_string()),
        agent_id: Some("agent_xyz".to_string()),
        agent_type: Some("researcher".to_string()),
        parent_agent_id: Some("agent_root".to_string()),
        retrieval_id: None,
        applied_log_ids: None,
    };

    let json = serde_json::to_string(&trace).unwrap();
    assert!(json.contains("\"session_id\":\"sess_123\""));
    assert!(json.contains("\"agent_id\":\"agent_xyz\""));
    assert!(json.contains("\"agent_type\":\"researcher\""));
    assert!(json.contains("\"parent_agent_id\":\"agent_root\""));

    let deserialized: ExecutionTrace = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.session_id.as_deref(), Some("sess_123"));
    assert_eq!(deserialized.agent_id.as_deref(), Some("agent_xyz"));
    assert_eq!(deserialized.agent_type.as_deref(), Some("researcher"));
    assert_eq!(deserialized.parent_agent_id.as_deref(), Some("agent_root"));
}

#[test]
fn test_ace_config_json_roundtrip() {
    let config = AceConfig {
        server_url: "https://custom.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "prj_123".to_string(),
        cache_ttl_minutes: 60,
        orgs: None,
        verbosity: Some(VerbosityLevel::Detailed),
        auth: None,
        default_org_id: Some("org_abc".to_string()),
        device_id: Some("dev_123".to_string()),
        graph_cache_dir: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: AceConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.server_url, "https://custom.example.com");
    assert_eq!(deserialized.verbosity, Some(VerbosityLevel::Detailed));
    assert_eq!(deserialized.default_org_id, Some("org_abc".to_string()));
}

#[test]
fn test_ace_config_default() {
    let config = AceConfig::default();
    assert_eq!(config.server_url, "https://ace-api.code-engine.app");
    assert_eq!(config.cache_ttl_minutes, 120);
    assert!(config.api_token.is_empty());
    assert!(config.project_id.is_empty());
    assert!(config.orgs.is_none());
    assert!(config.verbosity.is_none());
    assert!(config.auth.is_none());
    assert!(config.device_id.is_none());
}

#[test]
fn test_user_auth_json_roundtrip() {
    let auth = UserAuth {
        token: "ace_user_test123".to_string(),
        user_id: "user_clerk_abc".to_string(),
        email: "test@example.com".to_string(),
        organizations: vec![OrgMembership {
            org_id: "org_123".to_string(),
            name: "Test Org".to_string(),
            role: "admin".to_string(),
            created_at: Some("2025-01-01T00:00:00Z".to_string()),
        }],
        authenticated_at: Some("2025-01-01T00:00:00Z".to_string()),
        refresh_token: Some("ace_refresh_xyz".to_string()),
        expires_at: Some("2025-01-02T00:00:00Z".to_string()),
        refresh_expires_at: Some("2025-01-08T00:00:00Z".to_string()),
        absolute_expires_at: Some("2025-01-08T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&auth).unwrap();
    let deserialized: UserAuth = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.token, "ace_user_test123");
    assert_eq!(deserialized.email, "test@example.com");
    assert_eq!(deserialized.organizations.len(), 1);
    assert_eq!(deserialized.organizations[0].role, "admin");
}

#[test]
fn test_token_response_json_roundtrip() {
    let response = TokenResponse {
        access_token: "ace_user_new".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: Some(3600),
        user: Some(TokenUser {
            user_id: "user_1".to_string(),
            email: "user@example.com".to_string(),
            name: Some("Test User".to_string()),
            image_url: None,
        }),
        user_id: None,
        email: None,
        name: None,
        image_url: None,
        organizations: vec![],
        refresh_token: Some("ace_refresh_new".to_string()),
        refresh_expires_in: Some(604800),
        token_expires_at: None,
        refresh_expires_at: None,
        absolute_expires_at: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: TokenResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.access_token, "ace_user_new");
    assert_eq!(deserialized.expires_in, Some(3600));
}

#[test]
fn test_device_code_response_json() {
    let json = r#"{
        "device_code": "dc_abc123",
        "user_code": "ABCD-1234",
        "verification_uri": "https://ace.code-engine.app/device",
        "verification_uri_complete": "https://ace.code-engine.app/device?code=ABCD-1234",
        "expires_in": 900,
        "interval": 5
    }"#;
    let response: DeviceCodeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.user_code, "ABCD-1234");
    assert_eq!(response.expires_in, 900);
    assert_eq!(response.interval, 5);
}

#[test]
fn test_server_config_json_roundtrip() {
    let config = ServerConfig {
        dedup_similarity_threshold: Some(0.85),
        dedup_enabled: Some(true),
        constitution_threshold: Some(0.75),
        search_threshold: Some(0.7),
        pruning_threshold: Some(0.3),
        max_playbook_tokens: Some(50000),
        token_budget_enforcement: Some(true),
        max_batch_size: Some(50),
        auto_learning_enabled: Some(true),
        reflector_enabled: Some(true),
        curator_enabled: Some(true),
        search_top_k: Some(10),
        runtime_settings: Some(AceRuntimeSettings {
            search_top_k: Some(15),
            search_threshold: Some(0.8),
            learning_enabled: Some(true),
            learning_min_tokens: Some(100),
            learning_min_confidence: Some(0.3),
            summarization_style: Some("detailed".to_string()),
            summarization_max_tokens: Some(1000),
            pattern_min_helpful: Some(0),
            pattern_default_section: None,
            bootstrap_default_mode: Some("hybrid".to_string()),
            bootstrap_default_thoroughness: Some("medium".to_string()),
        }),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.dedup_enabled, Some(true));
    assert_eq!(deserialized.search_top_k, Some(10));
    assert!(deserialized.runtime_settings.is_some());
}

#[test]
fn test_server_config_default() {
    let config = ServerConfig::default();
    assert!(config.dedup_enabled.is_none());
    assert!(config.search_threshold.is_none());
    assert!(config.runtime_settings.is_none());
}

#[test]
fn test_usage_info_construction() {
    let info = UsageInfo {
        plan: "team/pro".to_string(),
        subscription_type: SubscriptionType::Team,
        plan_tier: PlanTier::Pro,
        status: SubscriptionStatus::Active,
        patterns: UsageMetric {
            used: 50,
            limit: 100,
        },
        patterns_total: UsageMetric {
            used: 200,
            limit: 500,
        },
        projects: UsageMetric { used: 3, limit: 10 },
        domains: UsageMetric { used: 0, limit: 5 },
        templates: UsageMetric { used: 0, limit: 10 },
        api_calls: UsageMetric {
            used: 1000,
            limit: 10000,
        },
        traces_today: UsageMetric { used: 5, limit: 50 },
        subscription_updated_at: Some("2025-01-01T00:00:00Z".to_string()),
    };
    assert_eq!(info.plan, "team/pro");
    assert_eq!(info.subscription_type, SubscriptionType::Team);
}

#[test]
fn test_learning_response_json_roundtrip() {
    let response = LearningResponse {
        stored: true,
        task: Some("Fix bug".to_string()),
        timestamp: Some("2025-01-01T00:00:00Z".to_string()),
        analysis_performed: true,
        server_learning_enabled: Some(true),
        learning_statistics: Some(LearningStatistics {
            patterns_created: 3,
            patterns_updated: 2,
            patterns_pruned: 1,
            patterns_deduplicated: 0,
            by_section: HashMap::new(),
            average_confidence: 0.85,
            helpful_delta: 5,
            helpful_count: 5,
            harmful_count: 0,
            analysis_time_seconds: 1.5,
            cumulative_v15_reward_delta: 0.0,
            patterns_rewarded: 0,
            reward_tier: String::new(),
        }),
        learning_queued: None,
        quota_exceeded: None,
        message: None,
        quota_error_code: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: LearningResponse = serde_json::from_str(&json).unwrap();
    assert!(deserialized.stored);
    assert!(deserialized.learning_statistics.is_some());
    assert_eq!(
        deserialized.learning_statistics.unwrap().patterns_created,
        3
    );
}

#[test]
fn test_learning_response_quota_exceeded() {
    let json = r#"{
        "stored": false,
        "analysis_performed": false,
        "quota_exceeded": true,
        "quota_error_code": "TRACES_LIMIT_EXCEEDED",
        "message": "Daily trace limit reached"
    }"#;
    let response: LearningResponse = serde_json::from_str(json).unwrap();
    assert!(!response.stored);
    assert_eq!(response.quota_exceeded, Some(true));
    assert_eq!(
        response.quota_error_code,
        Some("TRACES_LIMIT_EXCEEDED".to_string())
    );
}

#[test]
fn test_bootstrap_response_json_roundtrip() {
    let response = BootstrapResponse {
        success: true,
        blocks_received: 50,
        patterns_extracted: 25,
        compression_percentage: 89.0,
        patterns_after_dedup: Some(20),
        compression_ratio: Some("89%".to_string()),
        by_section: {
            let mut m = HashMap::new();
            m.insert("strategies_and_hard_rules".to_string(), 10);
            m.insert("useful_code_snippets".to_string(), 5);
            m.insert("troubleshooting_and_pitfalls".to_string(), 3);
            m.insert("apis_to_use".to_string(), 2);
            m
        },
        average_confidence: 0.75,
        analysis_time_seconds: 5.2,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: BootstrapResponse = serde_json::from_str(&json).unwrap();
    assert!(deserialized.success);
    assert_eq!(deserialized.patterns_extracted, 25);
}

#[test]
fn test_delta_operation_add_json() {
    let op = DeltaOperation {
        op_type: DeltaOperationType::ADD,
        section: Some(BulletSection::UsefulCodeSnippets),
        content: Some("Use .iter().filter_map()".to_string()),
        bullet_id: None,
        confidence: Some(0.7),
        evidence: Some(vec!["src/utils.rs".to_string()]),
        reason: Some("Common pattern".to_string()),
    };

    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("ADD"));
    assert!(json.contains("useful_code_snippets"));
}

#[test]
fn test_delta_operation_update_json() {
    let op = DeltaOperation {
        op_type: DeltaOperationType::UPDATE,
        section: None,
        content: None,
        bullet_id: Some("ctx-123".to_string()),
        confidence: None,
        evidence: None,
        reason: Some("Pattern was helpful".to_string()),
    };

    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("UPDATE"));
    assert!(json.contains("ctx-123"));
}

#[test]
fn test_delta_operation_delete_json() {
    let op = DeltaOperation {
        op_type: DeltaOperationType::DELETE,
        section: None,
        content: None,
        bullet_id: Some("ctx-old-456".to_string()),
        confidence: None,
        evidence: None,
        reason: Some("Outdated pattern".to_string()),
    };

    let json = serde_json::to_string(&op).unwrap();
    assert!(json.contains("DELETE"));
}

#[test]
fn test_token_metadata_json() {
    let metadata = TokenMetadata {
        tokens_in_response: 500,
        tokens_saved_vs_full_playbook: Some(4500),
        efficiency_gain: Some("90%".to_string()),
        full_playbook_size: Some(5000),
        cache_tier: Some("server".to_string()),
        latency_ms: Some(150),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: TokenMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.tokens_in_response, 500);
    assert_eq!(deserialized.efficiency_gain, Some("90%".to_string()));
}

#[test]
fn test_search_response_json_roundtrip() {
    let response = SearchResponseWithMetadata {
        similar_patterns: vec![],
        count: 0,
        threshold: 0.75,
        top_k: Some(10),
        domains_summary: Some(DomainsSummary {
            abstract_domains: vec!["auth".to_string()],
            concrete: vec!["src/auth/".to_string()],
        }),
        metadata: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: SearchResponseWithMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.threshold, 0.75);
    assert!(deserialized.domains_summary.is_some());
}

#[test]
fn test_playbook_response_json_roundtrip() {
    let response = PlaybookResponseWithMetadata {
        playbook: StructuredPlaybook::default(),
        total_bullets: 0,
        metadata: Some(TokenMetadata {
            tokens_in_response: 100,
            tokens_saved_vs_full_playbook: None,
            efficiency_gain: None,
            full_playbook_size: None,
            cache_tier: None,
            latency_ms: None,
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: PlaybookResponseWithMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_bullets, 0);
    assert!(deserialized.metadata.is_some());
}

#[test]
fn test_refresh_token_response_json() {
    let json = r#"{
        "access_token": "ace_user_new_token",
        "refresh_token": "ace_refresh_new",
        "expires_in": 3600,
        "refresh_expires_in": 604800,
        "token_expires_at": "2025-01-02T00:00:00Z",
        "refresh_expires_at": "2025-01-08T00:00:00Z",
        "absolute_expires_at": "2025-01-08T00:00:00Z"
    }"#;
    let response: RefreshTokenResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.access_token, "ace_user_new_token");
    assert_eq!(response.expires_in, 3600);
}

#[test]
fn test_current_user_json_roundtrip() {
    let user = CurrentUser {
        user_id: "user_1".to_string(),
        email: "user@example.com".to_string(),
        name: Some("Test User".to_string()),
        image_url: Some("https://example.com/avatar.png".to_string()),
        organizations: vec![OrgMembership {
            org_id: "org_1".to_string(),
            name: "My Org".to_string(),
            role: "member".to_string(),
            created_at: None,
        }],
        default_org_id: Some("org_1".to_string()),
        authenticated_at: Some("2025-01-01T00:00:00Z".to_string()),
    };

    let json = serde_json::to_string(&user).unwrap();
    let deserialized: CurrentUser = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.user_id, "user_1");
    assert_eq!(deserialized.organizations.len(), 1);
}

// =============================================================================
// Types: Token Detection Tests
// =============================================================================

#[test]
fn test_detect_token_type_user() {
    assert_eq!(detect_token_type("ace_user_abc123"), TokenType::User);
    assert_eq!(
        detect_token_type("ace_user_very_long_token_value"),
        TokenType::User
    );
}

#[test]
fn test_detect_token_type_org() {
    assert_eq!(detect_token_type("ace_12345678abc"), TokenType::Org);
    assert_eq!(detect_token_type("ace_orgtoken123"), TokenType::Org);
}

#[test]
fn test_detect_token_type_unknown() {
    assert_eq!(detect_token_type("invalid"), TokenType::Unknown);
    assert_eq!(detect_token_type(""), TokenType::Unknown);
    assert_eq!(detect_token_type("bearer_xyz"), TokenType::Unknown);
}

#[test]
fn test_is_user_token() {
    assert!(is_user_token("ace_user_test"));
    assert!(!is_user_token("ace_org12345"));
    assert!(!is_user_token("invalid"));
    assert!(!is_user_token(""));
}

#[test]
fn test_is_org_token() {
    assert!(is_org_token("ace_org12345"));
    assert!(is_org_token("ace_12345678abc"));
    assert!(!is_org_token("ace_user_test"));
    assert!(!is_org_token("invalid"));
    assert!(!is_org_token(""));
}

// =============================================================================
// Types: Plan Parsing Tests
// =============================================================================

#[test]
fn test_parse_plan_individual_free() {
    let (sub_type, tier) = parse_plan("individual/free");
    assert_eq!(sub_type, SubscriptionType::Individual);
    assert_eq!(tier, PlanTier::Free);
}

#[test]
fn test_parse_plan_individual_basic() {
    let (sub_type, tier) = parse_plan("individual/basic");
    assert_eq!(sub_type, SubscriptionType::Individual);
    assert_eq!(tier, PlanTier::Basic);
}

#[test]
fn test_parse_plan_individual_pro() {
    let (sub_type, tier) = parse_plan("individual/pro");
    assert_eq!(sub_type, SubscriptionType::Individual);
    assert_eq!(tier, PlanTier::Pro);
}

#[test]
fn test_parse_plan_team_free() {
    let (sub_type, tier) = parse_plan("team/free");
    assert_eq!(sub_type, SubscriptionType::Team);
    assert_eq!(tier, PlanTier::Free);
}

#[test]
fn test_parse_plan_team_pro() {
    let (sub_type, tier) = parse_plan("team/pro");
    assert_eq!(sub_type, SubscriptionType::Team);
    assert_eq!(tier, PlanTier::Pro);
}

#[test]
fn test_parse_plan_unknown_defaults() {
    let (sub_type, tier) = parse_plan("unknown/unknown");
    assert_eq!(sub_type, SubscriptionType::Individual);
    assert_eq!(tier, PlanTier::Free);
}

#[test]
fn test_parse_plan_empty() {
    let (sub_type, tier) = parse_plan("");
    assert_eq!(sub_type, SubscriptionType::Individual);
    assert_eq!(tier, PlanTier::Free);
}

// =============================================================================
// Types: Usage Metrics Tests
// =============================================================================

#[test]
fn test_usage_percentage_normal() {
    let m = UsageMetric {
        used: 80,
        limit: 100,
    };
    assert_eq!(get_usage_percentage(&m), 80);
}

#[test]
fn test_usage_percentage_zero_limit() {
    let m = UsageMetric { used: 50, limit: 0 };
    assert_eq!(get_usage_percentage(&m), 0);
}

#[test]
fn test_usage_percentage_negative_limit() {
    let m = UsageMetric {
        used: 50,
        limit: -1,
    };
    assert_eq!(get_usage_percentage(&m), 0);
}

#[test]
fn test_usage_percentage_over_100() {
    let m = UsageMetric {
        used: 150,
        limit: 100,
    };
    assert_eq!(get_usage_percentage(&m), 100); // capped at 100
}

#[test]
fn test_is_near_limit_at_80() {
    let m = UsageMetric {
        used: 80,
        limit: 100,
    };
    assert!(is_near_limit(&m));
}

#[test]
fn test_is_near_limit_below_80() {
    let m = UsageMetric {
        used: 79,
        limit: 100,
    };
    assert!(!is_near_limit(&m));
}

#[test]
fn test_is_over_limit_at_limit() {
    let m = UsageMetric {
        used: 100,
        limit: 100,
    };
    assert!(is_over_limit(&m));
}

#[test]
fn test_is_over_limit_below() {
    let m = UsageMetric {
        used: 99,
        limit: 100,
    };
    assert!(!is_over_limit(&m));
}

#[test]
fn test_is_over_limit_zero_limit() {
    let m = UsageMetric { used: 50, limit: 0 };
    assert!(!is_over_limit(&m));
}

// =============================================================================
// Types: Feature Flags Tests
// =============================================================================

#[test]
fn test_get_features_individual_free() {
    let features = get_features(&SubscriptionType::Individual, &PlanTier::Free);
    assert!(!features.teams);
    assert!(!features.sharing);
    assert!(!features.api_access);
    assert!(!features.priority_support);
}

#[test]
fn test_get_features_individual_pro() {
    let features = get_features(&SubscriptionType::Individual, &PlanTier::Pro);
    assert!(!features.teams);
    assert!(features.sharing);
    assert!(features.api_access);
    assert!(features.priority_support);
}

#[test]
fn test_get_features_team_pro() {
    let features = get_features(&SubscriptionType::Team, &PlanTier::Pro);
    assert!(features.teams);
    assert!(features.sharing);
    assert!(features.api_access);
    assert!(features.priority_support);
}

#[test]
fn test_get_features_team_free() {
    let features = get_features(&SubscriptionType::Team, &PlanTier::Free);
    assert!(features.teams);
    assert!(!features.sharing);
    assert!(!features.api_access);
    assert!(!features.priority_support);
}

#[test]
fn test_get_features_team_basic() {
    let features = get_features(&SubscriptionType::Team, &PlanTier::Basic);
    assert!(features.teams);
    assert!(features.sharing);
    assert!(features.api_access);
    assert!(!features.priority_support);
}

// =============================================================================
// Types: Thoroughness Presets
// =============================================================================

#[test]
fn test_thoroughness_preset_light() {
    let preset = get_thoroughness_preset(&ThoroughnessLevel::Light);
    assert_eq!(preset.max_files, 1000);
    assert_eq!(preset.commit_limit, 100);
    assert_eq!(preset.days_back, 30);
}

#[test]
fn test_thoroughness_preset_medium() {
    let preset = get_thoroughness_preset(&ThoroughnessLevel::Medium);
    assert_eq!(preset.max_files, 5000);
    assert_eq!(preset.commit_limit, 500);
    assert_eq!(preset.days_back, 90);
}

#[test]
fn test_thoroughness_preset_deep() {
    let preset = get_thoroughness_preset(&ThoroughnessLevel::Deep);
    assert_eq!(preset.max_files, -1); // unlimited
    assert_eq!(preset.commit_limit, 1000);
    assert_eq!(preset.days_back, 180);
}

#[test]
fn test_bootstrap_mode_default() {
    assert_eq!(BootstrapMode::default(), BootstrapMode::Hybrid);
}

#[test]
fn test_thoroughness_level_default() {
    assert_eq!(ThoroughnessLevel::default(), ThoroughnessLevel::Medium);
}

#[test]
fn test_verbosity_level_default() {
    assert_eq!(VerbosityLevel::default(), VerbosityLevel::Compact);
}

// =============================================================================
// Types: Default Runtime Settings
// =============================================================================

#[test]
fn test_default_runtime_settings() {
    let settings = default_runtime_settings();
    assert_eq!(settings.search_top_k, Some(10));
    assert_eq!(settings.search_threshold, Some(0.75));
    assert_eq!(settings.learning_enabled, Some(true));
    assert_eq!(settings.learning_min_tokens, Some(100));
    assert_eq!(settings.learning_min_confidence, Some(0.30));
    assert_eq!(settings.bootstrap_default_mode, Some("hybrid".to_string()));
}

// =============================================================================
// Types: Usage History
// =============================================================================

#[test]
fn test_usage_history_window_display() {
    assert_eq!(UsageWindow::H1.to_string(), "1h");
    assert_eq!(UsageWindow::D7.to_string(), "7d");
    assert_eq!(UsageWindow::D30.to_string(), "30d");
}

#[test]
fn test_usage_history_response_json() {
    let json = r#"{
        "org_id": "org_123",
        "project_id": "prj_456",
        "window": "1d",
        "granularity": "hourly",
        "buckets": [
            {
                "period": "2025-01-01T00:00:00Z",
                "api_calls_total": 100,
                "api_calls_patterns": 50,
                "api_calls_traces": 30,
                "api_calls_playbook": 20,
                "patterns_created": 5,
                "patterns_updated": 3,
                "patterns_deleted": 1,
                "patterns_searched": 10,
                "traces_submitted": 4,
                "bootstrap_runs": 0
            }
        ],
        "totals": {
            "api_calls_total": 100,
            "patterns_created": 5,
            "traces_submitted": 4
        }
    }"#;
    let response: UsageHistoryResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.org_id, "org_123");
    assert_eq!(response.window, UsageWindow::D1);
    assert_eq!(response.granularity, UsageGranularity::Hourly);
    assert_eq!(response.buckets.len(), 1);
    assert_eq!(response.buckets[0].api_calls_total, 100);
    assert_eq!(response.totals.api_calls_total, 100);
}

// =============================================================================
// Types: SSE Stream Events
// =============================================================================

#[test]
fn test_learning_stream_event_json() {
    let event = LearningStreamEvent {
        stage: LearningStreamStage::Analyzing,
        message: "Analyzing execution trace...".to_string(),
        timestamp: "2025-01-01T00:00:00Z".to_string(),
        data: None,
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: LearningStreamEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.stage, LearningStreamStage::Analyzing);
}

#[test]
fn test_learning_stream_stage_all_variants() {
    let stages = vec![
        (LearningStreamStage::Received, "received"),
        (LearningStreamStage::Analyzing, "analyzing"),
        (LearningStreamStage::Synthesizing, "synthesizing"),
        (LearningStreamStage::Merging, "merging"),
        (LearningStreamStage::Done, "done"),
        (LearningStreamStage::Error, "error"),
    ];

    for (stage, expected) in stages {
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_learning_stream_event_done_with_data() {
    let json = r#"{
        "stage": "done",
        "message": "Complete",
        "timestamp": "2025-01-01T00:00:00Z",
        "data": {
            "patterns_created": 3,
            "patterns_updated": 2
        }
    }"#;
    let event: LearningStreamEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.stage, LearningStreamStage::Done);
    assert!(event.data.is_some());
}

// =============================================================================
// Types: Reflection
// =============================================================================

#[test]
fn test_reflection_json_roundtrip() {
    let reflection = Reflection {
        operations: vec![DeltaOperation {
            op_type: DeltaOperationType::ADD,
            section: Some(BulletSection::StrategiesAndHardRules),
            content: Some("New rule".to_string()),
            bullet_id: None,
            confidence: Some(0.8),
            evidence: None,
            reason: Some("Extracted from trace".to_string()),
        }],
        summary: "Added 1 new pattern".to_string(),
    };

    let json = serde_json::to_string(&reflection).unwrap();
    let deserialized: Reflection = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operations.len(), 1);
    assert_eq!(deserialized.summary, "Added 1 new pattern");
}

// =============================================================================
// Types: BatchGetPatternsResponse
// =============================================================================

#[test]
fn test_batch_get_patterns_response_json() {
    let json = r#"{
        "patterns": [],
        "found_count": 0,
        "not_found": ["id-1", "id-2"]
    }"#;
    let response: BatchGetPatternsResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.found_count, 0);
    assert_eq!(response.not_found.len(), 2);
}

// =============================================================================
// Types: BulletSection Serialization
// =============================================================================

#[test]
fn test_bullet_section_serialization() {
    assert_eq!(
        serde_json::to_string(&BulletSection::StrategiesAndHardRules).unwrap(),
        "\"strategies_and_hard_rules\""
    );
    assert_eq!(
        serde_json::to_string(&BulletSection::UsefulCodeSnippets).unwrap(),
        "\"useful_code_snippets\""
    );
    assert_eq!(
        serde_json::to_string(&BulletSection::TroubleshootingAndPitfalls).unwrap(),
        "\"troubleshooting_and_pitfalls\""
    );
    assert_eq!(
        serde_json::to_string(&BulletSection::ApisToUse).unwrap(),
        "\"apis_to_use\""
    );
}

#[test]
fn test_bullet_section_deserialization() {
    let s: BulletSection = serde_json::from_str("\"strategies_and_hard_rules\"").unwrap();
    assert_eq!(s, BulletSection::StrategiesAndHardRules);
}

// =============================================================================
// Types: Subscription Status Serialization
// =============================================================================

#[test]
fn test_subscription_status_serde() {
    let statuses = vec![
        (SubscriptionStatus::Active, "\"active\""),
        (SubscriptionStatus::Trialing, "\"trialing\""),
        (SubscriptionStatus::ReadOnly, "\"read_only\""),
        (SubscriptionStatus::Blocked, "\"blocked\""),
    ];
    for (status, expected) in statuses {
        assert_eq!(serde_json::to_string(&status).unwrap(), expected);
    }
}

// =============================================================================
// Client: Creation Tests
// =============================================================================

#[test]
fn test_client_creation_user_token() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test123".to_string(),
        project_id: "test-project".to_string(),
        ..Default::default()
    };

    let client = ace_sdk_core::client::AceClient::new(config, Default::default()).unwrap();
    assert!(client.is_auto_refresh());
}

#[test]
fn test_client_creation_org_token() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_12345678test".to_string(),
        project_id: "test-project".to_string(),
        ..Default::default()
    };

    let client = ace_sdk_core::client::AceClient::new(config, Default::default()).unwrap();
    assert!(!client.is_auto_refresh());
}

#[test]
fn test_client_creation_auto_refresh_disabled() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test123".to_string(),
        project_id: "test-project".to_string(),
        ..Default::default()
    };

    let options = ace_sdk_core::client::AceClientOptions {
        auto_refresh: Some(false),
        custom_headers: None,
        graph_edges_throttle_ms: None,
    };

    let client = ace_sdk_core::client::AceClient::new(config, options).unwrap();
    assert!(!client.is_auto_refresh());
}

#[test]
fn test_client_creation_with_custom_headers() {
    let mut headers = HashMap::new();
    headers.insert("X-Custom-Header".to_string(), "test-value".to_string());

    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "test".to_string(),
        ..Default::default()
    };

    let options = ace_sdk_core::client::AceClientOptions {
        auto_refresh: None,
        custom_headers: Some(headers),
        graph_edges_throttle_ms: None,
    };

    let client = ace_sdk_core::client::AceClient::new(config, options).unwrap();
    assert!(client.is_auto_refresh());
}

// =============================================================================
// Client: Batch Get Patterns (edge cases)
// =============================================================================

#[tokio::test]
async fn test_batch_get_patterns_empty() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "test".to_string(),
        ..Default::default()
    };
    let client = ace_sdk_core::client::AceClient::new(config, Default::default()).unwrap();
    let result = client.batch_get_patterns(&[]).await.unwrap();
    assert_eq!(result.found_count, 0);
    assert!(result.patterns.is_empty());
    assert!(result.not_found.is_empty());
}

// =============================================================================
// Auth: Mask Token Tests
// =============================================================================

#[test]
fn test_mask_token_normal() {
    assert_eq!(
        ace_sdk_core::auth::mask_token("ace_user_test_token_very_long"),
        "ace_user_test_t..."
    );
}

#[test]
fn test_mask_token_short() {
    assert_eq!(ace_sdk_core::auth::mask_token("short"), "short");
}

#[test]
fn test_mask_token_exactly_15() {
    assert_eq!(
        ace_sdk_core::auth::mask_token("123456789012345"),
        "123456789012345"
    );
}

#[test]
fn test_mask_token_16_chars() {
    assert_eq!(
        ace_sdk_core::auth::mask_token("1234567890123456"),
        "123456789012345..."
    );
}

#[test]
fn test_mask_token_empty() {
    assert_eq!(ace_sdk_core::auth::mask_token(""), "(none)");
}

// =============================================================================
// Auth: Login Options
// =============================================================================

#[test]
fn test_login_options_default() {
    let opts = ace_sdk_core::auth::LoginOptions::default();
    assert_eq!(opts.client_type, "cli");
    assert_eq!(opts.timeout_ms, 300_000);
    assert!(!opts.no_browser);
}

// =============================================================================
// Config: Loading Tests
// =============================================================================

#[test]
fn test_load_config_defaults() {
    std::env::remove_var("ACE_SERVER_URL");
    std::env::remove_var("ACE_API_TOKEN");
    std::env::remove_var("ACE_PROJECT_ID");
    std::env::remove_var("ACE_CONFIG_PATH");

    let config = ace_sdk_core::config::load_config(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some("/nonexistent/path.json".to_string()),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(config.server_url, "https://ace-api.code-engine.app");
    assert_eq!(config.cache_ttl_minutes, 120);
}

#[test]
fn test_load_config_with_overrides() {
    let config = ace_sdk_core::config::load_config(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some("/nonexistent/path.json".to_string()),
            server_url: Some("https://custom.example.com".to_string()),
            api_token: Some("ace_user_test_token".to_string()),
            project_id: Some("my-project".to_string()),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(config.server_url, "https://custom.example.com");
    assert_eq!(config.api_token, "ace_user_test_token");
    assert_eq!(config.project_id, "my-project");
}

#[test]
fn test_load_config_from_temp_file() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    let config_json = r#"{
        "serverUrl": "https://test.example.com",
        "apiToken": "ace_org_test",
        "projectId": "prj_test",
        "cacheTtlMinutes": 60,
        "verbosity": "detailed"
    }"#;

    let mut file = std::fs::File::create(&config_path).unwrap();
    file.write_all(config_json.as_bytes()).unwrap();

    let config = ace_sdk_core::config::load_config(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some(config_path.to_str().unwrap().to_string()),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(config.server_url, "https://test.example.com");
    assert_eq!(config.api_token, "ace_org_test");
    assert_eq!(config.project_id, "prj_test");
    assert_eq!(config.cache_ttl_minutes, 60);
    assert_eq!(config.verbosity, Some(VerbosityLevel::Detailed));
}

#[test]
fn test_load_config_with_user_auth() {
    use std::io::Write;
    let dir = tempfile::TempDir::new().unwrap();
    let config_path = dir.path().join("config.json");

    let config_json = r#"{
        "serverUrl": "https://test.example.com",
        "auth": {
            "token": "ace_user_abc",
            "user_id": "user_1",
            "email": "test@example.com",
            "organizations": [{"org_id": "org_1", "name": "Test", "role": "admin"}]
        },
        "default_org_id": "org_1"
    }"#;

    let mut file = std::fs::File::create(&config_path).unwrap();
    file.write_all(config_json.as_bytes()).unwrap();

    let config = ace_sdk_core::config::load_config(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some(config_path.to_str().unwrap().to_string()),
            ..Default::default()
        },
        None,
    )
    .unwrap();

    assert_eq!(config.api_token, "ace_user_abc"); // Populated from auth
    assert!(config.auth.is_some());
    assert_eq!(config.default_org_id, Some("org_1".to_string()));
}

#[test]
fn test_is_configured_true() {
    let result = ace_sdk_core::config::is_configured(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some("/nonexistent".to_string()),
            api_token: Some("ace_test".to_string()),
            project_id: Some("prj_test".to_string()),
            ..Default::default()
        },
        None,
    );
    assert!(result);
}

#[test]
fn test_is_configured_false_no_token() {
    let result = ace_sdk_core::config::is_configured(
        ace_sdk_core::config::ConfigOverrides {
            config_path: Some("/nonexistent".to_string()),
            ..Default::default()
        },
        None,
    );
    assert!(!result);
}

// =============================================================================
// Config: Helper Functions
// =============================================================================

#[test]
fn test_extract_org_id_from_token() {
    assert_eq!(
        ace_sdk_core::config::extract_org_id_from_token("ace_12345678abcdef"),
        Some("org_12345678".to_string())
    );
}

#[test]
fn test_extract_org_id_from_token_invalid() {
    assert_eq!(
        ace_sdk_core::config::extract_org_id_from_token("not_ace_token"),
        None
    );
}

#[test]
fn test_extract_org_id_from_token_too_short() {
    assert_eq!(
        ace_sdk_core::config::extract_org_id_from_token("ace_sh"),
        None
    );
}

#[test]
fn test_get_token_for_org_user_auth() {
    let config = AceConfig {
        auth: Some(UserAuth {
            token: "ace_user_xyz".to_string(),
            user_id: "u1".to_string(),
            email: "e@e.com".to_string(),
            organizations: vec![],
            authenticated_at: None,
            refresh_token: None,
            expires_at: None,
            refresh_expires_at: None,
            absolute_expires_at: None,
        }),
        ..Default::default()
    };
    let token = ace_sdk_core::config::get_token_for_org(&config, "org_1").unwrap();
    assert_eq!(token, "ace_user_xyz");
}

#[test]
fn test_get_token_for_org_multi_org() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "Org 1".to_string(),
            api_token: "ace_org1_token".to_string(),
            projects: vec!["prj_1".to_string()],
        },
    );

    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };

    let token = ace_sdk_core::config::get_token_for_org(&config, "org_1").unwrap();
    assert_eq!(token, "ace_org1_token");
}

#[test]
fn test_get_token_for_org_fallback() {
    let config = AceConfig {
        api_token: "ace_fallback".to_string(),
        ..Default::default()
    };
    let token = ace_sdk_core::config::get_token_for_org(&config, "org_unknown").unwrap();
    assert_eq!(token, "ace_fallback");
}

#[test]
fn test_get_token_for_org_none() {
    let config = AceConfig::default();
    let result = ace_sdk_core::config::get_token_for_org(&config, "org_1");
    assert!(result.is_err());
}

#[test]
fn test_get_org_name_from_orgs() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "My Organization".to_string(),
            api_token: "ace_test".to_string(),
            projects: vec![],
        },
    );

    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };

    assert_eq!(
        ace_sdk_core::config::get_org_name(&config, "org_1"),
        "My Organization"
    );
}

#[test]
fn test_get_org_name_fallback() {
    let config = AceConfig::default();
    assert_eq!(
        ace_sdk_core::config::get_org_name(&config, "org_unknown"),
        "org_unknown"
    );
}

#[test]
fn test_is_multi_org_mode_true() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "Org".to_string(),
            api_token: "ace_test".to_string(),
            projects: vec![],
        },
    );
    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };
    assert!(ace_sdk_core::config::is_multi_org_mode(&config));
}

#[test]
fn test_is_multi_org_mode_false() {
    let config = AceConfig::default();
    assert!(!ace_sdk_core::config::is_multi_org_mode(&config));
}

#[test]
fn test_is_multi_org_mode_empty_map() {
    let config = AceConfig {
        orgs: Some(HashMap::new()),
        ..Default::default()
    };
    assert!(!ace_sdk_core::config::is_multi_org_mode(&config));
}

#[test]
fn test_project_belongs_to_org() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "Org".to_string(),
            api_token: "ace_test".to_string(),
            projects: vec!["prj_1".to_string(), "prj_2".to_string()],
        },
    );
    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };

    assert!(ace_sdk_core::config::project_belongs_to_org(
        &config, "org_1", "prj_1"
    ));
    assert!(!ace_sdk_core::config::project_belongs_to_org(
        &config, "org_1", "prj_3"
    ));
}

#[test]
fn test_project_belongs_to_org_single_org_mode() {
    let config = AceConfig::default();
    // In single-org mode, all projects are valid
    assert!(ace_sdk_core::config::project_belongs_to_org(
        &config,
        "any_org",
        "any_project"
    ));
}

#[test]
fn test_get_projects_for_org() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "Org".to_string(),
            api_token: "ace_test".to_string(),
            projects: vec!["prj_1".to_string(), "prj_2".to_string()],
        },
    );
    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };

    let projects = ace_sdk_core::config::get_projects_for_org(&config, "org_1");
    assert_eq!(projects.len(), 2);
    assert!(projects.contains(&"prj_1".to_string()));
}

#[test]
fn test_get_projects_for_org_nonexistent() {
    let config = AceConfig::default();
    let projects = ace_sdk_core::config::get_projects_for_org(&config, "org_1");
    assert!(projects.is_empty());
}

#[test]
fn test_list_organizations_multi_org() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "org_1".to_string(),
        OrgConfig {
            org_name: "Org 1".to_string(),
            api_token: "ace_1".to_string(),
            projects: vec!["prj_1".to_string()],
        },
    );
    orgs.insert(
        "org_2".to_string(),
        OrgConfig {
            org_name: "Org 2".to_string(),
            api_token: "ace_2".to_string(),
            projects: vec!["prj_2".to_string(), "prj_3".to_string()],
        },
    );
    let config = AceConfig {
        orgs: Some(orgs),
        ..Default::default()
    };

    let orgs = ace_sdk_core::config::list_organizations(&config);
    assert_eq!(orgs.len(), 2);
}

#[test]
fn test_list_organizations_single_org() {
    let config = AceConfig {
        api_token: "ace_12345678xyz".to_string(),
        ..Default::default()
    };
    let orgs = ace_sdk_core::config::list_organizations(&config);
    assert_eq!(orgs.len(), 1);
    assert_eq!(orgs[0].1, "Default Organization");
}

#[test]
fn test_list_organizations_empty() {
    let config = AceConfig::default();
    let orgs = ace_sdk_core::config::list_organizations(&config);
    assert!(orgs.is_empty());
}

#[test]
fn test_validate_config_valid() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_test".to_string(),
        ..Default::default()
    };
    let errors = ace_sdk_core::config::validate_config(&config);
    assert!(errors.is_empty());
}

#[test]
fn test_validate_config_missing_server() {
    let config = AceConfig {
        server_url: String::new(),
        api_token: "ace_test".to_string(),
        ..Default::default()
    };
    let errors = ace_sdk_core::config::validate_config(&config);
    assert!(errors.iter().any(|e| e.contains("serverUrl")));
}

#[test]
fn test_validate_config_no_token() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        ..Default::default()
    };
    let errors = ace_sdk_core::config::validate_config(&config);
    assert!(errors.iter().any(|e| e.contains("No API token")));
}

#[test]
fn test_validate_config_multi_org_invalid() {
    let mut orgs = HashMap::new();
    orgs.insert(
        "invalid_id".to_string(), // Doesn't start with org_
        OrgConfig {
            org_name: "Org".to_string(),
            api_token: String::new(), // Missing token
            projects: vec![],
        },
    );
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        orgs: Some(orgs),
        ..Default::default()
    };
    let errors = ace_sdk_core::config::validate_config(&config);
    assert!(errors.iter().any(|e| e.contains("missing apiToken")));
    assert!(errors.iter().any(|e| e.contains("Invalid organization ID")));
}

#[test]
fn test_config_to_context() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "prj_123".to_string(),
        cache_ttl_minutes: 60,
        default_org_id: Some("org_abc".to_string()),
        ..Default::default()
    };

    let context = ace_sdk_core::config::config_to_context(&config);
    assert_eq!(context.server_url, "https://test.example.com");
    assert_eq!(context.api_token, "ace_user_test");
    assert_eq!(context.project_id, "prj_123");
    assert_eq!(context.org_id, Some("org_abc".to_string()));
    assert_eq!(context.cache_ttl_minutes, 60);
}

#[test]
fn test_config_to_context_org_from_auth() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "prj_123".to_string(),
        auth: Some(UserAuth {
            token: "ace_user_test".to_string(),
            user_id: "u1".to_string(),
            email: "e@e.com".to_string(),
            organizations: vec![OrgMembership {
                org_id: "org_from_auth".to_string(),
                name: "From Auth".to_string(),
                role: "admin".to_string(),
                created_at: None,
            }],
            authenticated_at: None,
            refresh_token: None,
            expires_at: None,
            refresh_expires_at: None,
            absolute_expires_at: None,
        }),
        ..Default::default()
    };

    let context = ace_sdk_core::config::config_to_context(&config);
    assert_eq!(context.org_id, Some("org_from_auth".to_string()));
}

#[test]
fn test_is_valid_org_id() {
    assert!(ace_sdk_core::config::is_valid_org_id("org_abc123"));
    assert!(ace_sdk_core::config::is_valid_org_id("org_34fYIlitYk4n"));
    assert!(!ace_sdk_core::config::is_valid_org_id("invalid"));
    assert!(!ace_sdk_core::config::is_valid_org_id(""));
    assert!(!ace_sdk_core::config::is_valid_org_id("org_")); // technically valid
}

#[test]
fn test_is_valid_project_id() {
    assert!(ace_sdk_core::config::is_valid_project_id("prj_abc123def"));
    assert!(!ace_sdk_core::config::is_valid_project_id("invalid"));
    assert!(!ace_sdk_core::config::is_valid_project_id("prj_"));
    assert!(!ace_sdk_core::config::is_valid_project_id("prj_XYZ")); // uppercase not valid
}

// =============================================================================
// Config: Context Resolution
// =============================================================================

#[test]
fn test_resolve_context_from_flags() {
    let options = ResolveContextOptions {
        org: Some("org_1".to_string()),
        project: Some("prj_1".to_string()),
        cwd: None,
    };
    let result = ace_sdk_core::config::resolve_context(&options).unwrap();
    assert_eq!(result.org_id, "org_1");
    assert_eq!(result.project_id, "prj_1");
    assert_eq!(result.source, ContextSource::Flags);
}

#[test]
fn test_resolve_context_missing_both() {
    // Clear env vars
    std::env::remove_var("ACE_ORG_ID");
    std::env::remove_var("ACE_PROJECT_ID");

    let options = ResolveContextOptions::default();
    let result = ace_sdk_core::config::resolve_context(&options);
    assert!(result.is_err());
}

// =============================================================================
// Utils: Semver Tests
// =============================================================================

#[test]
fn test_parse_version_basic() {
    let v = ace_sdk_core::utils::parse_version("1.2.3").unwrap();
    assert_eq!(v.major, 1);
    assert_eq!(v.minor, 2);
    assert_eq!(v.patch, 3);
    assert!(v.prerelease.is_none());
    assert!(v.build.is_none());
}

#[test]
fn test_parse_version_prerelease() {
    let v = ace_sdk_core::utils::parse_version("1.0.0-beta.1").unwrap();
    assert_eq!(v.prerelease, Some("beta.1".to_string()));
}

#[test]
fn test_parse_version_build() {
    let v = ace_sdk_core::utils::parse_version("1.0.0+build123").unwrap();
    assert_eq!(v.build, Some("build123".to_string()));
}

#[test]
fn test_parse_version_prerelease_and_build() {
    let v = ace_sdk_core::utils::parse_version("1.0.0-rc.1+build42").unwrap();
    assert_eq!(v.prerelease, Some("rc.1".to_string()));
    assert_eq!(v.build, Some("build42".to_string()));
}

#[test]
fn test_parse_version_invalid() {
    assert!(ace_sdk_core::utils::parse_version("not-a-version").is_none());
    assert!(ace_sdk_core::utils::parse_version("1.2").is_none());
    assert!(ace_sdk_core::utils::parse_version("").is_none());
}

#[test]
fn test_compare_versions_greater() {
    assert_eq!(ace_sdk_core::utils::compare_versions("2.0.0", "1.9.9"), 1);
    assert_eq!(ace_sdk_core::utils::compare_versions("1.1.0", "1.0.9"), 1);
    assert_eq!(ace_sdk_core::utils::compare_versions("1.0.1", "1.0.0"), 1);
}

#[test]
fn test_compare_versions_less() {
    assert_eq!(ace_sdk_core::utils::compare_versions("1.0.0", "2.0.0"), -1);
    assert_eq!(ace_sdk_core::utils::compare_versions("1.0.0", "1.1.0"), -1);
    assert_eq!(ace_sdk_core::utils::compare_versions("1.0.0", "1.0.1"), -1);
}

#[test]
fn test_compare_versions_equal() {
    assert_eq!(ace_sdk_core::utils::compare_versions("1.0.0", "1.0.0"), 0);
}

#[test]
fn test_compare_versions_prerelease() {
    // Prerelease is less than stable
    assert_eq!(
        ace_sdk_core::utils::compare_versions("1.0.0-alpha", "1.0.0"),
        -1
    );
    assert_eq!(
        ace_sdk_core::utils::compare_versions("1.0.0", "1.0.0-beta"),
        1
    );
    // Compare prerelease strings
    assert_eq!(
        ace_sdk_core::utils::compare_versions("1.0.0-alpha", "1.0.0-beta"),
        -1
    );
}

#[test]
fn test_satisfies_version_gte() {
    assert!(ace_sdk_core::utils::satisfies_version("3.7.0", ">=3.6.0"));
    assert!(ace_sdk_core::utils::satisfies_version("3.6.0", ">=3.6.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("3.5.0", ">=3.6.0"));
}

#[test]
fn test_satisfies_version_caret() {
    assert!(ace_sdk_core::utils::satisfies_version("3.9.0", "^3.6.0"));
    assert!(ace_sdk_core::utils::satisfies_version("3.6.0", "^3.6.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("4.0.0", "^3.6.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("2.9.0", "^3.6.0"));
}

#[test]
fn test_satisfies_version_tilde() {
    assert!(ace_sdk_core::utils::satisfies_version("3.6.5", "~3.6.0"));
    assert!(ace_sdk_core::utils::satisfies_version("3.6.0", "~3.6.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("3.7.0", "~3.6.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("3.5.0", "~3.6.0"));
}

#[test]
fn test_satisfies_version_exact() {
    assert!(ace_sdk_core::utils::satisfies_version("1.0.0", "1.0.0"));
    assert!(!ace_sdk_core::utils::satisfies_version("1.0.1", "1.0.0"));
}

// =============================================================================
// Utils: Code Extractor Tests
// =============================================================================

#[test]
fn test_is_interesting_async() {
    assert!(ace_sdk_core::utils::is_interesting(
        "pub async fn handle() -> Result<(), Error> {}"
    ));
}

#[test]
fn test_is_interesting_impl() {
    assert!(ace_sdk_core::utils::is_interesting("impl MyStruct {"));
}

#[test]
fn test_is_interesting_derive() {
    assert!(ace_sdk_core::utils::is_interesting(
        "#[derive(Debug, Clone)]"
    ));
}

#[test]
fn test_is_interesting_result() {
    assert!(ace_sdk_core::utils::is_interesting(
        "fn foo() -> Result<T, E>"
    ));
}

#[test]
fn test_is_interesting_plain() {
    assert!(!ace_sdk_core::utils::is_interesting("let x = 5;"));
    assert!(!ace_sdk_core::utils::is_interesting("println!(\"hello\");"));
}

#[test]
fn test_extract_code_blocks_from_markdown() {
    let md = r#"# Example

```rust
pub async fn fetch_data() -> Result<Vec<u8>, Error> {
    let client = reqwest::Client::new();
    client.get("https://api.example.com").send().await?.bytes().await
}
```

```text
not interesting plain text
```
"#;
    let blocks = ace_sdk_core::utils::extract_code_blocks_from_markdown(md);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("async fn fetch_data"));
}

#[test]
fn test_extract_code_blocks_empty_markdown() {
    let blocks = ace_sdk_core::utils::extract_code_blocks_from_markdown("no code here");
    assert!(blocks.is_empty());
}

#[test]
fn test_extract_added_lines_from_diff() {
    let diff = "+++ b/src/main.rs\n+fn new_function() {\n+    println!(\"hello\");\n+}\n-old line\n context line\n";
    let result = ace_sdk_core::utils::extract_added_lines_from_diff(diff);
    assert!(result.contains("fn new_function()"));
    assert!(!result.contains("old line"));
    assert!(!result.contains("+++ b/src/main.rs")); // +++ lines excluded
}

#[test]
fn test_extract_added_lines_empty_diff() {
    let result = ace_sdk_core::utils::extract_added_lines_from_diff("");
    assert!(result.is_empty());
}

// =============================================================================
// Services: Language Detection Tests
// =============================================================================

#[test]
fn test_detect_primary_language_rust() {
    let files = vec![
        "src/main.rs".to_string(),
        "src/lib.rs".to_string(),
        "src/types.rs".to_string(),
        "README.md".to_string(),
    ];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        Some("Rust".to_string())
    );
}

#[test]
fn test_detect_primary_language_typescript() {
    let files = vec![
        "src/index.ts".to_string(),
        "src/app.tsx".to_string(),
        "package.json".to_string(),
    ];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        Some("TypeScript".to_string())
    );
}

#[test]
fn test_detect_primary_language_python() {
    let files = vec![
        "main.py".to_string(),
        "utils.py".to_string(),
        "tests.py".to_string(),
    ];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        Some("Python".to_string())
    );
}

#[test]
fn test_detect_primary_language_go() {
    let files = vec!["main.go".to_string(), "handler.go".to_string()];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        Some("Go".to_string())
    );
}

#[test]
fn test_detect_primary_language_mixed() {
    let files = vec![
        "index.ts".to_string(),
        "app.ts".to_string(),
        "main.rs".to_string(),
    ];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        Some("TypeScript".to_string())
    );
}

#[test]
fn test_detect_primary_language_empty() {
    let files: Vec<String> = vec![];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        None
    );
}

#[test]
fn test_detect_primary_language_no_recognized() {
    let files = vec!["README.md".to_string(), "Makefile".to_string()];
    assert_eq!(
        ace_sdk_core::services::detect_primary_language(&files),
        None
    );
}

// =============================================================================
// Services: SSE Line Parsing Tests
// =============================================================================

#[test]
fn test_parse_sse_line_valid() {
    let line = r#"data: {"stage":"analyzing","message":"Processing...","timestamp":"2025-01-01T00:00:00Z"}"#;
    let event = ace_sdk_core::services::parse_sse_line(line).unwrap();
    assert_eq!(event.stage, "analyzing");
    assert_eq!(event.message, "Processing...");
}

#[test]
fn test_parse_sse_line_with_data() {
    let line = r#"data: {"stage":"done","message":"Complete","timestamp":"2025-01-01T00:00:00Z","data":{"patterns_extracted":5}}"#;
    let event = ace_sdk_core::services::parse_sse_line(line).unwrap();
    assert_eq!(event.stage, "done");
    assert!(event.data.is_some());
}

#[test]
fn test_parse_sse_line_empty() {
    assert!(ace_sdk_core::services::parse_sse_line("").is_none());
}

#[test]
fn test_parse_sse_line_comment() {
    assert!(ace_sdk_core::services::parse_sse_line(": comment").is_none());
}

#[test]
fn test_parse_sse_line_no_data_prefix() {
    assert!(ace_sdk_core::services::parse_sse_line("event: test").is_none());
}

#[test]
fn test_parse_sse_line_invalid_json() {
    assert!(ace_sdk_core::services::parse_sse_line("data: {invalid json}").is_none());
}

#[test]
fn test_parse_sse_line_data_only_whitespace() {
    assert!(ace_sdk_core::services::parse_sse_line("data:   ").is_none());
}

// =============================================================================
// Services: Extension to Language
// =============================================================================

#[test]
fn test_extension_to_language() {
    assert_eq!(
        ace_sdk_core::services::extension_to_language("rs"),
        Some("Rust")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("ts"),
        Some("TypeScript")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("tsx"),
        Some("TypeScript")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("py"),
        Some("Python")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("go"),
        Some("Go")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("kt"),
        Some("Kotlin")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("swift"),
        Some("Swift")
    );
    assert_eq!(
        ace_sdk_core::services::extension_to_language("unknown"),
        None
    );
}

// =============================================================================
// Errors: Error Type Tests
// =============================================================================

#[test]
fn test_error_from_http_response_quota_exceeded() {
    let body = r#"{"error":"quota_exceeded","code":"PATTERNS_LIMIT","resource":"patterns","current":50,"limit":50,"upgrade_url":"https://example.com","message":"Quota exceeded"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(429, body);
    assert!(err.is_quota_error());
    match err {
        ace_sdk_core::errors::AceError::QuotaExceeded {
            resource,
            current,
            limit,
            ..
        } => {
            assert_eq!(resource, "patterns");
            assert_eq!(current, 50);
            assert_eq!(limit, 50);
        }
        _ => panic!("Expected QuotaExceeded"),
    }
}

#[test]
fn test_error_from_http_response_auth() {
    let body = r#"{"message":"Unauthorized"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(401, body);
    assert!(err.is_auth_error());
}

#[test]
fn test_error_from_http_response_feature_not_available() {
    let body = r#"{"error":"feature_not_available","code":"TEAMS_REQUIRED","feature":"team_sharing","required_plan":"team/pro","upgrade_url":"https://example.com"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(403, body);
    match err {
        ace_sdk_core::errors::AceError::FeatureNotAvailable {
            feature,
            required_plan,
            ..
        } => {
            assert_eq!(feature, "team_sharing");
            assert_eq!(required_plan, "team/pro");
        }
        _ => panic!("Expected FeatureNotAvailable"),
    }
}

#[test]
fn test_error_from_http_response_payment_required() {
    let body = r#"{"error":"payment_required","code":"OVERDUE","message":"Payment overdue","days_until_block":7,"upgrade_url":"https://example.com"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(402, body);
    match err {
        ace_sdk_core::errors::AceError::PaymentRequired {
            days_until_block, ..
        } => {
            assert_eq!(days_until_block, 7);
        }
        _ => panic!("Expected PaymentRequired"),
    }
}

#[test]
fn test_error_from_http_response_account_blocked() {
    let body = r#"{"error":"account_blocked","code":"BLOCKED","message":"Account suspended","upgrade_url":"https://example.com"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(403, body);
    match err {
        ace_sdk_core::errors::AceError::AccountBlocked { message, .. } => {
            assert_eq!(message, "Account suspended");
        }
        _ => panic!("Expected AccountBlocked"),
    }
}

#[test]
fn test_error_from_http_response_insufficient_permissions() {
    let body = r#"{"error":"insufficient_permissions","code":"ROLE_REQUIRED","message":"Admin required","required_role":"admin"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(403, body);
    match err {
        ace_sdk_core::errors::AceError::InsufficientPermissions { required_role, .. } => {
            assert_eq!(required_role, "admin");
        }
        _ => panic!("Expected InsufficientPermissions"),
    }
}

#[test]
fn test_error_from_http_response_plain_text() {
    let err = ace_sdk_core::errors::AceError::from_http_response(500, "Internal Server Error");
    match err {
        ace_sdk_core::errors::AceError::Http { status, .. } => assert_eq!(status, 500),
        _ => panic!("Expected Http error"),
    }
}

#[test]
fn test_error_from_http_response_generic_json() {
    let body = r#"{"message":"Something went wrong","code":"GENERIC"}"#;
    let err = ace_sdk_core::errors::AceError::from_http_response(400, body);
    match err {
        ace_sdk_core::errors::AceError::Http { status, code, .. } => {
            assert_eq!(status, 400);
            assert_eq!(code, Some("GENERIC".to_string()));
        }
        _ => panic!("Expected Http error"),
    }
}

#[test]
fn test_ace_error_is_auth_error() {
    assert!(ace_sdk_core::errors::AceError::Auth("test".to_string()).is_auth_error());
    assert!(ace_sdk_core::errors::AceError::TokenExpired.is_auth_error());
    assert!(!ace_sdk_core::errors::AceError::Other("test".to_string()).is_auth_error());
}

#[test]
fn test_ace_error_is_quota_error() {
    let err = ace_sdk_core::errors::AceError::QuotaExceeded {
        code: "TEST".to_string(),
        resource: "patterns".to_string(),
        current: 50,
        limit: 50,
        upgrade_url: "https://example.com".to_string(),
    };
    assert!(err.is_quota_error());
    assert!(!ace_sdk_core::errors::AceError::Other("test".to_string()).is_quota_error());
}

#[test]
fn test_ace_error_display() {
    let err = ace_sdk_core::errors::AceError::TokenExpired;
    let msg = format!("{}", err);
    assert!(msg.contains("expired"));
}

#[test]
fn test_ace_error_config() {
    let err = ace_sdk_core::errors::AceError::Config("Missing field".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("Missing field"));
}

#[test]
fn test_ace_error_timeout() {
    let err = ace_sdk_core::errors::AceError::Timeout(5000);
    let msg = format!("{}", err);
    assert!(msg.contains("5000"));
}

// =============================================================================
// Logger: Tests
// =============================================================================

#[test]
fn test_noop_logger() {
    use ace_sdk_core::logger::{ILogger, NoopLogger};

    let logger = NoopLogger;
    logger.debug("test");
    logger.info("test");
    logger.warn("test");
    logger.error("test");
    logger.success("test");
    logger.trace("test");
    assert!(!logger.is_verbose());
    assert!(!logger.is_trace());
}

#[test]
fn test_stderr_logger_default() {
    use ace_sdk_core::logger::{ILogger, StderrLogger};

    let logger = StderrLogger::default();
    assert!(!logger.is_verbose());
    assert!(!logger.is_trace());
}

#[test]
fn test_stderr_logger_verbose() {
    use ace_sdk_core::logger::{ILogger, StderrLogger};

    let logger = StderrLogger {
        verbose: true,
        trace: true,
    };
    assert!(logger.is_verbose());
    assert!(logger.is_trace());
}

// =============================================================================
// Cache: Tests
// =============================================================================

#[test]
fn test_cache_creation() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        5,
        Some(tmp.path().to_path_buf()),
    );
    assert!(cache.is_ok());
}

#[test]
fn test_cache_needs_sync_when_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        5,
        Some(tmp.path().to_path_buf()),
    )
    .unwrap();
    assert!(cache.needs_sync());
}

#[test]
fn test_cache_get_playbook_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        5,
        Some(tmp.path().to_path_buf()),
    )
    .unwrap();
    assert!(cache.get_playbook().is_none());
}

#[test]
fn test_cache_save_and_retrieve() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        60,
        Some(tmp.path().to_path_buf()),
    )
    .unwrap();

    let playbook = StructuredPlaybook {
        strategies_and_hard_rules: vec![PlaybookBullet {
            id: "test-1".to_string(),
            section: BulletSection::StrategiesAndHardRules,
            content: "Always use Result<T, E>".to_string(),
            domain: None,
            concrete_domain: None,
            helpful: 5.0,
            harmful: 0.0,
            confidence: 0.9,
            observations: 10.0,
            evidence: vec!["src/main.rs".to_string()],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: Some("2025-01-02T00:00:00Z".to_string()),
            root_cause: String::new(),
            error_context: String::new(),
        }],
        useful_code_snippets: vec![PlaybookBullet {
            id: "test-2".to_string(),
            section: BulletSection::UsefulCodeSnippets,
            content: "pub fn example() {}".to_string(),
            domain: None,
            concrete_domain: None,
            helpful: 3.0,
            harmful: 0.0,
            confidence: 0.8,
            observations: 5.0,
            evidence: vec![],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: Some("2025-01-01T00:00:00Z".to_string()),
            root_cause: String::new(),
            error_context: String::new(),
        }],
        ..Default::default()
    };

    cache.save_playbook(&playbook);
    let retrieved = cache.get_playbook();
    assert!(retrieved.is_some());
    let pb = retrieved.unwrap();
    assert_eq!(pb.strategies_and_hard_rules.len(), 1);
    assert_eq!(pb.useful_code_snippets.len(), 1);
    assert_eq!(pb.troubleshooting_and_pitfalls.len(), 0);
    assert_eq!(pb.apis_to_use.len(), 0);
}

#[test]
fn test_cache_clear() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        60,
        Some(tmp.path().to_path_buf()),
    )
    .unwrap();

    let playbook = StructuredPlaybook {
        apis_to_use: vec![PlaybookBullet {
            id: "test-3".to_string(),
            section: BulletSection::ApisToUse,
            content: "Use reqwest for HTTP".to_string(),
            domain: None,
            concrete_domain: None,
            helpful: 3.0,
            harmful: 0.0,
            confidence: 0.8,
            observations: 5.0,
            evidence: vec![],
            created_at: "2025-01-01T00:00:00Z".to_string(),
            last_used: Some("2025-01-01T00:00:00Z".to_string()),
            root_cause: String::new(),
            error_context: String::new(),
        }],
        ..Default::default()
    };

    cache.save_playbook(&playbook);
    assert!(cache.get_playbook().is_some());
    cache.clear();
    assert!(cache.get_playbook().is_none());
}

#[test]
fn test_cache_all_sections() {
    let tmp = tempfile::TempDir::new().unwrap();
    let cache = ace_sdk_core::cache::LocalCacheService::new(
        "test-org",
        "test-project",
        60,
        Some(tmp.path().to_path_buf()),
    )
    .unwrap();

    let make_bullet = |id: &str, section: BulletSection| PlaybookBullet {
        id: id.to_string(),
        section,
        content: format!("Content for {}", id),
        domain: None,
        concrete_domain: None,
        helpful: 1.0,
        harmful: 0.0,
        confidence: 0.5,
        observations: 1.0,
        evidence: vec![],
        created_at: "2025-01-01T00:00:00Z".to_string(),
        last_used: Some("2025-01-01T00:00:00Z".to_string()),
        root_cause: String::new(),
        error_context: String::new(),
    };

    let playbook = StructuredPlaybook {
        strategies_and_hard_rules: vec![make_bullet("s1", BulletSection::StrategiesAndHardRules)],
        useful_code_snippets: vec![make_bullet("c1", BulletSection::UsefulCodeSnippets)],
        troubleshooting_and_pitfalls: vec![make_bullet(
            "t1",
            BulletSection::TroubleshootingAndPitfalls,
        )],
        apis_to_use: vec![make_bullet("a1", BulletSection::ApisToUse)],
    };

    cache.save_playbook(&playbook);
    let retrieved = cache.get_playbook().unwrap();
    assert_eq!(retrieved.strategies_and_hard_rules.len(), 1);
    assert_eq!(retrieved.useful_code_snippets.len(), 1);
    assert_eq!(retrieved.troubleshooting_and_pitfalls.len(), 1);
    assert_eq!(retrieved.apis_to_use.len(), 1);
}

// =============================================================================
// Client: Memory Cache Tests
// =============================================================================

#[tokio::test]
async fn test_client_invalidate_cache() {
    let config = AceConfig {
        server_url: "https://test.example.com".to_string(),
        api_token: "ace_user_test".to_string(),
        project_id: "test".to_string(),
        ..Default::default()
    };
    let client = ace_sdk_core::client::AceClient::new(config, Default::default()).unwrap();
    // Should not panic
    client.invalidate_cache().await;
    client.clear_config_cache().await;
}

// =============================================================================
// Config: XDG Path Tests
// =============================================================================

#[test]
fn test_xdg_config_path() {
    let path = ace_sdk_core::config::get_xdg_config_path();
    assert!(path.to_str().unwrap().contains("ace"));
    assert!(path.to_str().unwrap().ends_with("config.json"));
}

#[test]
fn test_get_config_path() {
    let path = ace_sdk_core::config::get_config_path();
    assert!(path.to_str().unwrap().contains("ace"));
}

#[test]
fn test_autodiscover_config_path() {
    let path = ace_sdk_core::config::autodiscover_config_path();
    // Should return a path (may or may not exist)
    assert!(!path.to_str().unwrap().is_empty());
}
