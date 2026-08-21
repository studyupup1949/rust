// adminx-core/src/csrf.rs
//
// CSRF protection for the HTML form posts (login, MFA setup/verify), using the
// double-submit cookie pattern: a random token is stored in its own cookie and
// mirrored in a hidden form field. A forged cross-site POST can neither read the
// victim's cookie (cross-origin) nor have it sent (`SameSite=Strict`), so the
// two can only match on a request that genuinely originated from our own page.
//
// This is deliberately stateless, like the auth JWT: nothing to store or expire
// server-side, and identical behaviour on Actix and Axum.
//
// Note the auth cookie is already `SameSite=Strict`, which alone blocks most
// forged posts to *authenticated* endpoints. The case that needs this module is
// `POST /login`, which carries no prior cookie and so gets no SameSite
// protection: without a token an attacker can force a victim's browser to log
// into the *attacker's* account. The rest is defence in depth (legacy browsers,
// and same-site-but-not-same-origin attackers such as a hostile subdomain).

use crate::request::ReqCtx;
use rand::Rng;

/// Cookie name holding the CSRF token.
pub const COOKIE_NAME: &str = "adminx_csrf";
/// Hidden form field mirroring the cookie.
pub const FIELD_NAME: &str = "_csrf";

/// A fresh 256-bit token, hex-encoded.
fn generate() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `Set-Cookie` value for `token`. A session cookie (no `Max-Age`): it only has
/// to outlive the form it guards, and a fresh one is minted whenever a form page
/// is rendered without one.
fn set_cookie_value(token: &str) -> String {
    let mut v = format!("{COOKIE_NAME}={token}; HttpOnly; SameSite=Strict; Path=/");
    if crate::auth::secure_cookie() {
        v.push_str("; Secure");
    }
    v
}

/// Token to embed in a form, plus a `Set-Cookie` value when a new one was
/// minted. An existing cookie is reused rather than replaced, so opening the
/// same form in two tabs doesn't invalidate the first one's token.
pub fn ensure(ctx: &ReqCtx) -> (String, Option<String>) {
    match ctx.csrf.as_deref().filter(|t| !t.is_empty()) {
        Some(existing) => (existing.to_string(), None),
        None => {
            let token = generate();
            let cookie = set_cookie_value(&token);
            (token, Some(cookie))
        }
    }
}

/// Constant-time string equality. Length is not secret (tokens are fixed-width).
fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// True when the submitted field matches the cookie. Both must be present and
/// non-empty, so a request that carries neither is rejected rather than passed.
pub fn verify(ctx: &ReqCtx, submitted: Option<&str>) -> bool {
    let cookie = match ctx.csrf.as_deref().filter(|t| !t.is_empty()) {
        Some(c) => c,
        None => return false,
    };
    match submitted.filter(|s| !s.is_empty()) {
        Some(s) => ct_eq(cookie, s),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(token: Option<&str>) -> ReqCtx {
        let ctx = ReqCtx::new();
        match token {
            Some(t) => ctx.with_csrf(t),
            None => ctx,
        }
    }

    #[test]
    fn tokens_are_unique_and_hex() {
        let (a, b) = (generate(), generate());
        assert_ne!(a, b);
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn ensure_mints_when_absent_and_reuses_when_present() {
        let (token, cookie) = ensure(&ctx_with(None));
        assert!(cookie.expect("should mint a cookie").contains(&token));

        let (token, cookie) = ensure(&ctx_with(Some("existing")));
        assert_eq!(token, "existing");
        assert!(cookie.is_none(), "an existing token must be reused as-is");
    }

    #[test]
    fn verify_requires_a_matching_pair() {
        assert!(verify(&ctx_with(Some("abc")), Some("abc")));
        // Forged post: attacker guesses a value but the cookie isn't sent.
        assert!(!verify(&ctx_with(None), Some("abc")));
        // Cookie present but no field (a bare cross-site form post).
        assert!(!verify(&ctx_with(Some("abc")), None));
        assert!(!verify(&ctx_with(Some("abc")), Some("xyz")));
        // Empty values must never satisfy the check.
        assert!(!verify(&ctx_with(Some("")), Some("")));
        assert!(!verify(&ctx_with(Some("abc")), Some("")));
    }
}
