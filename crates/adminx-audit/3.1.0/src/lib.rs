// adminx-audit/src/lib.rs
//
// Audit logging for adminx. Register it and every create / update / delete that
// goes through the default `Resource` CRUD is recorded with who did it and a
// per-column before/after diff; leave it out and adminx behaves exactly as it
// did, issuing not one extra query.
//
// Storage-agnostic: rows are written through adminx-core's `Storage` trait, so
// the same crate works over SeaORM (SQL) or MongoDB. The one asymmetry is table
// creation — see `migrate_sql`.
//
// ## Startup order
//
// ```ignore
// adminx_seaorm::init(&db_url).await?;                   // 1. storage
// adminx_core::seed(adminx_audit::migrate_sql()).await?; // 2. SQL table (SQL backends only)
// adminx_audit::init(AuditConfig::default());            // 3. register the auditor
// configure_auth(AuthConfig { /* ... */ });              // 4. turn auth on
// register_resource(Box::new(MyResource));
// adminx_audit::register_resources();                    // 5. the in-panel viewer (optional)
// ```
//
// ## What is and isn't recorded
//
// The hook lives on the `Resource` trait's default `create` / `update` /
// `delete`, because only that layer holds the `ReqCtx` that identifies the
// actor. A resource that *extends* the defaults keeps its recording as long as
// it delegates to `adminx_core::crud::{create, update, delete}` rather than
// copying the body — `adminx-rbac`'s `PermissionResource` does exactly that. A
// resource that replaces them outright records nothing unless it emits its own
// entry; see [`adminx_core::audit::emit`].
//
// Writes that bypass adminx entirely (a migration, psql, another service) are
// invisible to this crate by construction. If you need those too, the answer is
// database triggers, not an application-level log.

mod resource;
mod schema;
mod store;

pub use resource::AuditVersionResource;
pub use store::{StorageAuditor, TABLE};

/// How the auditor behaves when it cannot write.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditConfig {
    /// When `true`, a failed audit write turns the request into a 500 instead of
    /// only logging the error.
    ///
    /// Note what this does *not* buy you: the mutation has already committed by
    /// then, and `Storage` exposes no transaction spanning both writes, so a
    /// strict failure reports an error for a change that did land. It makes a
    /// hole in the log loud rather than silent — it is not atomicity. Leave it
    /// `false` (the default) unless a compliance regime prefers a visible
    /// failure to a missing entry.
    pub strict: bool,
}

impl AuditConfig {
    /// Fail the request when an entry cannot be recorded. See the caveat on
    /// [`AuditConfig::strict`].
    pub fn strict() -> Self {
        Self { strict: true }
    }
}

/// Register the auditor with adminx-core. Call after storage is set (and, on a
/// SQL backend, after running [`migrate_sql`]).
///
/// Set-once, matching `set_storage` / `set_authorizer`: a second call is ignored
/// with a warning.
pub fn init(config: AuditConfig) {
    adminx_core::set_auditor(Box::new(StorageAuditor::new(config.strict)));
    tracing::info!(
        "adminx-audit: recording to `{TABLE}` (strict={})",
        config.strict
    );
}

/// SQL `CREATE TABLE IF NOT EXISTS` + index statements for the audit table. Run
/// once on a SQL backend via `adminx_core::seed(adminx_audit::migrate_sql())`.
/// Mongo needs nothing (collections auto-create).
pub fn migrate_sql() -> &'static [&'static str] {
    schema::SQL
}

/// Register the read-only in-panel log viewer at `/adminx/adminx-audit-versions/list`.
/// Optional — omit it to keep the log out of the UI and query it directly.
pub fn register_resources() {
    adminx_core::register_resource(Box::new(AuditVersionResource));
}
