// adminx-core/src/authz.rs
//
// The authorization seam. Today's role check (a static per-resource role list)
// is one strategy; this module lets a crate like `adminx-rbac` plug in a richer
// one without adminx-core depending on it — the same pattern `storage` uses for
// pluggable backends.
//
// The action being performed is threaded in from each `Resource` method, so a
// backend can decide per operation ("editor may update but not delete") rather
// than per resource. With no backend registered, `authorize` falls back to the
// original role-list intersection, so behaviour is unchanged until a crate opts in.

use crate::request::ReqCtx;
use once_cell::sync::OnceCell;
use std::sync::Once;

/// The operation being authorized. Borrows so `Custom` carries a handler's
/// `&str` name without allocating; it's `Copy`, and the lifetime elides to
/// `Action<'_>` at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<'a> {
    List,
    Read,
    Create,
    Update,
    Delete,
    Export,
    /// A custom action, identified by its `CustomAction::name`.
    Custom(&'a str),
}

impl<'a> Action<'a> {
    /// The token an authorizer compares against (and a DB backend stores). Custom
    /// actions serialize to their literal name, so a `publish` action is the
    /// grant `"publish"`.
    pub fn as_str(&self) -> &'a str {
        match self {
            Action::List => "list",
            Action::Read => "read",
            Action::Create => "create",
            Action::Update => "update",
            Action::Delete => "delete",
            Action::Export => "export",
            Action::Custom(name) => name,
        }
    }
}

/// A pluggable authorization policy. `can` is synchronous and called several
/// times per request, so an implementation must not perform I/O here — a
/// DB-backed one reads a pre-loaded in-memory cache.
pub trait Authorizer: Send + Sync {
    /// Whether a principal holding `roles` may perform `action` on `resource`
    /// (the resource's `base_path()`).
    fn can(&self, roles: &[String], resource: &str, action: &Action<'_>) -> bool;
}

static AUTHORIZER: OnceCell<Box<dyn Authorizer>> = OnceCell::new();

/// Register the global authorization policy. Set-once: a later call is ignored
/// with a warning, matching `set_storage`.
pub fn set_authorizer(authorizer: Box<dyn Authorizer>) {
    if AUTHORIZER.set(authorizer).is_err() {
        tracing::warn!("adminx authorizer already initialized; ignoring reset");
    }
}

/// The registered authorizer, if any. `None` means the built-in role-list check
/// is used.
pub fn authorizer() -> Option<&'static dyn Authorizer> {
    AUTHORIZER.get().map(|b| b.as_ref())
}

/// The one place the access decision is made, so the invariants live together:
///
/// 1. Auth unconfigured ⇒ allow (the panel is public while prototyping).
/// 2. A password-verified but MFA-pending session never passes.
///
/// Then: consult the registered [`Authorizer`] if there is one, otherwise fall
/// back to the historical `allowed_roles ∩ ctx.roles()` intersection — so with
/// no authorizer registered this is byte-for-byte the old `is_authorized`.
/// Warn exactly once — not per request — that the panel is wide open.
static UNCONFIGURED_WARNED: Once = Once::new();

pub fn authorize(
    ctx: &ReqCtx,
    allowed_roles: &[String],
    resource: &str,
    action: Action<'_>,
) -> bool {
    if !crate::auth::is_configured() {
        // A real request reached an access check with auth off: the whole panel
        // (and API) is public. Fine while prototyping, dangerous in production —
        // say so loudly, once, so a forgotten `configure_auth` can't hide.
        UNCONFIGURED_WARNED.call_once(|| {
            tracing::warn!(
                "adminx: authentication is NOT configured — every page and API route is \
                 PUBLIC and RBAC is bypassed. Call `configure_auth(..)` (and seed an admin) \
                 to secure the panel. This warning is shown once."
            );
        });
        return true;
    }
    if crate::auth::mfa_pending(ctx) {
        return false;
    }
    let roles = ctx.roles();
    match authorizer() {
        Some(a) => a.can(&roles, resource, &action),
        None => allowed_roles.iter().any(|r| roles.contains(r)),
    }
}
