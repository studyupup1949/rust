//! ACE 1.5 reward model + MatchFactors + F-080 round-trip tests.
//!
//! TDD: these tests were written BEFORE the implementation.
//! They drive the design of:
//!   - `Pattern` (1.5 reward fields + `match_factors`)
//!   - `MatchFactors` (typed, per-result, all Optional)
//!   - `PatternEffectiveness` (recommendation enum)
//!   - deprecated `helpful`/`harmful` getter methods (COLD = 0.1)
//!   - `ExecutionTrace`: `retrieval_id` + `applied_log_ids`
//!   - Search request: `task_intent` + `exploration_*` omitted when unset
//!
//! Fixture: spec/fixtures/search-1.5.json (real server capture, 6 patterns covering
//! all three decode paths: full-1.5, empty {}, null-fields, legacy-1.0).
//!
//! ENV isolation: no env vars touched here; ENV_LOCK used in client tests.

use ace_sdk_core::{
    ExecutionResult, ExecutionTrace, LearningStatistics, Pattern, PlaybookStats,
    RecommendationLabel, SearchRequest15, SearchResponse15,
};

const FIXTURE: &str = include_str!("../../../../spec/fixtures/search-1.5.json");

// =============================================================================
// Helper: load fixture
// =============================================================================

fn load_fixture() -> SearchResponse15 {
    serde_json::from_str(FIXTURE).expect("fixture must parse as SearchResponse15")
}

// =============================================================================
// (a) 1.5 fields decode on every pattern that has them
// =============================================================================

#[test]
fn a_full_15_pattern_decodes_reward_fields() {
    let resp = load_fixture();
    // first pattern: PATH_1, full 1.5 with ucb_score
    let p = &resp.similar_patterns[0];

    assert_eq!(p.payload_version, Some(15));
    assert_eq!(p.n_hot_pos, Some(0));
    assert_eq!(p.n_hot_neg, Some(0));
    assert_eq!(p.n_warm_pos, Some(1));
    assert_eq!(p.n_warm_neg, Some(0));
    assert_eq!(p.n_cold_pos, Some(1));
    assert_eq!(p.n_cold_neg, Some(0));
    assert_eq!(p.cumulative_v15_reward, Some(0.8));
    assert_eq!(p.n_retrieval_no_apply, Some(1));
}

#[test]
fn a_second_15_pattern_with_non_zero_tier_counters() {
    let resp = load_fixture();
    // second pattern: PATH_1, n_hot_pos=3, n_warm_pos=5, n_cold_pos=2
    let p = &resp.similar_patterns[1];

    assert_eq!(p.payload_version, Some(15));
    assert_eq!(p.n_hot_pos, Some(3));
    assert_eq!(p.n_warm_pos, Some(5));
    assert_eq!(p.n_cold_pos, Some(2));
    assert_eq!(p.n_hot_neg, Some(0));
    assert_eq!(p.n_warm_neg, Some(1));
    assert_eq!(p.n_cold_neg, Some(0));
    assert_eq!(p.cumulative_v15_reward, Some(6.7));
}

// =============================================================================
// (b) match_factors typed + captured per result — retrieval_log_id (int) + retrieval_id (UUID)
// =============================================================================

#[test]
fn b_full_match_factors_decode_with_ucb_and_log_id() {
    let resp = load_fixture();
    // first pattern: full match_factors with ucb_score + bandit_rank + retrieval_log_id
    let mf = resp.similar_patterns[0]
        .match_factors
        .as_ref()
        .expect("pattern[0] must have match_factors");

    assert!((mf.semantic_score.unwrap_or(0.0) - 0.7698).abs() < 1e-4);
    assert_eq!(mf.domain_boost, Some(false));
    assert_eq!(mf.formula_boost_applied, Some(true));
    assert!((mf.ucb_score.unwrap() - 0.810429).abs() < 1e-5);
    assert_eq!(mf.bandit_rank, Some(1));
    // retrieval_log_id is INTEGER (i64)
    assert_eq!(mf.retrieval_log_id, Some(47868_i64));
    assert_eq!(
        mf.retrieval_id.as_deref(),
        Some("dcca8a60-8c8d-42bb-9a5c-1115d0e2d808")
    );
    assert_eq!(mf.shadow_mode, Some(true));
}

