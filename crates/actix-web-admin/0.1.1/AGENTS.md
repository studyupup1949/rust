# AGENTS.md

## Project Overview
- **Type**: Rust library for building admin panels with Actix-web 4
- **Entry point**: `src/lib.rs` exports `AdminSite`, `AdminRegistry`, `AdminResource`, `cli`
- **Version**: 0.1.0 ready for publication
- **Documentation**: Complete README with examples, troubleshooting guide, and advanced configuration
- **Status**: Fully tested with real-world implementation validation

## AI Assistant Skills & Capabilities

### 🤖 Development Skills
- **Rust Development**: Full crate architecture, async traits, error handling
- **Actix-web Integration**: Route configuration, middleware, session management
- **Template Engine**: Tera templates, embedded resources, context management
- **Testing**: Unit tests, integration tests, mock resources
- **Authentication**: UserStore trait, JsonUserStore, Argon2 hashing
- **CLI Tools**: Reusable `cli` module for user management binaries

### 🛠️ Common Tasks for AI Assistants

#### **Adding New Resources**
```rust
// Ask: "Add a User resource with email, password, role fields"
// AI will implement AdminResource trait with proper validation
```

#### **Custom UserStore Backend**
```rust
// Ask: "Implement UserStore for PostgreSQL"
// AI will create a UserStore impl using sqlx
```

#### **Authentication Enhancement**
```rust
// Ask: "Add OAuth2 authentication support"
// AI will implement new auth handlers and middleware
```

#### **Custom CLI Binary**
```rust
// Ask: "Create a custom CLI that uses my database"
// AI will write a thin binary using cli::* functions + your UserStore
```

### 📋 Project Commands
- `cargo build` — Build the library
- `cargo test` — Run all tests (23+ tests passing)
- `cargo test --test test_handlers` — Run integration tests
- `cargo run --bin admin-cli -- <command>` — Run CLI tool
- `cargo run --example memory` — Run the example app
- `cargo publish --dry-run` — Validate before publishing

### 🏗️ Architecture
- **AdminSite**: Configures the admin URL prefix, UserStore, title, and mounts routes
- **AdminRegistry**: Holds registered resources; order matters for UI
- **AdminResource**: Trait that resources implement to define CRUD operations
- **UserStore**: Trait for authentication backend (JsonUserStore included)
- **cli module**: Public async functions for user management (create, delete, list)
- **Templates**: Embedded at compile time via `include_str!` in `lib.rs`

### 🔧 Key Patterns
- Resources implement `AdminResource` trait (async)
- Authentication uses `UserStore` trait (not `SimpleAuth` which is deprecated)
- Slugs must be unique (panics on duplicate)
- Routes: `/{prefix}/`, `/{prefix}/login`, `/{prefix}/logout`, `/{prefix}/{slug}/`, etc.
- AdminPrefix stored without leading slash (e.g. `"admin"` not `"/admin"`) to avoid double-slash
- CLI functions are `async` — must be called from within a Tokio runtime
- Template variables: always provide defaults using `| default(value='')`

### 🧪 Testing Strategy
- Tests use mock resources implementing `AdminResource`
- Tests located in `tests/test_*.rs` and inside `src/` modules
- 23 tests total: 13 unit (auth/store, middleware, json_store), 4 handler integration, 3 registry, 2 resource, 1 doc-test
- All tests passing in release mode

### 📦 Published Modules
- `pub mod auth` — UserStore trait, JsonUserStore, middleware, password hashing
- `pub mod cli` — Reusable CLI functions (async)
- `pub mod resource` — AdminResource trait, types
- `pub mod registry` — AdminRegistry
- `pub mod site` — AdminSite
- `pub mod handlers` — Route handlers (login, logout, dashboard, CRUD)
- `pub mod types` — Shared types (Column, FormField, ListQuery, etc.)

### 🚀 AI-Enhanced Development Workflow

#### **For New Contributors**
1. **Ask AI**: "Explain the AdminResource trait and how to implement it"
2. **Ask AI**: "Create a new resource for [your model]"
3. **Ask AI**: "Add tests for my new resource"

#### **For Maintenance**
1. **Ask AI**: "Review code for performance optimizations"
2. **Ask AI**: "Update dependencies and check for breaking changes"
3. **Ask AI**: "Generate changelog for new version"

#### **For Feature Development**
1. **Ask AI**: "Design [feature] following current architecture"
2. **Ask AI**: "Implement [feature] with proper error handling"
3. **Ask AI**: "Add comprehensive tests for [feature]"

### 📦 Publishing & Deployment
- Ready for crates.io publication
- AGPL-3.0 License included
- Complete documentation with troubleshooting guide
- Version 0.1.0 stable
- `cargo publish --dry-run` validated successfully

### 🔍 Debugging Assistance
- **Template errors**: Check Tera syntax and context variables, use default values
- **Route errors**: Verify AdminSite configuration and slug uniqueness
- **CLI runtime errors**: Ensure async CLI functions are called with `.await` inside a Tokio runtime
- **Double-slash URLs**: AdminPrefix stored without leading slash now; rebuild if still seeing `//admin/...`
- **Build errors**: Check trait implementations and dependency versions
- **Common issues**: See `docs/troubleshooting.md` for comprehensive solutions to:
  - Cannot start runtime from within runtime (CLI async functions)
  - Actix-web routing with/without trailing slashes
  - Double slash URLs in redirects
  - Tera filter limitations (range, arithmetic)
  - AdminRegistry Clone requirements
  - FormField parameter requirements
  - User already exists on re-run
