//! Regression tests for PlaybookBullet schema fix (0.4.1).
//!
//! Verifies that the Rust SDK can decode prod-shape `PlaybookBullet`
//! payloads from `ace-api.code-engine.app`:
//!   - `helpful` / `harmful` / `observations` are floats (e.g. 81.5, 9.5, 91.0)
//!   - `last_used` is `null` for ~24% of bullets (never-retrieved patterns)
//!
//! Fixture sourced from `spec/fixtures/playbook-bullet-prod-2026-04-28.json`
//! (sample size 551 bullets, captured via prod live verification).

use ace_sdk_core::PlaybookBullet;

const PROD_FIXTURE: &str =
    include_str!("../../../../spec/fixtures/playbook-bullet-prod-2026-04-28.json");

#[test]
fn decodes_bullet_with_fractional_counters() {
    let fixture: serde_json::Value = serde_json::from_str(PROD_FIXTURE).unwrap();
    let raw = &fixture["with_fractional_counters"];

    let bullet: PlaybookBullet = serde_json::from_value(raw.clone())
        .expect("PlaybookBullet must decode fractional helpful/harmful/observations");

    assert_eq!(bullet.id, "e5875b2b-c0f8-549a-83b2-eaeccf3fb802");
    assert_eq!(bullet.helpful, 81.5);
    assert_eq!(bullet.harmful, 9.5);
    assert_eq!(bullet.observations, 91.0);
    assert_eq!(bullet.confidence, 0.9);
    assert_eq!(
        bullet.last_used,
        Some("2026-04-21T16:53:11.972602Z".to_string())
    );
    assert_eq!(bullet.evidence.len(), 4);
}

#[test]
fn decodes_bullet_with_null_last_used() {
    let fixture: serde_json::Value = serde_json::from_str(PROD_FIXTURE).unwrap();
    let raw = &fixture["with_null_last_used"];

    let bullet: PlaybookBullet =
        serde_json::from_value(raw.clone()).expect("PlaybookBullet must decode null last_used");

    assert_eq!(bullet.id, "083e8ee3-8544-53f9-8674-204e339fcf55");
    assert_eq!(bullet.helpful, 1.0);
    assert_eq!(bullet.harmful, 0.0);
    assert_eq!(bullet.observations, 1.0);
    assert!(
        bullet.last_used.is_none(),
        "last_used must be None for never-retrieved bullets"
    );
}

#[test]
fn decodes_fractional_counters_inline() {
    // Inline regression: minimal payload that previously crashed serde_json
    // with `error decoding response body` because `helpful=30.4` did not fit
    // into `i32`.
    let json = r#"{
        "id": "regression-1",
        "section": "strategies_and_hard_rules",
        "content": "fractional counter regression",
        "helpful": 30.4,
        "harmful": 1.6,
        "observations": 32.0,
        "created_at": "2026-04-28T00:00:00Z",
        "last_used": null
    }"#;

    let bullet: PlaybookBullet = serde_json::from_str(json).unwrap();
    assert_eq!(bullet.helpful, 30.4);
    assert_eq!(bullet.harmful, 1.6);
    assert_eq!(bullet.observations, 32.0);
    assert!(bullet.last_used.is_none());
}

// =============================================================================
// is_at_risk() on PlaybookBullet — playbook read path + top-patterns path
//
//   reward < 0.0  -> at-risk (net-negative / harmful)
//   reward == 0.0 -> NOT at-risk (uncredited / neutral — fresh pattern)
//   reward > 0.0  -> NOT at-risk (net-positive)
//   reward absent -> NOT at-risk (legacy row, no reward data)
// =============================================================================

/// Build a minimal PlaybookBullet JSON with the given cumulative_v15_reward value.
fn make_bullet_json(reward_json: &str) -> String {
    format!(
        r#"{{
            "id": "test-bullet-1",
            "section": "strategies_and_hard_rules",
            "content": "test content",
            "helpful": 1.0,
            "harmful": 0.0,
            "observations": 1.0,
            "created_at": "2026-01-01T00:00:00Z",
            "cumulative_v15_reward": {reward_json}
        }}"#
    )
}

#[test]
fn playbook_bullet_negative_reward_is_at_risk() {
    // TDD: PlaybookBullet with cumulative_v15_reward < 0 must surface is_at_risk() == true.
    // Covers the getPlaybook decode path.
    let json = make_bullet_json("-3.5");
    let bullet: PlaybookBullet =
        serde_json::from_str(&json).expect("PlaybookBullet must decode cumulative_v15_reward");
    assert_eq!(bullet.cumulative_v15_reward, Some(-3.5));
    assert!(
        bullet.is_at_risk(),
        "reward < 0.0 must surface is_at_risk() == true on PlaybookBullet"
    );
}

