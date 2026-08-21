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