#[test]
fn b_second_pattern_retrieval_log_id_47870() {
    let resp = load_fixture();
    let mf = resp.similar_patterns[1]
        .match_factors
        .as_ref()
        .expect("pattern[1] must have match_factors");
    assert_eq!(mf.retrieval_log_id, Some(47870_i64));
}

// =============================================================================
// (c) empty {} match_factors → all fields None, NO throw
// =============================================================================

#[test]
fn c_empty_match_factors_decodes_to_all_none() {
    let resp = load_fixture();
    // pattern[2]: PATH_2, match_factors: {}
    let p = &resp.similar_patterns[2];
    assert_eq!(p.payload_version, Some(15));

    // match_factors is present (Some) but all internal fields are None
    let mf = p
        .match_factors
        .as_ref()
        .expect("pattern[2] has match_factors present (even if empty)");
    assert!(mf.semantic_score.is_none());
    assert!(mf.ucb_score.is_none());
    assert!(mf.bandit_rank.is_none());
    assert!(mf.retrieval_log_id.is_none());
    assert!(mf.retrieval_id.is_none());
}

#[test]
fn c_null_fields_match_factors_no_throw() {
    let resp = load_fixture();
    // pattern[3]: PATH_2, match_factors has ucb_score:null, bandit_rank:null, retrieval_log_id:null
    let mf = resp.similar_patterns[3]
        .match_factors
        .as_ref()
        .expect("pattern[3] has match_factors");
    // base fields present
    assert!((mf.semantic_score.unwrap() - 0.7584).abs() < 1e-4);
    // ucb/bandit/log_id are null → None
    assert!(mf.ucb_score.is_none());
    assert!(mf.bandit_rank.is_none());
    assert!(mf.retrieval_log_id.is_none());
    // shadow_mode: false
    assert_eq!(mf.shadow_mode, Some(false));
}

// =============================================================================
// (d) legacy 1.0 row decodes with v15 fields defaulted — NO throw
// =============================================================================

#[test]
fn d_legacy_10_row_decodes_gracefully() {
    let resp = load_fixture();
    // pattern[4]: PATH_3, legacy 1.0 — no payload_version, no n_hot_pos, no match_factors
    let p = &resp.similar_patterns[4];

    assert_eq!(p.id, "ctx-8328763857-6733");
    // v15 fields absent → None defaults
    assert!(p.payload_version.is_none());
    assert!(p.n_hot_pos.is_none());
    assert!(p.n_warm_pos.is_none());
    assert!(p.cumulative_v15_reward.is_none());
    assert!(p.n_retrieval_no_apply.is_none());
    // match_factors absent → None
    assert!(p.match_factors.is_none());
    // legacy fields present
    assert!((p.confidence - 0.81).abs() < 1e-6);
}

// =============================================================================
// (e) deprecated helpful/harmful getters — COLD = 0.1
// =============================================================================

const COLD_WEIGHT: f64 = 0.1;

#[test]
fn e_deprecated_helpful_computes_from_tier_counters() {
    // pattern[1]: n_hot_pos=3, n_warm_pos=5, n_cold_pos=2
    // helpful = 3*1.0 + 5*0.7 + 2*0.1 = 3.0 + 3.5 + 0.2 = 6.7
    let resp = load_fixture();
    let p = &resp.similar_patterns[1];

    let expected = p.n_hot_pos.unwrap_or(0) as f64 * 1.0
        + p.n_warm_pos.unwrap_or(0) as f64 * 0.7
        + p.n_cold_pos.unwrap_or(0) as f64 * COLD_WEIGHT;

    let got = p.legacy_helpful();
    assert!(
        (got - expected).abs() < 1e-9,
        "helpful: expected {expected}, got {got}"
    );
}

#[test]
fn e_deprecated_harmful_computes_from_tier_counters() {
    // pattern[1]: n_hot_neg=0, n_warm_neg=1, n_cold_neg=0
    // harmful = 0*1.0 + 1*0.7 + 0*0.1 = 0.7
    let resp = load_fixture();
    let p = &resp.similar_patterns[1];

    let expected = p.n_hot_neg.unwrap_or(0) as f64 * 1.0
        + p.n_warm_neg.unwrap_or(0) as f64 * 0.7
        + p.n_cold_neg.unwrap_or(0) as f64 * COLD_WEIGHT;

    let got = p.legacy_harmful();
    assert!(
        (got - expected).abs() < 1e-9,
        "harmful: expected {expected}, got {got}"
    );
}

