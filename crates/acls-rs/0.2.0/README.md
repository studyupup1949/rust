# acls-rs

Access control primitives for Rust — permission sets, role-based access
control (RBAC), and temporal permissions with time-based validity windows.

## What it does

- **Permission sets** with union, intersection, and difference — composition
  order does not affect the result
- **Grant/deny pairs** — explicit denials compose correctly with grants
- **RBAC** — role hierarchies with inheritance and cycle detection
- **Temporal permissions** — validity windows that expire automatically
- **Subjects** — represent users or entities with group memberships
- **PermissionMapping** — configurable bitmask-to-permission translation
  for bridging domain-specific access control models
- Optional **serde** support (enable the `serde` feature)

## Quick start

```rust
use acls_rs::prelude::*;

// Define permissions with namespace:action pairs
let viewer_perms = PermissionSet::from([
    AtomicPermission::new("file", "read"),
]);
let editor_perms = PermissionSet::from([
    AtomicPermission::new("file", "read"),
    AtomicPermission::new("file", "write"),
]);

// Build roles with grant/denial pairs
let mut rbac = RbacPolicy::new();
rbac.add_role(Role::new("viewer", GrantDenialPair::new(
    viewer_perms, PermissionSet::new(),
)));
rbac.add_role(Role::new("editor", GrantDenialPair::new(
    editor_perms, PermissionSet::new(),
)).with_parent("viewer"));

// Resolve effective permissions
let roles = vec!["editor".to_string()];
let perms = rbac.resolve_permissions(&roles).unwrap();
let effective = perms.effective_permissions();
assert!(effective.contains(&AtomicPermission::new("file", "read")));
assert!(effective.contains(&AtomicPermission::new("file", "write")));
```

## Documentation

Full API reference and design guide:
<https://akamu.dev/bac-rules/>

## License

Licensed under either of Apache License 2.0 or MIT license, at your option.