#[test]
fn playbook_bullet_zero_reward_is_not_at_risk() {
    // reward == 0.0 means uncredited/neutral — NOT at-risk.
    let json = make_bullet_json("0.0");
    let bullet: PlaybookBullet = serde_json::from_str(&json).unwrap();
    assert_eq!(bullet.cumulative_v15_reward, Some(0.0));
    assert!(
        !bullet.is_at_risk(),
        "reward == 0.0 (uncredited/neutral) must NOT be at-risk"
    );
}

#[test]
fn playbook_bullet_positive_reward_is_not_at_risk() {
    let json = make_bullet_json("4.2");
    let bullet: PlaybookBullet = serde_json::from_str(&json).unwrap();
    assert_eq!(bullet.cumulative_v15_reward, Some(4.2));
    assert!(!bullet.is_at_risk(), "reward > 0.0 must NOT be at-risk");
}

#[test]
fn playbook_bullet_absent_reward_is_not_at_risk() {
    // No cumulative_v15_reward field (legacy row) — must decode and be NOT at-risk.
    let json = r#"{
        "id": "legacy-bullet",
        "section": "apis_to_use",
        "content": "legacy content",
        "helpful": 0.5,
        "harmful": 0.0,
        "observations": 1.0,
        "created_at": "2026-01-01T00:00:00Z"
    }"#;
    let bullet: PlaybookBullet = serde_json::from_str(json)
        .expect("PlaybookBullet must decode without cumulative_v15_reward");
    assert_eq!(bullet.cumulative_v15_reward, None);
    assert!(
        !bullet.is_at_risk(),
        "None reward (legacy row) must NOT be at-risk"
    );
}

#[test]
fn playbook_bullet_null_reward_is_not_at_risk() {
    // Explicit null in JSON (server sends null for rows without reward data).
    let json = make_bullet_json("null");
    let bullet: PlaybookBullet = serde_json::from_str(&json).unwrap();
    assert_eq!(bullet.cumulative_v15_reward, None);
    assert!(!bullet.is_at_risk(), "null reward must NOT be at-risk");
}

#[test]
fn playbook_bullet_barely_negative_reward_is_at_risk() {
    // Boundary: -0.001 is still < 0.0 -> at-risk.
    let json = make_bullet_json("-0.001");
    let bullet: PlaybookBullet = serde_json::from_str(&json).unwrap();
    assert!(
        bullet.is_at_risk(),
        "reward = -0.001 (barely negative) must be at-risk"
    );
}

#[test]
fn top_patterns_bullet_negative_reward_is_at_risk() {
    // TDD: covers the getTopPatterns decode path — PlaybookStats.top_harmful contains
    // PlaybookBullet with negative reward; is_at_risk() must return true.
    use ace_sdk_core::PlaybookStats;
    let json = r#"{
        "avg_confidence": 0.7,
        "top_helpful": [],
        "top_harmful": [
            {
                "id": "harm-bullet-1",
                "section": "strategies_and_hard_rules",
                "content": "harmful pattern",
                "helpful": 0.0,
                "harmful": 5.0,
                "observations": 5.0,
                "created_at": "2026-01-01T00:00:00Z",
                "cumulative_v15_reward": -2.5
            }
        ]
    }"#;
    let stats: PlaybookStats =
        serde_json::from_str(json).expect("PlaybookStats with top_harmful must decode");
    assert_eq!(stats.top_harmful.len(), 1);
    let bullet = &stats.top_harmful[0];
    assert_eq!(bullet.cumulative_v15_reward, Some(-2.5));
    assert!(
        bullet.is_at_risk(),
        "top_harmful bullet with reward < 0 must surface is_at_risk() == true"
    );
}

#[test]
fn top_patterns_bullet_positive_reward_is_not_at_risk() {
    // top_helpful bullet with positive reward: NOT at-risk.
    use ace_sdk_core::PlaybookStats;
    let json = r#"{
        "avg_confidence": 0.9,
        "top_helpful": [
            {
                "id": "help-bullet-1",
                "section": "useful_code_snippets",
                "content": "helpful pattern",
                "helpful": 8.0,
                "harmful": 0.0,
                "observations": 8.0,
                "created_at": "2026-01-01T00:00:00Z",
                "cumulative_v15_reward": 6.4
            }
        ],
        "top_harmful": []
    }"#;
    let stats: PlaybookStats =
        serde_json::from_str(json).expect("PlaybookStats with top_helpful must decode");
    assert_eq!(stats.top_helpful.len(), 1);
    let bullet = &stats.top_helpful[0];
    assert_eq!(bullet.cumulative_v15_reward, Some(6.4));
    assert!(
        !bullet.is_at_risk(),
        "top_helpful bullet with reward > 0 must NOT be at-risk"
    );
}
