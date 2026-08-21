# Troubleshooting Guide

This guide covers common issues and solutions when working with actix-web-admin.

## Common Issues

### 1. "Cannot start a runtime from within a runtime"

**Problem:** Using CLI functions (`cli::create_superuser_interactive`, `cli::delete_user`, `cli::list_users`) inside an existing Tokio runtime panics with:

```
Cannot start a runtime from within a runtime.
```

**Root cause:** Older versions of the CLI functions created their own `tokio::runtime::Runtime::new()` internally. When called from `#[actix_web::main]` or `#[tokio::main]`, this creates a nested runtime which Tokio forbids.

**Solution:** CLI functions are now `async` — call them with `.await` from within your runtime:

```rust
// ✅ Correct — call async functions with .await
"deleteuser" => {
    cli::delete_user(&args[2..]).await.unwrap();
}

// ❌ Wrong — don't create your own runtime
"deleteuser" => {
    // Never do this:
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(cli::delete_user(&args[2..]))
}
```

Both `#[tokio::main]` and `#[actix_web::main]` work:

```rust
#[tokio::main]  // or #[actix_web::main]
async fn main() {
    cli::create_superuser_interactive(&args[2..]).await.unwrap();
}
```

### 2. Route Not Found (404 errors)

**Problem:** URLs with trailing slashes don't work.

**Solution:** The library configures both variants, but if extending routes:

```rust
.route("/my-route", web::get().to(my_handler))
.route("/my-route/", web::get().to(my_handler))
```

### 3. Double Slash in URLs

**Problem:** Seeing `//admin/login` or `//admin/logout`.

**Root cause:** The prefix was stored as `"/admin"` (with leading slash), so `format!("/{}/login", "admin")` produced `"//admin/login"`.

**Solution:** This is fixed. The `AdminPrefix` now stores `"admin"` (no leading slash). If you construct URLs manually, use:

```rust
format!("/{}/login", prefix.0)   // ✅ "/admin/login"
format!("/{}/", prefix.0)        // ✅ "/admin/"
```

### 4. Template Variables Undefined

**Problem:** Tera template errors about undefined variables.

**Solution:** Always provide default values in templates:

```jinja2
{{ query.search | default(value='') }}
{{ query.sort_dir | default(value='asc') }}
{{ query.page | default(value=1) }}
```

### 5. FormField::textarea Compilation Error

**Problem:** `FormField::textarea()` missing required parameter.

**Solution:** Include the rows parameter:

```rust
// ✅ Correct — include rows (number of visible lines)
FormField::textarea("description", "Description", 4)
```

### 6. Tera Template Filter Errors

**Problem:** Tera doesn't have all filters you might expect.

**Common issues:**

- **`range` filter doesn't exist:**
```jinja2
{# ✅ Use manual pagination instead #}
{% if page > 1 %}<a href="?page={{ page - 1 }}">Prev</a>{% endif %}
{% if page < total_pages %}<a href="?page={{ page + 1 }}">Next</a>{% endif %}
```

- **Arithmetic operations need intermediate variables:**
```jinja2
{% set col_count = columns | length %}
<td colspan="{{ col_count + 1 }}">
```

### 7. AdminRegistry Clone Error

**Problem:** `AdminRegistry` doesn't implement Clone.

**Solution:** Create registry inside HttpServer closure:

```rust
// ✅ Create registry inside closure
HttpServer::new(move || {
    let mut registry = AdminRegistry::new();
    registry.register(ProductAdmin::new(database.clone()));

    App::new()
        .configure(|cfg| {
            AdminSite::new("/admin").mount(cfg, registry)
        })
})
```

### 8. Session Errors

**Problem:** Session insertion fails or authentication doesn't work.

**Solution:** Ensure session middleware is properly configured:

```rust
.wrap(SessionMiddleware::new(
    CookieSessionStore::default(),
    actix_web::cookie::Key::generate()
))
```

### 9. User Already Exists

**Problem:** `store.create_user(...)` panics on second run because the user already exists.

**Solution:** Check before creating:

```rust
if store.find_by_username("admin").await.unwrap().is_none() {
    store.create_user("admin", "admin@example.com", "Admin", "admin", true).await.unwrap();
}
```

### 10. `all_users()` Returns Synchronously

**Problem:** The `UserStore` trait has a synchronous `all_users()` method.

**Solution:** If using an async database, cache users or use `std::sync::Mutex`:

```rust
fn all_users(&self) -> Result<Vec<User>, AuthError> {
    // For async DBs, maintain a cached list or use block_on sparingly
    let users = self.cached_users.lock().unwrap().clone();
    Ok(users)
}
```

## Debug Mode

Enable logging to debug issues:

```rust
env_logger::init();

// Add middleware to see requests
.wrap(middleware::Logger::default())
```

## Getting Help

If you encounter issues not covered here:

1. Check the [examples](../examples/) directory
2. Review the [quick start guide](quick-start.md)
3. Run `RUST_BACKTRACE=1 cargo run` for full backtraces
4. Enable debug logging to identify the problem

## Architecture Notes

### Routing Philosophy

This library follows Actix-web's routing philosophy:
- Routes are explicit, not magical
- Slash handling is manual by design
- URL construction follows framework conventions

This approach provides:
- Better performance
- Clearer routing logic
- Compatibility with Actix-web patterns
- No hidden magic behavior
