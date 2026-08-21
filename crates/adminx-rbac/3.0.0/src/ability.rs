// adminx-rbac/src/ability.rs
//
// The "central ability block": a code-level way to declare a role's grants,
// used to SEED the database on first boot. Once the permission table has rows
// the database is authoritative and admins edit it in the panel — so this block
// is a starting point, not a running config. That reconciles "define policy in
// code" with "edit policy at runtime without a redeploy".

use serde_json::{Map, Value};

/// Wildcard resource: a grant on `"*"` applies to every resource.
pub const ANY_RESOURCE: &str = "*";
/// Wildcard action: a grant of `"manage"` applies to every action on its resource.
pub const MANAGE: &str = "manage";

/// Grants for a single role. Build with [`Ability::role`] then chain `can*`.
///
/// ```ignore
/// Ability::role("editor")
///     .can("update", "posts")
///     .can("publish", "posts")   // a custom action, by name
///     .can_manage("comments");   // every action on comments
/// ```
#[derive(Debug, Clone)]
pub struct Ability {
    role: String,
    /// `(resource, action)` pairs. `resource` may be `"*"`, `action` may be `"manage"`.
    grants: Vec<(String, String)>,
}

impl Ability {
    /// Begin a grant set for `role` (the string stored in the admin user's
    /// `role` column and carried in the JWT).
    pub fn role(name: impl Into<String>) -> Self {
        Self {
            role: name.into(),
            grants: Vec::new(),
        }
    }

    /// Allow `action` on `resource`. `action` is a built-in token
    /// (`list`/`read`/`create`/`update`/`delete`/`export`) or a custom action's
    /// name; `resource` is a resource's `base_path()`.
    pub fn can(mut self, action: impl Into<String>, resource: impl Into<String>) -> Self {
        self.grants.push((resource.into(), action.into()));
        self
    }

    /// Allow every action on `resource`.
    pub fn can_manage(self, resource: impl Into<String>) -> Self {
        let resource = resource.into();
        self.can(MANAGE, resource)
    }

    /// Allow every action on every resource (the superuser grant).
    pub fn can_manage_all(self) -> Self {
        self.can(MANAGE, ANY_RESOURCE)
    }

    /// Allow read-only access everywhere: `list`, `read`, and `export` on every
    /// resource. Handy for a "viewer" role.
    pub fn can_read_all(self) -> Self {
        self.can("list", ANY_RESOURCE)
            .can("read", ANY_RESOURCE)
            .can("export", ANY_RESOURCE)
    }

    pub(crate) fn role_name(&self) -> &str {
        &self.role
    }

    /// One `{role, resource, action}` row per grant, for seeding
    /// `adminx_permissions`.
    pub(crate) fn permission_rows(&self) -> Vec<Map<String, Value>> {
        self.grants
            .iter()
            .map(|(resource, action)| {
                let mut m = Map::new();
                m.insert("role".into(), Value::String(self.role.clone()));
                m.insert("resource".into(), Value::String(resource.clone()));
                m.insert("action".into(), Value::String(action.clone()));
                m
            })
            .collect()
    }
}
