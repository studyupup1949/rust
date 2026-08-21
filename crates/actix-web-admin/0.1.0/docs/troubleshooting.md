# Troubleshooting Guide

This guide covers common issues and solutions when working with actix-web-admin.

## Common Issues

### 1. Route Not Found (404 errors)

**Problem:** URLs with trailing slashes don't work.

**Solution:** Actix-web requires explicit configuration for both variants:

```rust
// The library automatically configures both, but if you're extending routes:
.route("/my-route", web::get().to(my_handler))
.route("/my-route/", web::get().to(my_handler))  // Add trailing slash version
```

### 2. Double Slash in URLs

**Problem:** Seeing URLs like `//admin/login` or `//admin/products`.

**Solution:** Use absolute paths instead of dynamic formatting:

```rust
// ❌ Incorrect - can cause double slashes
.insert_header(("Location", format!("/{}/login", prefix.0)))

// ✅ Correct - use absolute path
.insert_header(("Location", "/admin/login"))
```

### 3. Template Variables Undefined

**Problem:** Tera template errors about undefined variables.

**Solution:** Always provide default values in templates:

```jinja2
{{ query.search | default(value='') }}
{{ query.sort_dir | default(value='asc') }}
{{ query.page | default(value=1) }}
```

### 4. FormField::textarea Compilation Error

**Problem:** `FormField::textarea()` missing required parameter.

**Solution:** Include the rows parameter:

```rust
// ❌ Incorrect - missing rows parameter
FormField::textarea("description", "Description")

// ✅ Correct - include rows (number of visible lines)
FormField::textarea("description", "Description", 4)
```

### 5. Tera Template Filter Errors

**Problem:** Tera doesn't have all filters you might expect.

**Common issues:**

- **`range` filter doesn't exist:**
```jinja2
{# ❌ This will fail #}
{% for i in (1..5) | range %}

{# ✅ Use manual pagination instead #}
{% if page > 1 %}<a href="?page={{ page - 1 }}">Prev</a>{% endif %}
{% if page < total_pages %}<a href="?page={{ page + 1 }}">Next</a>{% endif %}
```

- **Arithmetic operations need intermediate variables:**
```jinja2
{# ❌ This will fail #}
<td colspan="{{ columns | length + 1 }}">

{# ✅ Use intermediate variable #}
{% set col_count = columns | length %}
<td colspan="{{ col_count + 1 }}">
```

### 6. AdminRegistry Clone Error

**Problem:** `AdminRegistry` doesn't implement Clone when used with HttpServer.

**Solution:** Create registry inside HttpServer closure:

```rust
// ❌ This will fail - AdminRegistry doesn't implement Clone
let registry = AdminRegistry::new();
registry.register(ProductAdmin::new());

HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(registry.clone()))  // Clone fails here
})

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

### 7. AdminError Variants

**Problem:** Not sure which AdminError variant to use.

**Solution:** Use appropriate error types:

```rust
use actix_web_admin::types::AdminError;

// For missing resources
Err(AdminError::NotFound)

// For database operations (if you implement this)
Err(AdminError::DatabaseError("Connection failed".to_string()))

// For validation errors
Err(AdminError::ValidationError("Invalid email format".to_string()))
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

### 7. Template Rendering Errors

**Problem:** Templates fail to render with Tera errors.

**Common solutions:**

- Check template syntax
- Ensure all variables exist in context
- Use proper filters for operations:

```jinja2
{% set col_count = columns | length %}
<td colspan="{{ col_count + 1 }}">
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
3. Enable debug logging to identify the problem
4. Ensure all dependencies are compatible versions

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