#[test]
fn e_deprecated_getters_return_zero_for_legacy_10_row() {
    // legacy row has no tier counters → getters return 0.0
    let resp = load_fixture();
    let p = &resp.similar_patterns[4];
    assert_eq!(p.legacy_helpful(), 0.0);
    assert_eq!(p.legacy_harmful(), 0.0);
}

// =============================================================================
// (f) unknown / absent effectiveness.recommendation → Unknown, no crash
// =============================================================================

#[test]
fn f_known_recommendation_parses_correctly() {
    let resp = load_fixture();
    // pattern[1] has effectiveness.recommendation = "reliable"
    let p = &resp.similar_patterns[1];
    let eff = p
        .effectiveness
        .as_ref()
        .expect("pattern[1] has effectiveness");
    assert_eq!(eff.recommendation, Some(RecommendationLabel::Reliable));
}

#[test]
fn f_absent_recommendation_produces_unknown() {
    let resp = load_fixture();
    // pattern[0] has no effectiveness field
    let p = &resp.similar_patterns[0];
    // effectiveness absent → None; calling helper returns Unknown
    let label = p
        .effectiveness
        .as_ref()
        .and_then(|e| e.recommendation.clone())
        .unwrap_or(RecommendationLabel::Unknown);
    assert_eq!(label, RecommendationLabel::Unknown);
}

#[test]
fn f_unknown_recommendation_string_decodes_to_unknown() {
    let json = r#"{
        "id": "test",
        "content": "test",
        "confidence": 0.8,
        "section": "strategies_and_hard_rules",
        "created_at": "2026-01-01T00:00:00Z",
        "effectiveness": { "recommendation": "some_future_value_server_may_add" }
    }"#;
    let p: Pattern = serde_json::from_str(json).expect("must not throw on unknown recommendation");
    let label = p
        .effectiveness
        .as_ref()
        .and_then(|e| e.recommendation.clone())
        .unwrap_or(RecommendationLabel::Unknown);
    assert_eq!(label, RecommendationLabel::Unknown);
}

// =============================================================================
// (g) F-080 round-trip: retrieval_log_id → ExecutionTrace.applied_log_ids
// =============================================================================

#[test]
fn g_f080_roundtrip_retrieval_log_ids_survive_into_trace() {
    let resp = load_fixture();

    // Collect retrieval_log_ids from patterns that have them
    let applied_ids: Vec<i64> = resp
        .similar_patterns
        .iter()
        .filter_map(|p| p.match_factors.as_ref())
        .filter_map(|mf| mf.retrieval_log_id)
        .collect();

    // We expect 3 non-null log_ids (patterns 0, 1, 5 in fixture)
    assert!(
        !applied_ids.is_empty(),
        "should have at least one retrieval_log_id"
    );
    assert!(applied_ids.contains(&47868));
    assert!(applied_ids.contains(&47870));
    assert!(applied_ids.contains(&47869));

    // Capture search-scoped retrieval_id
    let retrieval_id = resp.retrieval_id.as_deref().unwrap_or("");
    assert_eq!(retrieval_id, "dcca8a60-8c8d-42bb-9a5c-1115d0e2d808");

    // Build ExecutionTrace with F-080 fields
    let trace = ExecutionTrace {
        task: "test task".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output: "done".to_string(),
            error: None,
            summary: None,
        },
        playbook_used: vec![],
        timestamp: "2026-06-05T00:00:00Z".to_string(),
        git: None,
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id: Some(retrieval_id.to_string()),
        applied_log_ids: Some(applied_ids.clone()),
    };

    // Serialize and verify F-080 fields appear in JSON
    let json = serde_json::to_string(&trace).expect("serialize");
    assert!(
        json.contains("\"retrieval_id\""),
        "retrieval_id must be serialized: {json}"
    );
    assert!(
        json.contains("\"applied_log_ids\""),
        "applied_log_ids must be serialized: {json}"
    );
    assert!(
        json.contains("47868"),
        "log id 47868 must be in json: {json}"
    );
    assert!(
        json.contains("47870"),
        "log id 47870 must be in json: {json}"
    );

    // Deserialize round-trip
    let rt: ExecutionTrace = serde_json::from_str(&json).expect("deserialize round-trip");
    assert_eq!(
        rt.retrieval_id.as_deref(),
        Some("dcca8a60-8c8d-42bb-9a5c-1115d0e2d808")
    );
    let rt_ids = rt.applied_log_ids.unwrap();
    assert_eq!(rt_ids.len(), applied_ids.len());
    assert!(rt_ids.contains(&47868));
    assert!(rt_ids.contains(&47870));
}

