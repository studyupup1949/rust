// adminx-core/src/auth.rs
//
// Framework-neutral authentication: a signed JWT carried in an HttpOnly cookie.
// No server-side session store and no framework dependency — the core issues the
// token and emits `Set-Cookie` via `ApiResponse` headers; adapters only extract
// the cookie value from their request and hand it back here to verify.

use crate::error::CoreError;
use crate::ratelimit;
use crate::request::{Claims, ReqCtx};
use crate::response::ApiResponse;
use crate::storage::storage;
use crate::ui;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cookie name holding the JWT.
pub const COOKIE_NAME: &str = "adminx_token";

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_ttl_secs: i64,
    /// Table/collection holding admin users.
    pub admin_table: String,
    /// Set the `Secure` flag on the auth cookie (disable only for local HTTP).
    pub secure_cookie: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: String::new(),
            token_ttl_secs: 86_400,
            admin_table: "adminx_users".to_string(),
            secure_cookie: true,
        }
    }
}

static AUTH_CONFIG: OnceCell<AuthConfig> = OnceCell::new();

/// Enable authentication globally. Until this is called, auth is disabled and
/// every resource is publicly accessible (convenient for quick starts / tests).
pub fn configure(config: AuthConfig) {
    if AUTH_CONFIG.set(config).is_err() {
        tracing::warn!("adminx auth already configured; ignoring reconfigure");
    }
}

pub fn is_configured() -> bool {
    AUTH_CONFIG.get().is_some()
}

/// Whether cookies should carry the `Secure` flag. Defaults to `true` (the safe
/// side) when auth isn't configured. Shared with `crate::csrf`.
pub(crate) fn secure_cookie() -> bool {
    config().map(|c| c.secure_cookie).unwrap_or(true)
}

fn config() -> Option<&'static AuthConfig> {
    AUTH_CONFIG.get()
}

// ===== JWT =====

/// Session is fully authenticated (second factor satisfied, or none required).
pub const MFA_OK: &str = "ok";
/// Password verified, but a second factor is still required before access.
pub const MFA_PENDING: &str = "pending";

fn default_mfa_ok() -> String {
    MFA_OK.to_string()
}

