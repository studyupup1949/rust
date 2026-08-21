# AGENTS.md

## Project Overview
- **Type**: Rust library for building admin panels with Actix-web 4
- **Entry point**: `src/lib.rs` exports `AdminSite`, `AdminRegistry`, `AdminResource`
- **Version**: 0.1.0 ready for publication
- **Documentation**: Complete README with examples, troubleshooting guide, and advanced configuration
- **Status**: Fully tested with real-world implementation validation

## AI Assistant Skills & Capabilities

### 🤖 Development Skills
- **Rust Development**: Full crate architecture, async traits, error handling
- **Actix-web Integration**: Route configuration, middleware, session management
- **Template Engine**: Tera templates, embedded resources, context management
- **Testing**: Unit tests, integration tests, mock resources
- **Documentation**: README generation, API docs, examples

### 🛠️ Common Tasks for AI Assistants

#### **Adding New Resources**
```rust
// Ask: "Add a User resource with email, password, role fields"
// AI will implement AdminResource trait with proper validation
```

#### **Custom Field Types**
```rust
// Ask: "Add date picker and file upload field types"
// AI will extend FormField enum and add template rendering
```

#### **Authentication Enhancement**
```rust
// Ask: "Add OAuth2 authentication support"
// AI will implement new auth handlers and middleware
```

#### **Database Integration**
```rust
// Ask: "Add PostgreSQL integration with sqlx"
// AI will create database-backed resource implementations
```

### 📋 Project Commands
- `cargo build` - Build the library
- `cargo test` - Run all tests (7 tests passing)
- `cargo test --test test_registry` - Run specific test file
- `cargo run --example memory` - Run the example app
- `cargo publish --dry-run` - Validate before publishing
- `cargo publish` - Publish to crates.io

### 🏗️ Architecture
- **AdminSite**: Configures the admin URL prefix and mounts routes
- **AdminRegistry**: Holds registered resources; order matters for UI
- **AdminResource**: Trait that resources implement to define CRUD operations
- **Templates**: Embedded at compile time via `include_str!` in `lib.rs:15-24`

### 🔧 Key Patterns
- Resources must implement `AdminResource` trait (async)
- Slugs must be unique (panics on duplicate)
- Routes: `/{prefix}/{slug}`, `/{prefix}/{slug}/new`, `/{prefix}/{slug}/{id}`, `/{prefix}/{slug}/{id}/delete`
- JSON serialization for template context to avoid Tera serialization issues
- Actix-web routing philosophy: explicit slash handling, absolute URLs for redirects
- Template variables: always provide defaults using `| default(value='')`

### 🧪 Testing Strategy
- Tests use mock resources implementing `AdminResource`
- Tests located in `tests/test_*.rs`
- 7 tests total: 2 handlers, 3 registry, 2 resource tests
- All tests passing in release mode

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
- **Route errors**: Verify AdminSite configuration and slug uniqueness, check slash handling
- **Async errors**: Ensure proper .await usage and error propagation
- **Build errors**: Check trait implementations and dependency versions
- **Common issues**: See `docs/troubleshooting.md` for comprehensive solutions to:
  - Actix-web routing with/without trailing slashes
  - Double slash URLs in redirects
  - Tera filter limitations (range, arithmetic)
  - AdminRegistry Clone requirements
  - FormField parameter requirements