#[test]
fn g_f080_fields_omitted_when_none() {
    let trace = ExecutionTrace {
        task: "task".to_string(),
        trajectory: vec![],
        result: ExecutionResult {
            success: true,
            output: "ok".to_string(),
            error: None,
            summary: None,
        },
        playbook_used: vec![],
        timestamp: "2026-06-05T00:00:00Z".to_string(),
        git: None,
        session_id: None,
        agent_id: None,
        agent_type: None,
        parent_agent_id: None,
        retrieval_id: None,
        applied_log_ids: None,
    };

    let json = serde_json::to_string(&trace).expect("serialize");
    assert!(
        !json.contains("retrieval_id"),
        "retrieval_id must be omitted when None: {json}"
    );
    assert!(
        !json.contains("applied_log_ids"),
        "applied_log_ids must be omitted when None: {json}"
    );
}

// =============================================================================
// (h) Search request includes task_intent + exploration_* ONLY when set
// =============================================================================

#[test]
fn h_search_request_omits_optional_fields_when_not_set() {
    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp",
            "content": "query",
            "confidence": 0.8,
            "created_at": "2026-01-01T00:00:00Z",
            "section": "general"
        }),
        threshold: Some(0.75),
        top_k: Some(10),
        task_intent: None,
        exploration_enabled: None,
        exploration_rate: None,
        session_id: None,
    };

    let json = serde_json::to_string(&req).expect("serialize");
    assert!(
        !json.contains("task_intent"),
        "task_intent must be absent: {json}"
    );
    assert!(
        !json.contains("exploration_enabled"),
        "exploration_enabled must be absent: {json}"
    );
    assert!(
        !json.contains("exploration_rate"),
        "exploration_rate must be absent: {json}"
    );
}

#[test]
fn h_search_request_includes_optional_fields_when_set() {
    let req = SearchRequest15 {
        pattern: serde_json::json!({
            "id": "temp",
            "content": "query",
            "confidence": 0.8,
            "created_at": "2026-01-01T00:00:00Z",
            "section": "general"
        }),
        threshold: Some(0.75),
        top_k: Some(10),
        task_intent: Some("routine".to_string()),
        exploration_enabled: Some(true),
        exploration_rate: Some(0.25),
        session_id: None,
    };

    let json = serde_json::to_string(&req).expect("serialize");
    assert!(
        json.contains("\"task_intent\":\"routine\""),
        "task_intent present: {json}"
    );
    assert!(
        json.contains("\"exploration_enabled\":true"),
        "exploration_enabled present: {json}"
    );
    assert!(
        json.contains("\"exploration_rate\":0.25"),
        "exploration_rate present: {json}"
    );
}

// =============================================================================
// (extra) at-risk pattern: cumulative_v15_reward == 0.0 → is_at_risk()
// =============================================================================

#[test]
fn at_risk_pattern_detected() {
    let resp = load_fixture();
    // pattern[5]: cumulative_v15_reward == 0.0
    let p = &resp.similar_patterns[5];
    assert_eq!(p.cumulative_v15_reward, Some(0.0));
    assert!(p.is_at_risk(), "cumulative_v15_reward == 0.0 means at-risk");
}

#[test]
fn non_at_risk_pattern() {
    let resp = load_fixture();
    // pattern[0]: cumulative_v15_reward == 0.8
    let p = &resp.similar_patterns[0];
    assert_eq!(p.cumulative_v15_reward, Some(0.8));
    assert!(!p.is_at_risk());
}

// =============================================================================
// (extra) top-level retrieval_id on search response
// =============================================================================