#[derive(Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    email: String,
    role: String,
    exp: i64,
    /// MFA step. Defaulted for tokens issued before MFA existed so they keep
    /// working (treated as fully authenticated).
    #[serde(default = "default_mfa_ok")]
    mfa: String,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Issue a signed JWT for a principal at the given MFA `step` (`MFA_OK` or
/// `MFA_PENDING`). Returns `None` if auth isn't configured.
pub fn issue_token(sub: &str, email: &str, role: &str, step: &str) -> Option<String> {
    let cfg = config()?;
    let claims = TokenClaims {
        sub: sub.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: now_secs() + cfg.token_ttl_secs,
        mfa: step.to_string(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .ok()
}

/// Verify a JWT and return the principal, or `None` if invalid/expired/unconfigured.
pub fn verify_token(token: &str) -> Option<Claims> {
    let cfg = config()?;
    let data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;
    let c = data.claims;
    Some(Claims {
        sub: c.sub,
        email: c.email,
        role: c.role.clone(),
        roles: vec![c.role],
        mfa: c.mfa,
    })
}

// ===== Passwords =====

pub fn hash_password(password: &str) -> Result<String, CoreError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| CoreError::Internal(format!("password hash failed: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

// ===== Cookies (built as ApiResponse headers) =====

fn set_cookie_value(token: &str) -> String {
    let max_age = config().map(|c| c.token_ttl_secs).unwrap_or(86_400);
    let mut v = format!(
        "{COOKIE_NAME}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age={max_age}"
    );
    if secure_cookie() {
        v.push_str("; Secure");
    }
    v
}

fn clear_cookie_value() -> String {
    format!("{COOKIE_NAME}=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0")
}

// ===== Context building & guards =====

/// Build a request context, verifying the auth cookie into `claims` if present
/// and carrying the raw CSRF cookie through for `crate::csrf` to check.
pub fn build_ctx(mount: &str, query: &str, token: Option<&str>, csrf: Option<&str>) -> ReqCtx {
    let mut ctx = ReqCtx::new().with_mount(mount).with_query(query);
    if let Some(t) = token {
        if let Some(claims) = verify_token(t) {
            ctx = ctx.with_claims(claims);
        }
    }
    if let Some(c) = csrf {
        ctx = ctx.with_csrf(c);
    }
    ctx
}

/// True when the session still owes a second factor (password verified but MFA
/// not yet satisfied). Such a session must not reach protected pages.
pub(crate) fn mfa_pending(ctx: &ReqCtx) -> bool {
    matches!(&ctx.claims, Some(c) if c.mfa == MFA_PENDING)
}

/// True if the principal in `ctx` holds at least one of `allowed_roles` and has
/// cleared MFA. Always true when auth is not configured.
pub fn is_authorized(ctx: &ReqCtx, allowed_roles: &[String]) -> bool {
    if !is_configured() {
        return true;
    }
    if mfa_pending(ctx) {
        return false;
    }
    let roles = ctx.roles();
    allowed_roles.iter().any(|r| roles.contains(r))
}

/// Redirect a blocked UI visitor to the right place: the MFA challenge when a
/// second factor is still pending, otherwise the login page.
pub fn login_redirect(ctx: &ReqCtx) -> ApiResponse {
    if mfa_pending(ctx) {
        ApiResponse::redirect(format!("{}/mfa/verify", ctx.mount))
    } else {
        ApiResponse::redirect(format!("{}/login", ctx.mount))
    }
}

/// UI guard for the dashboard and other non-resource pages: `Some(redirect)`
/// when auth is configured and the visitor is unauthenticated or MFA-pending.
pub fn guard_ui(ctx: &ReqCtx) -> Option<ApiResponse> {
    if is_configured() && (ctx.claims.is_none() || mfa_pending(ctx)) {
        Some(login_redirect(ctx))
    } else {
        None
    }
}

// ===== Login / logout handlers =====

/// Message shown when a form post fails the CSRF check. Deliberately vague and
/// identical for every cause: a forged post learns nothing, and the benign cause
/// (a token that expired with the browser session) is fixed by simply retrying.
const CSRF_ERROR: &str = "Your session expired. Please try again.";

/// Shown when an account has burned through its attempts. Says nothing about
/// whether the account exists or the password was close.
const THROTTLED_ERROR: &str = "Too many attempts. Please wait a few minutes and try again.";

/// Throttle key for password attempts. Lower-cased so `Admin@x.io` can't be used
/// to open a fresh budget against `admin@x.io`.
fn login_key(email: &str) -> String {
    format!("login:{}", email.to_lowercase())
}

/// Throttle key for second-factor attempts, keyed by the session's account.
fn mfa_key(email: &str) -> String {
    format!("mfa:{}", email.to_lowercase())
}

/// Render the login page (optionally with an error message).
pub fn login_page(ctx: &ReqCtx, error: Option<&str>) -> ApiResponse {
    let mut c = ui::base_context(ctx, "Sign in");
    // Login has no sidebar menus; keep it minimal.
    c.insert("menus", &Vec::<crate::menu::MenuItem>::new());
    if let Some(e) = error {
        c.insert("error", e);
    }
    ui::render_with_csrf(ctx, c, "login.html")
}

/// Validate credentials against the admin table and, on success, redirect to the
/// dashboard with the auth cookie set. `csrf` is the submitted hidden field; it
/// must match the CSRF cookie or the post is rejected before any lookup.
pub async fn handle_login(
    ctx: &ReqCtx,
    email: &str,
    password: &str,
    csrf: Option<&str>,
) -> ApiResponse {
    // Checked first: an unauthenticated endpoint gets no `SameSite` protection,
    // so this is the only thing standing between a forged post and a login. It
    // also keeps forged traffic off the database.
    if !crate::csrf::verify(ctx, csrf) {
        let mut resp = login_page(ctx, Some(CSRF_ERROR));
        resp.status = 403;
        return resp;
    }

    let cfg = match config() {
        Some(c) => c,
        None => return CoreError::Internal("auth not configured".into()).into(),
    };

    // Throttle before the lookup, so a throttled attacker can't even use this to
    // probe which emails exist (and doesn't get to spend a bcrypt verify).
    let key = login_key(email);
    if let Some(limit) = ratelimit::login_limit() {
        if ratelimit::is_limited(&key, limit) {
            let mut resp = login_page(ctx, Some(THROTTLED_ERROR));
            resp.status = 429;
            return resp;
        }
    }

    // A miss counts against the limit exactly like a wrong password: if only real
    // accounts were throttled, the difference would enumerate them.
    let fail = |ctx: &ReqCtx| {
        if let Some(limit) = ratelimit::login_limit() {
            ratelimit::record_failure(&key, limit);
        }
        login_page(ctx, Some("Invalid email or password"))
    };

    let user = match storage().find_one_by(&cfg.admin_table, "email", email).await {
        Ok(Some(u)) => u,
        Ok(None) => return fail(ctx),
        Err(e) => return CoreError::from(e).into(),
    };

    let hash = user
        .get("encrypted_password")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !verify_password(password, hash) {
        return fail(ctx);
    }
    ratelimit::reset(&key);

    let role = user.get("role").and_then(|v| v.as_str()).unwrap_or("admin");
    let sub = json_id(&user).unwrap_or_else(|| email.to_string());

    // MFA-enabled users get a pending session and must clear the second factor
    // before reaching the panel. Others are logged straight in, then nudged to
    // the (skippable) setup page.
    if mfa_is_enabled(&user) {
        match issue_token(&sub, email, role, MFA_PENDING) {
            Some(token) => ApiResponse::redirect(format!("{}/mfa/verify", ctx.mount))
                .with_header("Set-Cookie", set_cookie_value(&token)),
            None => CoreError::Internal("failed to issue token".into()).into(),
        }
    } else {
        match issue_token(&sub, email, role, MFA_OK) {
            Some(token) => ApiResponse::redirect(format!("{}/mfa/setup", ctx.mount))
                .with_header("Set-Cookie", set_cookie_value(&token)),
            None => CoreError::Internal("failed to issue token".into()).into(),
        }
    }
}

// ===== MFA (TOTP) flow =====

/// Read the string `id` of a user row (unwrapping JSON strings/numbers).
fn json_id(user: &serde_json::Value) -> Option<String> {
    match user.get("id")? {
        serde_json::Value::String(s) => Some(s.clone()),
        other => Some(other.to_string()),
    }
}

/// Loosely interpret an `mfa_enabled` column (bool, 0/1, or "true"/"1").
fn mfa_is_enabled(user: &serde_json::Value) -> bool {
    match user.get("mfa_enabled") {
        Some(serde_json::Value::Bool(b)) => *b,
        Some(serde_json::Value::Number(n)) => n.as_i64().map(|i| i != 0).unwrap_or(false),
        Some(serde_json::Value::String(s)) => matches!(s.as_str(), "true" | "1" | "t"),
        _ => false,
    }
}

fn nonempty_str<'a>(user: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    user.get(key)?.as_str().filter(|s| !s.is_empty())
}

/// Look up the current session's user row by the email in its claims.
async fn current_user(ctx: &ReqCtx) -> Option<serde_json::Value> {
    let cfg = config()?;
    let email = ctx.claims.as_ref()?.email.clone();
    storage()
        .find_one_by(&cfg.admin_table, "email", &email)
        .await
        .ok()
        .flatten()
}

/// Persist a partial update to the current user's row.
async fn update_user(id: &str, patch: serde_json::Map<String, serde_json::Value>) -> Result<(), CoreError> {
    let cfg = config().ok_or_else(|| CoreError::Internal("auth not configured".into()))?;
    storage()
        .update(&cfg.admin_table, "id", id, patch)
        .await
        .map(|_| ())
        .map_err(CoreError::from)
}

/// Issue a fully-authenticated token and redirect to `dest`, setting the cookie.
fn authed_redirect(sub: &str, email: &str, role: &str, dest: String) -> ApiResponse {
    match issue_token(sub, email, role, MFA_OK) {
        Some(token) => ApiResponse::redirect(dest).with_header("Set-Cookie", set_cookie_value(&token)),
        None => CoreError::Internal("failed to issue token".into()).into(),
    }
}

/// Setup page: shows a QR + secret and asks the user to confirm a code to enable
/// MFA. Skippable — the "Skip for now" link just goes back to the dashboard.
pub async fn mfa_setup_page(ctx: &ReqCtx, error: Option<&str>) -> ApiResponse {
    if is_configured() && ctx.claims.is_none() {
        return ApiResponse::redirect(format!("{}/login", ctx.mount));
    }
    let user = match current_user(ctx).await {
        Some(u) => u,
        None => return ApiResponse::redirect(format!("{}/login", ctx.mount)),
    };
    // Already enabled -> nothing to set up.
    if mfa_is_enabled(&user) {
        return ApiResponse::redirect(ctx.mount.clone());
    }

    // Reuse a previously-generated (but not-yet-enabled) secret across refreshes,
    // otherwise the QR would change on every load.
    let secret = match nonempty_str(&user, "mfa_secret") {
        Some(s) => s.to_string(),
        None => {
            let s = crate::mfa::generate_secret();
            let Some(id) = json_id(&user) else {
                return CoreError::Internal("user row has no id".into()).into();
            };
            if let Err(e) = update_user(&id, patch(&[("mfa_secret", s.clone().into())])).await {
                return e.into();
            }
            s
        }
    };

    let email = ctx.claims.as_ref().map(|c| c.email.as_str()).unwrap_or("");
    let url = match crate::mfa::provisioning_url(&secret, email) {
        Ok(u) => u,
        Err(e) => return e.into(),
    };
    let qr = match crate::mfa::qr_svg(&url) {
        Ok(s) => s,
        Err(e) => return e.into(),
    };

    // Rendered inside the panel shell (layout.html), so keep the real sidebar menus.
    let mut c = ui::base_context(ctx, "Set up two-factor auth");
    c.insert("qr_svg", &qr);
    c.insert("secret", &secret);
    if let Some(e) = error {
        c.insert("error", e);
    }
    ui::render_with_csrf(ctx, c, "mfa_setup.html")
}

/// Confirm the code, enable MFA, generate backup codes, and show them once.
pub async fn handle_mfa_enable(ctx: &ReqCtx, code: &str, csrf: Option<&str>) -> ApiResponse {
    if !crate::csrf::verify(ctx, csrf) {
        let mut resp = mfa_setup_page(ctx, Some(CSRF_ERROR)).await;
        resp.status = 403;
        return resp;
    }
    let user = match current_user(ctx).await {
        Some(u) => u,
        None => return ApiResponse::redirect(format!("{}/login", ctx.mount)),
    };
    let email = ctx.claims.as_ref().map(|c| c.email.clone()).unwrap_or_default();
    let secret = match nonempty_str(&user, "mfa_secret") {
        Some(s) => s.to_string(),
        None => return mfa_setup_page(ctx, Some("Setup expired, please rescan")).await,
    };
    if !crate::mfa::check_code(&secret, &email, code) {
        return mfa_setup_page(ctx, Some("That code didn't match. Try again.")).await;
    }

    let codes = crate::mfa::generate_backup_codes();
    let hashed = match crate::mfa::hash_backup_codes(&codes) {
        Ok(h) => h,
        Err(e) => return e.into(),
    };
    let Some(id) = json_id(&user) else {
        return CoreError::Internal("user row has no id".into()).into();
    };
    if let Err(e) = update_user(
        &id,
        patch(&[
            ("mfa_enabled", true.into()),
            ("mfa_backup_codes", hashed.into()),
        ]),
    )
    .await
    {
        return e.into();
    }

    // Refresh the cookie so the step is unambiguously "ok".
    let role = user.get("role").and_then(|v| v.as_str()).unwrap_or("admin");
    let token = issue_token(&id, &email, role, MFA_OK);

    // Rendered inside the panel shell (layout.html), so keep the real sidebar menus.
    let mut c = ui::base_context(ctx, "Save your backup codes");
    c.insert("backup_codes", &codes);
    let resp = ui::render("mfa_backup.html", &c);
    match token {
        Some(t) => resp.with_header("Set-Cookie", set_cookie_value(&t)),
        None => resp,
    }
}

/// Challenge page for an MFA-enabled user mid-login.
pub async fn mfa_verify_page(ctx: &ReqCtx, error: Option<&str>) -> ApiResponse {
    if is_configured() && ctx.claims.is_none() {
        return ApiResponse::redirect(format!("{}/login", ctx.mount));
    }
    // Already cleared -> straight to the panel.
    if matches!(&ctx.claims, Some(c) if c.mfa == MFA_OK) {
        return ApiResponse::redirect(ctx.mount.clone());
    }
    let mut c = ui::base_context(ctx, "Two-factor verification");
    c.insert("menus", &Vec::<crate::menu::MenuItem>::new());
    if let Some(e) = error {
        c.insert("error", e);
    }
    ui::render_with_csrf(ctx, c, "mfa_verify.html")
}

/// Verify a TOTP or one-time backup code and, on success, upgrade the session.
pub async fn handle_mfa_verify(ctx: &ReqCtx, code: &str, csrf: Option<&str>) -> ApiResponse {
    if !crate::csrf::verify(ctx, csrf) {
        let mut resp = mfa_verify_page(ctx, Some(CSRF_ERROR)).await;
        resp.status = 403;
        return resp;
    }
    let user = match current_user(ctx).await {
        Some(u) => u,
        None => return ApiResponse::redirect(format!("{}/login", ctx.mount)),
    };
    let email = ctx.claims.as_ref().map(|c| c.email.clone()).unwrap_or_default();
    let role = user.get("role").and_then(|v| v.as_str()).unwrap_or("admin");
    let Some(id) = json_id(&user) else {
        return CoreError::Internal("user row has no id".into()).into();
    };

    // A 6-digit code is only 10^6 possibilities, so this throttle — not the code
    // itself — is what makes the second factor hold up.
    let key = mfa_key(&email);
    let limit = ratelimit::mfa_limit();
    if let Some(l) = limit {
        if ratelimit::is_limited(&key, l) {
            let mut resp = mfa_verify_page(ctx, Some(THROTTLED_ERROR)).await;
            resp.status = 429;
            return resp;
        }
    }

    // 1) Try the authenticator code.
    if let Some(secret) = nonempty_str(&user, "mfa_secret") {
        if crate::mfa::check_code(secret, &email, code) {
            ratelimit::reset(&key);
            return authed_redirect(&id, &email, role, ctx.mount.clone());
        }
    }

    // 2) Fall back to a one-time backup code, consuming it on success.
    if let Some(stored) = nonempty_str(&user, "mfa_backup_codes") {
        if let Some(remaining) = crate::mfa::consume_backup_code(stored, code) {
            if let Err(e) = update_user(&id, patch(&[("mfa_backup_codes", remaining.into())])).await {
                return e.into();
            }
            ratelimit::reset(&key);
            return authed_redirect(&id, &email, role, ctx.mount.clone());
        }
    }

    if let Some(l) = limit {
        ratelimit::record_failure(&key, l);
    }
    mfa_verify_page(ctx, Some("Invalid code. Try again or use a backup code.")).await
}

/// Build a small update map from `(column, value)` pairs.
fn patch(pairs: &[(&str, serde_json::Value)]) -> serde_json::Map<String, serde_json::Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

/// Clear the auth cookie and return to the login page.
pub fn handle_logout(ctx: &ReqCtx) -> ApiResponse {
    ApiResponse::redirect(format!("{}/login", ctx.mount))
        .with_header("Set-Cookie", clear_cookie_value())
}

/// Convenience for seeding: insert an admin user (bcrypt-hashing the password).
pub async fn create_admin(email: &str, password: &str, role: &str) -> Result<(), CoreError> {
    let cfg = config().ok_or_else(|| CoreError::Internal("auth not configured".into()))?;
    let mut data = serde_json::Map::new();
    data.insert("email".into(), serde_json::Value::String(email.into()));
    data.insert(
        "encrypted_password".into(),
        serde_json::Value::String(hash_password(password)?),
    );
    data.insert("role".into(), serde_json::Value::String(role.into()));
    storage()
        .create(&cfg.admin_table, data)
        .await
        .map_err(CoreError::from)?;
    Ok(())
}
