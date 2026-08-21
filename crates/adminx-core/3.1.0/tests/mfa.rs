// Integration test for the MFA login flow, which the unit tests in mfa.rs don't
// reach (they only cover the TOTP/backup-code primitives in isolation). Drives
// the real handlers: an MFA-enabled user logs in → gets a *pending* session that
// is blocked from resources → clears the second factor → gets a full session.
//
// Uses the backup-code branch so no live TOTP generator is needed in the test.

use adminx_core::auth::{
    self, build_ctx, configure, guard_ui, handle_login, handle_mfa_verify, is_authorized,
    AuthConfig,
};
use adminx_core::mfa;
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};

const TOK: &str = "csrf-token-value";
const BACKUP_CODE: &str = "abcd-efgh-1234";

struct MfaMock {
    backup_hash: String,
}

#[async_trait]
impl Storage for MfaMock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![], total: 0 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(None)
    }
    async fn find_one_by(&self, _t: &str, column: &str, value: &str) -> Result<Option<Value>, StorageError> {
        if column == "email" && value == "mfa@x.io" {
            return Ok(Some(json!({
                "id": 1,
                "email": "mfa@x.io",
                "encrypted_password": auth::hash_password("secret").unwrap(),
                "role": "admin",
                "mfa_enabled": true,
                "mfa_secret": "JBSWY3DPEHPK3PXP",
                "mfa_backup_codes": self.backup_hash,
            })));
        }
        Ok(None)
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome::default())
    }
    async fn update(&self, _t: &str, _pk: &str, _i: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn delete(&self, _t: &str, _pk: &str, _i: &str, _s: bool) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn health(&self) -> bool {
        true
    }
}

fn token_from(resp: &ApiResponse) -> Option<String> {
    resp.headers
        .iter()
        .find(|(k, _)| k == "Set-Cookie")
        .and_then(|(_, v)| v.strip_prefix("adminx_token="))
        .and_then(|v| v.split(';').next())
        .map(|s| s.to_string())
}

fn location(resp: &ApiResponse) -> Option<String> {
    resp.headers
        .iter()
        .find(|(k, _)| k == "Location")
        .map(|(_, v)| v.clone())
}

#[tokio::test]
async fn mfa_login_requires_the_second_factor() {
    let backup_hash = mfa::hash_backup_codes(&[BACKUP_CODE.to_string()]).unwrap();
    configure(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(MfaMock { backup_hash }));

    // A browser that fetched a form: carries the CSRF cookie, no auth yet.
    let csrf_ctx = build_ctx("/adminx", "", None, Some(TOK));

    // --- 1. Password login on an MFA user yields a PENDING session, redirected
    //        to the challenge, not straight into the panel. ---
    let login = handle_login(&csrf_ctx, "mfa@x.io", "secret", Some(TOK)).await;
    assert_eq!(login.status, 303);
    assert_eq!(
        location(&login).as_deref(),
        Some("/adminx/mfa/verify"),
        "MFA user must be sent to the challenge, not the dashboard"
    );
    let pending = token_from(&login).expect("login sets a (pending) cookie");

    // --- 2. A pending session is blocked from resources and pushed to /mfa/verify. ---
    let pending_ctx = build_ctx("/adminx", "", Some(&pending), Some(TOK));
    assert!(
        !is_authorized(&pending_ctx, &["admin".to_string()]),
        "a pending (second-factor-owing) session must not be authorized"
    );
    let deny = guard_ui(&pending_ctx).expect("pending UI visitor is redirected");
    assert_eq!(location(&deny).as_deref(), Some("/adminx/mfa/verify"));

    // --- 3. A wrong code re-renders the challenge, no upgrade. ---
    let bad = handle_mfa_verify(&pending_ctx, "not-a-real-code", Some(TOK)).await;
    assert_eq!(bad.status, 200, "wrong code re-renders the verify page");
    assert!(token_from(&bad).is_none(), "no new cookie on a failed verify");

    // --- 4. A valid backup code clears MFA and upgrades to a full session. ---
    let ok = handle_mfa_verify(&pending_ctx, BACKUP_CODE, Some(TOK)).await;
    assert_eq!(ok.status, 303);
    assert_eq!(location(&ok).as_deref(), Some("/adminx"), "verified -> into the panel");
    let full = token_from(&ok).expect("verify issues a full-session cookie");

    let full_ctx = build_ctx("/adminx", "", Some(&full), Some(TOK));
    assert!(
        is_authorized(&full_ctx, &["admin".to_string()]),
        "the upgraded session is now authorized"
    );
}
