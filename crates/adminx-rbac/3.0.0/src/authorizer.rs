// adminx-rbac/src/authorizer.rs
//
// The DB-backed authorizer. Grants live in `adminx_permissions`, but the
// authorization decision is on the request hot path (called several times per
// page) and adminx-core's `Authorizer::can` is synchronous. So we load every
// grant into an in-memory cache once (and on `reload`), and `can` answers from
// the cache with no I/O — the storage async work is confined to `reload`.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use adminx_core::authz::{Action, Authorizer};
use adminx_core::storage::{storage, QueryOptions, StorageError};
use serde_json::Value;

use crate::ability::MANAGE;

/// Actions granted on one resource key for one role.
#[derive(Debug, Clone)]
enum ActionSet {
    /// A `manage` grant — every action.
    All,
    /// A specific set of action tokens.
    Only(HashSet<String>),
}

impl ActionSet {
    fn allows(&self, action: &str) -> bool {
        match self {
            ActionSet::All => true,
            ActionSet::Only(set) => set.contains(action),
        }
    }
}

/// role -> (resource-key -> actions). The resource key is a `base_path()` or the
/// `"*"` wildcard.
type Grants = HashMap<String, HashMap<String, ActionSet>>;

/// The registered authorization backend. Cheap to clone — the cache is shared
/// (`Arc`), so the instance handed to `set_authorizer` and the one kept for
/// `reload` see the same data.
#[derive(Clone, Default)]
pub struct DbAuthorizer {
    cache: Arc<RwLock<Grants>>,
}

impl DbAuthorizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Re-read every grant from `adminx_permissions` into the cache. Async (the
    /// only place this backend touches storage); call at startup and after any
    /// edit to the permission table.
    pub async fn reload(&self) -> Result<(), StorageError> {
        let rows = load_all("adminx_permissions").await?;
        let mut grants: Grants = HashMap::new();
        for row in &rows {
            let (Some(role), Some(resource), Some(action)) = (
                str_field(row, "role"),
                str_field(row, "resource"),
                str_field(row, "action"),
            ) else {
                tracing::warn!("adminx-rbac: skipping permission row missing a field: {row}");
                continue;
            };
            // Treat "*" as a synonym for "manage" (any action).
            let per_resource = grants.entry(role).or_default();
            let entry = per_resource
                .entry(resource)
                .or_insert_with(|| ActionSet::Only(HashSet::new()));
            if action == MANAGE || action == "*" {
                *entry = ActionSet::All;
            } else if let ActionSet::Only(set) = entry {
                set.insert(action);
            }
            // (if the entry is already `All`, a specific grant adds nothing)
        }
        *self.cache.write().unwrap_or_else(|e| e.into_inner()) = grants;
        tracing::info!(
            "adminx-rbac: loaded {} grant row(s) across {} role(s)",
            rows.len(),
            self.cache.read().unwrap_or_else(|e| e.into_inner()).len()
        );
        Ok(())
    }
}

impl Authorizer for DbAuthorizer {
    fn can(&self, roles: &[String], resource: &str, action: &Action<'_>) -> bool {
        let action = action.as_str();
        let grants = self.cache.read().unwrap_or_else(|e| e.into_inner());
        roles.iter().any(|role| {
            let Some(per_resource) = grants.get(role) else {
                return false;
            };
            // A grant on the exact resource or on the "*" wildcard both count.
            let exact = per_resource.get(resource).is_some_and(|s| s.allows(action));
            let wild = per_resource.get("*").is_some_and(|s| s.allows(action));
            exact || wild
        })
    }
}

/// Read every row of `table` by paging through `Storage::list`. The permission
/// table is tiny (tens–hundreds of rows), so a full scan into memory is the
/// simplest correct load — `can` then never touches the database.
async fn load_all(table: &str) -> Result<Vec<Value>, StorageError> {
    const PER_PAGE: u64 = 500;
    let mut out: Vec<Value> = Vec::new();
    let mut page = 1u64;
    loop {
        let opts = QueryOptions {
            page,
            per_page: PER_PAGE,
            sort_by: None,
            sort_desc: false,
            filters: Vec::new(),
        };
        let res = storage().list(table, &opts).await?;
        let fetched = res.rows.len() as u64;
        out.extend(res.rows);
        // Stop when we've collected the reported total, or a short/empty page
        // tells us there's no more (guards backends that under-report `total`).
        if out.len() as u64 >= res.total || fetched < PER_PAGE {
            break;
        }
        page += 1;
    }
    Ok(out)
}

