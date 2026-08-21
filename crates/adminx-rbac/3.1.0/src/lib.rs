// adminx-rbac/src/lib.rs
//
// Pluggable, DB-backed RBAC for adminx. Register it and every resource route is
// gated by `(role, action, resource)` grants read from the database and cached
// in memory; leave it out and adminx uses its built-in per-resource role list.
//
// Storage-agnostic: grants are read and written through adminx-core's `Storage`
// trait, so the same crate works over SeaORM (SQL) or MongoDB. The one asymmetry
// is table creation — see `migrate_sql`.
//
// ## Startup order
//
// ```ignore
// adminx_seaorm::init(&db_url).await?;                  // 1. storage
// adminx_core::seed(adminx_rbac::migrate_sql()).await?; // 2. SQL tables (SQL backends only)
// adminx_rbac::init(vec![                               // 3. seed defaults + load cache + register
//     Ability::role("admin").can_manage_all(),
//     Ability::role("editor").can("update", "posts").can("publish", "posts"),
//     Ability::role("viewer").can_read_all(),
// ]).await?;
// configure_auth(AuthConfig { /* ... */ });             // 4. turn auth on
// register_resource(Box::new(MyResource));
// adminx_rbac::register_resources();                    // 5. roles/permissions editors (optional)
// ```

mod ability;
mod authorizer;
mod resources;
mod schema;

pub use ability::{Ability, ANY_RESOURCE, MANAGE};
pub use authorizer::DbAuthorizer;
pub use resources::{PermissionResource, RoleResource};

use adminx_core::storage::{storage, QueryOptions, StorageError};
use once_cell::sync::OnceCell;
use serde_json::{Map, Value};

/// Kept so `reload()` can reach the same cache that was registered as the global
/// authorizer.
static RBAC: OnceCell<DbAuthorizer> = OnceCell::new();

/// Seed the `abilities` into the DB if the permission table is empty, load all
/// grants into the cache, and register the authorizer with adminx-core. Call
/// after storage is set (and, on SQL, after running [`migrate_sql`]).
///
/// Idempotent-ish: seeding only happens when the table is empty, so restarts and
/// panel edits are preserved — the database is authoritative once populated.
pub async fn init(abilities: Vec<Ability>) -> Result<(), StorageError> {
    seed(&abilities).await?;
    let authz = DbAuthorizer::new();
    authz.reload().await?;
    // Ignore a second init: mirrors set_authorizer/set_storage's set-once policy.
    let _ = RBAC.set(authz.clone());
    adminx_core::set_authorizer(Box::new(authz));
    Ok(())
}

/// Re-read grants from the database into the cache. Called automatically after a
/// permission edit via the panel; also available for programmatic edits. A no-op
/// if [`init`] was never called.
pub async fn reload() -> Result<(), StorageError> {
    match RBAC.get() {
        Some(a) => a.reload().await,
        None => Ok(()),
    }
}

/// SQL `CREATE TABLE IF NOT EXISTS` statements for the RBAC tables. Run once on a
/// SQL backend via `adminx_core::seed(adminx_rbac::migrate_sql())`. Mongo needs
/// nothing (collections auto-create). See `schema` for the dialect note.
pub fn migrate_sql() -> &'static [&'static str] {
    schema::SQL
}

/// Register the in-panel editors for roles and permissions (admin-only). Optional
/// — omit it if you manage grants only in code/SQL.
pub fn register_resources() {
    adminx_core::register_resource(Box::new(RoleResource));
    adminx_core::register_resource(Box::new(PermissionResource));
}

/// Insert the ability block's grants only when `adminx_permissions` is empty, so
/// the code block bootstraps a fresh DB but never overwrites runtime edits.
async fn seed(abilities: &[Ability]) -> Result<(), StorageError> {
    let probe = QueryOptions {
        page: 1,
        per_page: 1,
        sort_by: None,
        sort_desc: false,
        filters: Vec::new(),
    };
    let existing = storage().list("adminx_permissions", &probe).await?;
    if existing.total > 0 || !existing.rows.is_empty() {
        tracing::info!(
            "adminx-rbac: {} permission row(s) already present; skipping seed",
            existing.total.max(existing.rows.len() as u64)
        );
        return Ok(());
    }

    for ab in abilities {
        // Role metadata is best-effort — a duplicate name just means the row is
        // already there; the grants below are what actually matter.
        let mut role_row = Map::new();
        role_row.insert("name".into(), Value::String(ab.role_name().to_string()));
        if let Err(e) = storage().create("adminx_roles", role_row).await {
            tracing::debug!("adminx-rbac: role '{}' not inserted ({e:?})", ab.role_name());
        }
        for row in ab.permission_rows() {
            storage().create("adminx_permissions", row).await?;
        }
    }
    tracing::info!("adminx-rbac: seeded default abilities for {} role(s)", abilities.len());
    Ok(())
}
