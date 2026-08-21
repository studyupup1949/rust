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
- Optional **serde** support (enable the `serde` feature)
- Minimal dependencies (`log`, `thiserror`)

## Quick start

```rust
use acls_rs::prelude::*;

// Define roles with inheritance
let mut rbac = RbacPolicy::new();

let viewer = PermissionSet::from_iter([AtomicPermission::new("read")]);
let editor = PermissionSet::from_iter([
    AtomicPermission::new("read"),
    AtomicPermission::new("write"),
]);

rbac.add_role(Role::new("viewer", viewer));
rbac.add_role(Role::new("editor", editor).inherits("viewer"));

// Resolve effective permissions for a role
let perms = rbac.resolve_permissions("editor").unwrap();
assert!(perms.contains(&AtomicPermission::new("read")));
assert!(perms.contains(&AtomicPermission::new("write")));
```

## Documentation

Full API reference and design guide:
<https://akamu.dev/bac-rules/>

## License

Licensed under either of Apache License 2.0 or MIT license, at your option.