/// Pull a non-empty string column out of a row (numbers/strings both handled).
fn str_field(row: &Value, key: &str) -> Option<String> {
    match row.get(key)? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[(&str, &str, &str)]) -> DbAuthorizer {
        // Build a cache directly, bypassing storage.
        let mut grants: Grants = HashMap::new();
        for (role, resource, action) in items {
            let per = grants.entry(role.to_string()).or_default();
            let entry = per
                .entry(resource.to_string())
                .or_insert_with(|| ActionSet::Only(HashSet::new()));
            if *action == MANAGE || *action == "*" {
                *entry = ActionSet::All;
            } else if let ActionSet::Only(s) = entry {
                s.insert(action.to_string());
            }
        }
        DbAuthorizer {
            cache: Arc::new(RwLock::new(grants)),
        }
    }

    fn roles(rs: &[&str]) -> Vec<String> {
        rs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn manage_all_allows_everything() {
        let a = set(&[("admin", "*", "manage")]);
        let r = roles(&["admin"]);
        for act in [
            Action::List,
            Action::Read,
            Action::Create,
            Action::Update,
            Action::Delete,
            Action::Export,
            Action::Custom("publish"),
        ] {
            assert!(a.can(&r, "posts", &act), "admin should do {act:?} on posts");
            assert!(a.can(&r, "anything", &act));
        }
    }

    #[test]
    fn scoped_grant_allows_only_that_tuple() {
        let a = set(&[("editor", "posts", "update")]);
        let r = roles(&["editor"]);
        assert!(a.can(&r, "posts", &Action::Update));
        assert!(!a.can(&r, "posts", &Action::Delete), "no delete grant");
        assert!(!a.can(&r, "comments", &Action::Update), "wrong resource");
        assert!(!a.can(&roles(&["viewer"]), "posts", &Action::Update), "unknown role");
    }

    #[test]
    fn custom_action_is_granted_by_name() {
        let a = set(&[("editor", "posts", "publish")]);
        let r = roles(&["editor"]);
        assert!(a.can(&r, "posts", &Action::Custom("publish")));
        assert!(!a.can(&r, "posts", &Action::Custom("archive")));
    }

    #[test]
    fn manage_on_one_resource_is_not_global() {
        let a = set(&[("editor", "posts", "manage")]);
        let r = roles(&["editor"]);
        assert!(a.can(&r, "posts", &Action::Delete));
        assert!(!a.can(&r, "users", &Action::Read), "manage is per-resource");
    }

    #[test]
    fn resource_wildcard_grant() {
        let a = set(&[("viewer", "*", "read")]);
        let r = roles(&["viewer"]);
        assert!(a.can(&r, "posts", &Action::Read));
        assert!(a.can(&r, "users", &Action::Read));
        assert!(!a.can(&r, "posts", &Action::Update), "read-only wildcard");
    }

    #[test]
    fn star_action_normalizes_to_manage() {
        let a = set(&[("admin", "posts", "*")]);
        assert!(a.can(&roles(&["admin"]), "posts", &Action::Delete));
    }

    #[test]
    fn union_across_roles() {
        let a = set(&[("editor", "posts", "update"), ("viewer", "*", "read")]);
        let both = roles(&["editor", "viewer"]);
        assert!(a.can(&both, "posts", &Action::Update), "from editor");
        assert!(a.can(&both, "users", &Action::Read), "from viewer");
        assert!(!a.can(&both, "users", &Action::Delete), "neither grants this");
    }

    #[test]
    fn empty_cache_denies() {
        let a = DbAuthorizer::new();
        assert!(!a.can(&roles(&["admin"]), "posts", &Action::Read));
    }
}