#[test]
fn search_response_has_top_level_retrieval_id() {
    let resp = load_fixture();
    assert_eq!(
        resp.retrieval_id.as_deref(),
        Some("dcca8a60-8c8d-42bb-9a5c-1115d0e2d808")
    );
    assert_eq!(resp.count, 6);
}

// =============================================================================
// (i) PlaybookStats — ACE 1.5 reward-aggregate fields
// =============================================================================

#[test]
fn i_playbook_stats_reward_aggregate_fields_decode_with_values() {
    let json = r#"{
        "avg_confidence": 0.82,
        "cumulative_reward_total": 123.45,
        "hot_total": 12,
        "warm_total": 34,
        "cold_total": 7,
        "at_risk_count": 3,
        "patterns_with_v15_reward": 50
    }"#;
    let stats: PlaybookStats = serde_json::from_str(json).expect("must parse PlaybookStats");
    assert!((stats.cumulative_reward_total - 123.45).abs() < 1e-9);
    assert_eq!(stats.hot_total, 12);
    assert_eq!(stats.warm_total, 34);
    assert_eq!(stats.cold_total, 7);
    assert_eq!(stats.at_risk_count, 3);
    assert_eq!(stats.patterns_with_v15_reward, 50);
}

#[test]
fn i_playbook_stats_reward_aggregate_fields_default_to_zero_when_absent() {
    // Older server responses that don't include the 1.5 reward-aggregate
    // fields must deserialize without error and yield zero values.
    let json = r#"{"avg_confidence": 0.75}"#;
    let stats: PlaybookStats =
        serde_json::from_str(json).expect("must parse without reward fields");
    assert_eq!(stats.cumulative_reward_total, 0.0);
    assert_eq!(stats.hot_total, 0);
    assert_eq!(stats.warm_total, 0);
    assert_eq!(stats.cold_total, 0);
    assert_eq!(stats.at_risk_count, 0);
    assert_eq!(stats.patterns_with_v15_reward, 0);
}

// =============================================================================
// (j) LearningStatistics — ACE 1.5 reward fields
// =============================================================================

#[test]
fn j_learning_statistics_reward_fields_decode_with_values() {
    let json = r#"{
        "patterns_created": 2,
        "patterns_updated": 5,
        "patterns_pruned": 0,
        "patterns_deduplicated": 1,
        "cumulative_v15_reward_delta": 3.7,
        "patterns_rewarded": 4,
        "reward_tier": "warm"
    }"#;
    let stats: LearningStatistics =
        serde_json::from_str(json).expect("must parse LearningStatistics");
    assert!((stats.cumulative_v15_reward_delta - 3.7).abs() < 1e-9);
    assert_eq!(stats.patterns_rewarded, 4);
    assert_eq!(stats.reward_tier, "warm");
}

#[test]
fn j_learning_statistics_reward_fields_default_to_zero_when_absent() {
    // Older /traces done-responses that omit the 1.5 reward fields must
    // deserialize without error and yield zero / empty-string defaults.
    let json = r#"{
        "patterns_created": 1,
        "patterns_updated": 0,
        "patterns_pruned": 0,
        "patterns_deduplicated": 0
    }"#;
    let stats: LearningStatistics =
        serde_json::from_str(json).expect("must parse without reward fields");
    assert_eq!(stats.cumulative_v15_reward_delta, 0.0);
    assert_eq!(stats.patterns_rewarded, 0);
    assert_eq!(stats.reward_tier, "");
}

#[test]
fn j_learning_statistics_reward_tier_omitted_from_json_when_empty() {
    // `reward_tier` uses skip_serializing_if = "String::is_empty" so an
    // empty string must NOT appear in the serialized JSON.
    let stats = LearningStatistics {
        patterns_created: 1,
        patterns_updated: 0,
        patterns_pruned: 0,
        patterns_deduplicated: 0,
        by_section: Default::default(),
        average_confidence: 0.0,
        helpful_delta: 0,
        helpful_count: 0,
        harmful_count: 0,
        analysis_time_seconds: 0.0,
        cumulative_v15_reward_delta: 0.0,
        patterns_rewarded: 0,
        reward_tier: String::new(),
    };
    let json = serde_json::to_string(&stats).expect("serialize");
    assert!(
        !json.contains("reward_tier"),
        "empty reward_tier must be omitted from JSON: {json}"
    );
}
