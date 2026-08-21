# Actix JWT Middleware

A full-featured JWT authentication middleware for [actix-web](https://github.com/actix/actix-web), built on top of [jsonwebtoken](https://github.com/Keats/jsonwebtoken).
Easily add login, token refresh, and authorization to your actix-web applications.

Based on [appleboy/gin-jwt](https://github.com/appleboy/gin-jwt) (ported from Gin to Echo to actix-web) with additional features adopted from [labstack/echo-jwt](https://github.com/labstack/echo-jwt).

---

## Table of Contents

- [Actix JWT Middleware](#actix-jwt-middleware)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Extra Features (from labstack/echo-jwt)](#extra-features-from-labstackecho-jwt)
  - [Security Notice](#security-notice)
    - [Critical Security Requirements](#critical-security-requirements)
    - [Production Security Checklist](#production-security-checklist)
    - [OAuth 2.0 Security Standards](#oauth-20-security-standards)
    - [Secure Configuration Example](#secure-configuration-example)
  - [Installation](#installation)
  - [Quick Start Example](#quick-start-example)
  - [Complete Examples](#complete-examples)
  - [Configuration](#configuration)
  - [JWT Parsing Options](#jwt-parsing-options)
    - [Clock Skew Tolerance (Leeway)](#clock-skew-tolerance-leeway)
    - [Other Parsing Options](#other-parsing-options)
  - [Supporting Multiple JWT Providers](#supporting-multiple-jwt-providers)
    - [Use Cases](#use-cases)
    - [Solution: Dynamic Key Function](#solution-dynamic-key-function)
    - [Key Considerations](#key-considerations)
    - [Additional Resources](#additional-resources)
  - [Token Generator (Direct Token Creation)](#token-generator-direct-token-creation)
    - [Basic Usage](#basic-usage)
    - [Token Structure](#token-structure)
    - [Refresh Token Management](#refresh-token-management)
  - [Redis Store Configuration](#redis-store-configuration)
    - [Redis Features](#redis-features)
    - [Configuration Options](#configuration-options)
    - [Fallback Behavior](#fallback-behavior)
    - [Example with Redis](#example-with-redis)
  - [Demo](#demo)
    - [Login](#login)
    - [Refresh Token](#refresh-token)
    - [Hello World](#hello-world)
    - [Authorization Example](#authorization-example)
  - [Understanding the Authorizer](#understanding-the-authorizer)
    - [How Authorizer Works](#how-authorizer-works)
    - [Authorizer Function Signature](#authorizer-function-signature)
    - [Basic Usage Examples](#basic-usage-examples)
    - [Advanced Authorization Patterns](#advanced-authorization-patterns)
    - [Setting Up Different Authorization for Different Routes](#setting-up-different-authorization-for-different-routes)
    - [Common Patterns and Best Practices](#common-patterns-and-best-practices)
    - [Complete Example](#complete-example)
    - [Logout](#logout)
  - [Cookie Token](#cookie-token)
    - [Refresh Token Cookie Support](#refresh-token-cookie-support)
    - [Login request flow (using login_handler)](#login-request-flow-using-login_handler)
    - [Subsequent requests on endpoints requiring jwt token (using middleware)](#subsequent-requests-on-endpoints-requiring-jwt-token-using-middleware)
    - [Logout request flow (using logout_handler)](#logout-request-flow-using-logout_handler)
    - [Refresh request flow (using refresh_handler)](#refresh-request-flow-using-refresh_handler)
    - [Failures with logging in, bad tokens, or lacking privileges](#failures-with-logging-in-bad-tokens-or-lacking-privileges)

---

## Features

- Simple JWT authentication for actix-web
- Built-in login, refresh, and logout handlers
- Customizable authentication, authorization, and claims
- Cookie and header token support
- Easy integration and clear API
- RFC 6749 compliant refresh tokens (OAuth 2.0 standard)
- Pluggable refresh token storage (in-memory, Redis via feature gate)
- Direct token generation without HTTP middleware
- Structured `Token` type with metadata

---

## Extra Features (from [labstack/echo-jwt](https://github.com/labstack/echo-jwt))

The following features are adopted from [labstack/echo-jwt](https://github.com/labstack/echo-jwt), which only provides token validation. This module combines them with the full gin-jwt feature set (login, refresh, logout, token store, etc.):

| Feature | Description |
|---------|-------------|
| **Skipper** | Skip middleware for specific routes (e.g. public endpoints) |
| **BeforeFunc** | Hook called before token extraction - useful for request preprocessing |
| **SuccessHandler** | Hook called after successful token validation |
| **ErrorHandler** | Custom error handler with access to the original error |
| **ContinueOnIgnoredError** | Continue to the next handler when `ErrorHandler` returns `None` - enables hybrid public/authenticated routes |
| **Typed errors** | `TokenParsing` and `TokenExtraction` variants on `JwtError` allow distinguishing between missing and invalid tokens |

### Example: Hybrid Public/Authenticated Route

```rust
let mut jwt = ActixJwtMiddleware::new();
// ... standard config ...
jwt.continue_on_ignored_error = true;
jwt.error_handler = Some(Arc::new(|req, err| {
    // No valid token - allow through as a public user
    None // returning None means "ignore the error, continue"
}));
```

### Example: Skipper

```rust
jwt.skipper = Some(Arc::new(|service_req| {
    service_req.path().starts_with("/public")
}));
```

---

## Security Notice

### Critical Security Requirements

> **JWT Secret Security**
>
> - **Minimum Requirements:** Use secrets of at least **256 bits (32 bytes)** in length
> - **Never use:** Simple passwords, dictionary words, or predictable patterns
> - **Recommended:** Generate cryptographically secure random secrets or use `RS256` algorithm
> - **Storage:** Store secrets in environment variables, never hardcode in source code
> - **Vulnerability:** Weak secrets are vulnerable to brute-force attacks ([jwt-cracker](https://github.com/lmammino/jwt-cracker))

### Production Security Checklist

- **HTTPS Only:** Always use HTTPS in production environments
- **Strong Secrets:** Minimum 256-bit randomly generated secrets
- **Token Expiry:** Set appropriate timeout values (recommended: 15-60 minutes for access tokens)
- **Secure Cookies:** Enable `secure_cookie`, `cookie_http_only`, and appropriate `cookie_same_site` settings
- **Environment Variables:** Store sensitive configuration in environment variables
- **Input Validation:** Validate all authentication inputs thoroughly

### OAuth 2.0 Security Standards

This library follows **RFC 6749 OAuth 2.0** security standards:

- **Separate Tokens:** Uses distinct opaque refresh tokens (not JWT) for enhanced security
- **Server-Side Storage:** Refresh tokens are stored and validated server-side
- **Token Rotation:** Refresh tokens are automatically rotated on each use
- **Improved Security:** Prevents JWT refresh token vulnerabilities and replay attacks

### Secure Configuration Example

```rust
use std::env;
use std::sync::Arc;
use std::time::Duration;
use actix_web::cookie::SameSite;
use actix_jwt::ActixJwtMiddleware;

// BAD: Weak secret, insecure settings
let mut bad_jwt = ActixJwtMiddleware::new();
bad_jwt.key = b"weak".to_vec();                    // Too short!
bad_jwt.timeout = Duration::from_secs(86400);       // Too long!
bad_jwt.secure_cookie = false;                      // Insecure in production!

// GOOD: Strong security configuration
let mut jwt = ActixJwtMiddleware::new();
jwt.key = env::var("JWT_SECRET")                    // From environment
    .expect("JWT_SECRET must be set")
    .into_bytes();
jwt.timeout = Duration::from_secs(900);             // 15-minute access tokens
jwt.max_refresh = Duration::from_secs(604800);      // 1 week refresh validity
jwt.secure_cookie = true;                           // HTTPS only
jwt.cookie_http_only = true;                        // Prevent XSS
jwt.cookie_same_site = SameSite::Strict;            // CSRF protection
jwt.send_cookie = true;                             // Enable secure cookies
```

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
actix-jwt = { git = "https://github.com/LdDl/actix-jwt" }
```

To enable Redis refresh token storage:

```toml
[dependencies]
actix-jwt = { git = "https://github.com/LdDl/actix-jwt", features = ["redis-store"] }
```

---

## Quick Start Example

Please see the [example file](examples/basic.rs) and you can use `extract_claims` to fetch user data.

```rust
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use actix_jwt::{extract_claims, get_identity, ActixJwtMiddleware, JwtError};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use serde::Deserialize;
use serde_json::{Value, json};

const IDENTITY_KEY: &str = "id";

#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let bind_addr = format!("0.0.0.0:{}", port);

    // Build the JWT middleware
    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "test zone".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(3600);
    jwt.identity_key = IDENTITY_KEY.to_string();
    jwt.token_lookup = "header:Authorization, query:token, cookie:jwt".to_string();
    jwt.token_head_name = "Bearer".to_string();

    // Authenticator: validate username/password, return user data as JSON Value
    jwt.authenticator = Some(Arc::new(|_req: &HttpRequest, body: &[u8]| {
        let login: LoginRequest =
            serde_json::from_slice(body).map_err(|_| JwtError::MissingLoginValues)?;

        if (login.username == "admin" && login.password == "admin")
            || (login.username == "test" && login.password == "test")
        {
            Ok(json!({
                "user_name": login.username,
                "first_name": "Wu",
                "last_name": "Bo-Yi",
            }))
        } else {
            Err(JwtError::FailedAuthentication)
        }
    }));

    // PayloadFunc: extract identity claims from user data returned by authenticator
    jwt.payload_func = Some(Arc::new(|data: &Value| {
        let mut claims = HashMap::new();
        if let Some(user_name) = data.get("user_name") {
            claims.insert(IDENTITY_KEY.to_string(), user_name.clone());
        }
        claims
    }));

    // IdentityHandler: reconstruct user data from JWT claims stored in request extensions
    jwt.identity_handler = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        claims.get(IDENTITY_KEY).cloned()
    });

    // Authorizer: only allow the "admin" user to access protected routes
    jwt.authorizer = Arc::new(|_req: &HttpRequest, data: &Value| {
        data.as_str() == Some("admin")
    });

    // Unauthorized response
    jwt.unauthorized = Arc::new(|_req: &HttpRequest, code: u16, message: &str| {
        HttpResponse::build(
            actix_web::http::StatusCode::from_u16(code)
                .unwrap_or(actix_web::http::StatusCode::UNAUTHORIZED),
        )
        .json(json!({
            "code": code,
            "message": message,
        }))
    });

    // Logout response: show logged-out user info
    jwt.logout_response = Arc::new(|req: &HttpRequest| {
        let claims = extract_claims(req);
        let identity = get_identity(req);

        let mut response = json!({
            "code": 200,
            "message": "Successfully logged out",
        });

        if let Some(user_id) = claims.get(IDENTITY_KEY) {
            response["logged_out_user"] = user_id.clone();
        }
        if let Some(user) = identity {
            response["user_info"] = user;
        }

        HttpResponse::Ok().json(response)
    });

    // Initialize (validates config and prepares keys)
    jwt.init().expect("JWT middleware init failed");

    let jwt_arc = Arc::new(jwt);

    println!("Listening on {}", bind_addr);

    HttpServer::new(move || {
        let jwt_data = web::Data::new(jwt_arc.clone());

        App::new()
            .app_data(jwt_data.clone())
            // Public routes
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            // Protected routes
            .service(
                web::scope("/auth")
                    .wrap(jwt_arc.middleware())
                    .route("/hello", web::get().to(hello))
                    .route("/logout", web::post().to(logout)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn login(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.login_handler(&req, &body).await
}

async fn refresh(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.refresh_handler(&req, &body).await
}

async fn logout(
    jwt: web::Data<Arc<ActixJwtMiddleware>>,
    req: HttpRequest,
    body: web::Bytes,
) -> HttpResponse {
    jwt.logout_handler(&req, &body).await
}

async fn hello(req: HttpRequest) -> HttpResponse {
    let claims = extract_claims(&req);
    let identity = get_identity(&req);

    HttpResponse::Ok().json(json!({
        "userID": claims.get(IDENTITY_KEY),
        "userName": identity,
        "text": "Hello World.",
    }))
}
```

---

## Complete Examples

This repository provides several complete example implementations demonstrating different use cases:

### [Basic Authentication](examples/basic.rs)

The basic example showing fundamental JWT authentication with login, protected routes, and token validation.

### [OAuth SSO Integration](examples/oauth_sso.rs)

**OAuth 2.0 Single Sign-On** conceptual example supporting multiple identity providers (Google, GitHub):

- OAuth 2.0 Authorization Code Flow pattern
- CSRF protection with state tokens
- Secure token delivery for both browser and mobile apps

### [Token Generator](examples/token_generator.rs)

Direct token generation without HTTP middleware, perfect for:

- Programmatic authentication
- Service-to-service communication
- Testing authenticated endpoints
- Custom authentication flows

### [Redis Store](examples/redis_simple.rs)

Demonstrates Redis integration for refresh token storage with:

- Automatic fallback to in-memory store
- Production-ready configuration examples

### [Redis Store (Explicit Config)](examples/redis_store.rs)

Explicit Redis configuration using `RedisConfig` struct with store info endpoint.

### [Redis TLS](examples/redis_tls.rs)

Redis store with TLS configuration for secure connections.

### [Authorization](examples/authorization.rs)

Advanced authorization patterns including:

- Role-based access control (admin/user/guest)
- Path-based authorization
- Multiple middleware instances
- Fine-grained permission control

---

## Configuration

The `ActixJwtMiddleware` struct provides the following configuration options:

| Option | Type | Required | Default | Description |
| ------ | ---- | -------- | ------- | ----------- |
| `realm` | `String` | No | `"actix jwt"` | Realm name to display to the user. |
| `signing_algorithm` | `String` | No | `"HS256"` | Signing algorithm (HS256, HS384, HS512, RS256, RS384, RS512). |
| `key` | `Vec<u8>` | Yes | - | Secret key used for signing. |
| `timeout` | `Duration` | No | `3600s` (1 hour) | Duration that a JWT access token is valid. |
| `max_refresh` | `Duration` | No | `0` | Duration that a refresh token is valid. |
| `authenticator` | `Fn(&HttpRequest, &[u8]) -> Result<Value, JwtError>` | Yes | - | Callback to authenticate the user. Returns user data as `serde_json::Value`. |
| `authorizer` | `Fn(&HttpRequest, &Value) -> bool` | No | `true` | Callback to authorize the authenticated user. |
| `payload_func` | `Fn(&Value) -> HashMap<String, Value>` | No | - | Callback to add additional payload data to the token. |
| `unauthorized` | `Fn(&HttpRequest, u16, &str) -> HttpResponse` | No | default JSON | Callback for unauthorized requests. |
| `login_response` | `Fn(&HttpRequest, &Token) -> HttpResponse` | No | default JSON | Callback for successful login response. |
| `logout_response` | `Fn(&HttpRequest) -> HttpResponse` | No | default JSON | Callback for successful logout response. |
| `refresh_response` | `Fn(&HttpRequest, &Token) -> HttpResponse` | No | default JSON | Callback for successful refresh response. |
| `identity_handler` | `Fn(&HttpRequest) -> Option<Value>` | No | - | Callback to retrieve identity from claims. |
| `identity_key` | `String` | No | `"identity"` | Key used to store identity in claims. |
| `token_lookup` | `String` | No | `"header:Authorization"` | Source to extract token from (header, query, cookie). |
| `token_head_name` | `String` | No | `"Bearer"` | Header name prefix. |
| `time_func` | `Fn() -> DateTime<Utc>` | No | `Utc::now` | Function to provide current time. |
| `priv_key_file` | `Option<String>` | No | `None` | Path to private key file (for RS algorithms). |
| `pub_key_file` | `Option<String>` | No | `None` | Path to public key file (for RS algorithms). |
| `send_cookie` | `bool` | No | `false` | Whether to send token as a cookie. |
| `cookie_max_age` | `Duration` | No | `timeout` | Duration that the cookie is valid. |
| `secure_cookie` | `bool` | No | `false` | Whether to use secure cookies for access token (HTTPS only). Refresh token cookies are always secure. |
| `cookie_http_only` | `bool` | No | `false` | Whether to use HTTPOnly cookies. |
| `cookie_domain` | `Option<String>` | No | `None` | Domain for the cookie. |
| `cookie_name` | `String` | No | `"jwt"` | Name of the cookie. |
| `refresh_token_cookie_name` | `String` | No | `"refresh_token"` | Name of the refresh token cookie. |
| `cookie_same_site` | `SameSite` | No | `SameSite::Lax` | SameSite attribute for the cookie. |
| `send_authorization` | `bool` | No | `false` | Whether to return authorization header for every request. |
| `key_func` | `Fn(&Header) -> Result<DecodingKey, JwtError>` | No | `None` | Dynamic key function for multi-provider JWT support. |
| `skipper` | `Fn(&ServiceRequest) -> bool` | No | `None` | Skip middleware for matching requests. |
| `before_func` | `Fn(&ServiceRequest)` | No | `None` | Hook called before token extraction. |
| `success_handler` | `Fn(&HttpRequest) -> Result<(), JwtError>` | No | `None` | Hook called after successful token validation. |
| `error_handler` | `Fn(&HttpRequest, JwtError) -> Option<JwtError>` | No | `None` | Custom error handler; return `None` to ignore the error. |
| `continue_on_ignored_error` | `bool` | No | `false` | Continue to next handler when `error_handler` returns `None`. |

---

## JWT Parsing Options

The `jsonwebtoken` crate's `Validation` struct controls JWT parsing behavior. While the middleware configures sensible defaults internally, you can customize validation by using `key_func` or by modifying the middleware source.

### Clock Skew Tolerance (Leeway)

When running distributed systems across multiple servers, clock synchronization issues can cause valid tokens to be rejected. The `jsonwebtoken` crate supports leeway via `Validation::leeway`:

#### When to Use Leeway

- **Microservices Architecture**: Services on different machines with slightly unsynchronized clocks
- **Cloud Deployments**: Distributed systems across different availability zones or regions
- **Load Balanced Environments**: Multiple backend servers with small time drift
- **Testing Environments**: Development/staging systems with less strict time synchronization

#### How Leeway Works

With a 60-second leeway configuration:

- **Expired tokens**: A token that expired 30 seconds ago will still be accepted
- **Not-before tokens**: A token with `nbf` 30 seconds in the future will be accepted

**Security Note**: Use reasonable leeway values (30-120 seconds). Excessive leeway reduces token security by extending validity beyond intended expiration times.

#### Validation Example

```rust
use jsonwebtoken::Validation;

let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
validation.leeway = 60; // 60 seconds clock skew tolerance
validation.validate_exp = true;
```

### Other Parsing Options

#### Required Claims Validation

Enforce that certain claims must be present in the token:

```rust
let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
validation.set_required_spec_claims(&["exp", "iat", "sub"]);
```

#### Audience Validation

```rust
let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
validation.set_audience(&["my-api"]);
```

#### Combining Multiple Options

```rust
let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
validation.leeway = 60;                                    // 60s clock skew tolerance
validation.set_required_spec_claims(&["exp", "sub"]);      // Require expiration and subject
validation.set_audience(&["my-api"]);                      // Validate audience
```

---

## Supporting Multiple JWT Providers

In some scenarios, you may need to accept JWT tokens from multiple sources, such as your own authentication system and external identity providers like Azure AD, Auth0, or other OAuth 2.0 providers. This section explains how to implement multi-provider token validation using the `key_func` callback.

### Use Cases

- **Hybrid Authentication**: Support both internal and external authentication
- **Third-Party Integration**: Accept tokens from Azure AD, Google, Auth0, etc.
- **Migration Scenarios**: Gradually migrate from one auth system to another
- **Enterprise SSO**: Support enterprise Single Sign-On alongside regular auth

### Solution: Dynamic Key Function

The recommended approach is to use a **single middleware with a dynamic `key_func`** that determines the appropriate validation method based on token properties (such as the issuer claim).

#### Why This Works

The `key_func` callback is designed for exactly this purpose. It allows you to:

- Inspect the token header before validation
- Choose the correct signing key/method dynamically
- Avoid issues when chaining multiple middlewares

### Implementation

```rust
use std::sync::Arc;
use jsonwebtoken::{DecodingKey, Header};
use actix_jwt::{ActixJwtMiddleware, JwtError};

let own_secret = b"your-secret-key";

let mut jwt = ActixJwtMiddleware::new();
jwt.key = own_secret.to_vec();
jwt.identity_key = "sub".to_string();

// Dynamic key function - the core of multi-provider support
jwt.key_func = Some(Arc::new(|header: &Header| {
    match header.alg {
        // RS256 tokens (e.g., Azure AD, Google)
        jsonwebtoken::Algorithm::RS256 => {
            // Look up the public key by KID from header
            let kid = header.kid.as_deref().unwrap_or("");
            // In production: fetch from JWKS endpoint and cache
            // let key = get_cached_jwks_key(kid)?;
            Err(JwtError::InvalidToken(
                format!("unknown key ID: {}", kid)
            ))
        }
        // HS256 tokens (your own)
        jsonwebtoken::Algorithm::HS256 => {
            Ok(DecodingKey::from_secret(b"your-secret-key"))
        }
        _ => Err(JwtError::InvalidToken(
            format!("unexpected algorithm: {:?}", header.alg)
        )),
    }
}));
```

### Provider-Specific Identity Handler

```rust
use actix_web::HttpRequest;
use actix_jwt::extract_claims;
use serde_json::json;

jwt.identity_handler = Arc::new(|req: &HttpRequest| {
    let claims = extract_claims(req);

    // Try standard "sub" claim (used by most OAuth providers)
    if let Some(sub) = claims.get("sub") {
        return Some(sub.clone());
    }

    // Fallback to custom "identity" claim
    if let Some(identity) = claims.get("identity") {
        return Some(identity.clone());
    }

    None
});
```

### Provider-Specific Authorization

```rust
jwt.authorizer = Arc::new(|req: &HttpRequest, data: &serde_json::Value| {
    let claims = extract_claims(req);
    let issuer = claims.get("iss")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if issuer.contains("login.microsoftonline.com") {
        // Azure AD specific authorization
        // Check roles, groups, app_role claims
        return claims.get("roles")
            .and_then(|v| v.as_array())
            .map(|roles| roles.iter().any(|r| r.as_str() == Some("User")))
            .unwrap_or(false);
    }

    // Your own token authorization
    data.get("role")
        .and_then(|r| r.as_str())
        .map(|r| r == "admin" || r == "user")
        .unwrap_or(false)
});
```

### Key Considerations

1. **Token Issuer Validation**: Always validate the `iss` claim to ensure tokens are from trusted sources
2. **Audience Validation**: Verify the `aud` claim matches your application's client ID
3. **Algorithm Validation**: Ensure the signing algorithm matches expectations (HS256 for your tokens, RS256 for Azure AD)
4. **Key Caching**: Cache public keys from JWKS endpoints to reduce latency
5. **Key Rotation**: Implement automatic key refresh to handle provider key rotation
6. **Error Handling**: Provide clear error messages indicating which provider validation failed
7. **Security**: Never skip signature validation or disable security checks

### Testing Multi-Provider Setup

```bash
# Test with your own token
curl -H "Authorization: Bearer YOUR_INTERNAL_TOKEN" \
     http://localhost:8000/api/profile

# Test with Azure AD token
curl -H "Authorization: Bearer AZURE_AD_TOKEN" \
     http://localhost:8000/api/profile
```

### Additional Resources

- [Azure AD Token Validation](https://docs.microsoft.com/en-us/azure/active-directory/develop/access-tokens)
- [JWKS (JSON Web Key Sets)](https://auth0.com/docs/secure/tokens/json-web-tokens/json-web-key-sets)
- [RFC 7517 - JSON Web Key (JWK)](https://tools.ietf.org/html/rfc7517)
- [jsonwebtoken crate](https://docs.rs/jsonwebtoken) for JWT handling in Rust

---

## Token Generator (Direct Token Creation)

The `token_generator` functionality allows you to create JWT tokens directly without HTTP middleware, perfect for programmatic authentication, testing, and custom flows.

### Basic Usage

```rust
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use actix_jwt::{ActixJwtMiddleware, JwtError};
use serde_json::json;

#[tokio::main]
async fn main() {
    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "example zone".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(86400);

    jwt.payload_func = Some(Arc::new(|data: &serde_json::Value| {
        let mut claims = HashMap::new();
        claims.insert("user_id".to_string(), data.clone());
        claims
    }));

    jwt.authenticator = Some(Arc::new(|_req, _body| {
        Err(JwtError::MissingLoginValues)
    }));

    jwt.init().expect("JWT init failed");

    // Generate a complete token pair (access + refresh tokens)
    let user_data = json!("user123");
    let token = jwt.token_generator(&user_data).await.unwrap();

    println!("Access Token: {}", token.access_token);
    println!("Refresh Token: {:?}", token.refresh_token);
    println!("Expires In: {} seconds", token.expires_in());
}
```

### Token Structure

The `token_generator` method returns a structured `Token`:

```rust
pub struct Token {
    pub access_token: String,           // JWT access token
    pub token_type: String,             // Always "Bearer"
    pub refresh_token: Option<String>,  // Opaque refresh token
    pub expires_at: i64,                // Unix timestamp
    pub created_at: i64,                // Unix timestamp
}

impl Token {
    pub fn expires_in(&self) -> i64;    // Returns seconds until expiry
}
```

### Refresh Token Management

Use `token_generator_with_revocation` to refresh tokens and automatically revoke old ones:

```rust
// Refresh with automatic revocation of old token
let new_token = jwt.token_generator_with_revocation(
    &user_data,
    &old_refresh_token,
).await?;

// Old refresh token is now invalid
println!("New Access Token: {}", new_token.access_token);
println!("New Refresh Token: {:?}", new_token.refresh_token);
```

**Use Cases:**

- **Programmatic Authentication**: Service-to-service communication
- **Testing**: Generate tokens for testing authenticated endpoints
- **Registration Flow**: Issue tokens immediately after user signup
- **Background Jobs**: Create tokens for automated processes
- **Custom Auth Flows**: Build custom authentication logic

See the [complete example](examples/token_generator.rs) for more details.

---

## Redis Store Configuration

This library supports Redis as a backend for refresh token storage, feature-gated behind `redis-store`. Redis store provides better scalability and persistence compared to the default in-memory store.

### Redis Features

- Automatic fallback to in-memory store if Redis connection fails
- Easy configuration via `RedisConfig`
- Configurable key prefix for Redis keys
- Optional TLS support

### Configuration Options

#### RedisConfig

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `addr` | `String` | `"redis://127.0.0.1:6379/"` | Redis connection URL |
| `password` | `Option<String>` | `None` | Redis password |
| `db` | `i32` | `0` | Redis database number |
| `pool_size` | `u32` | `10` | Connection pool size |
| `key_prefix` | `String` | `"actix-jwt:"` | Prefix for all Redis keys |
| `tls` | `bool` | `false` | Whether to use TLS |

### Fallback Behavior

If Redis connection fails during initialization:

- The middleware logs an error message
- Automatically falls back to in-memory store
- Application continues to function normally

This ensures high availability and prevents application failures due to Redis connectivity issues.

### Example with Redis

Enable the `redis-store` feature in `Cargo.toml`:

```toml
[dependencies]
actix-jwt = { git = "https://github.com/LdDl/actix-jwt", features = ["redis-store"] }
```

```rust
use std::sync::Arc;
use std::time::Duration;
use actix_jwt::{ActixJwtMiddleware, JwtError};
use actix_jwt::store::RedisRefreshTokenStore;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut jwt = ActixJwtMiddleware::new();
    jwt.realm = "example zone".to_string();
    jwt.key = b"secret key".to_vec();
    jwt.timeout = Duration::from_secs(3600);
    jwt.max_refresh = Duration::from_secs(86400);
    jwt.identity_key = "id".to_string();

    // Configure Redis store
    let redis_config = actix_jwt::store::RedisConfig {
        addr: "redis://127.0.0.1:6379/".to_string(),
        ..Default::default()
    };
    let redis_store = RedisRefreshTokenStore::new(redis_config)
        .await
        .expect("Failed to connect to Redis");
    jwt.refresh_token_store = Arc::new(redis_store);

    jwt.authenticator = Some(Arc::new(|_req: &HttpRequest, body: &[u8]| {
        // your authentication logic
        Ok(json!({"username": "admin"}))
    }));

    jwt.init().expect("JWT middleware init failed");

    let jwt_arc = Arc::new(jwt);

    HttpServer::new(move || {
        let jwt_data = web::Data::new(jwt_arc.clone());
        App::new()
            .app_data(jwt_data.clone())
            .route("/login", web::post().to({
                let jwt = jwt_arc.clone();
                move |req: HttpRequest, body: web::Bytes| {
                    let jwt = jwt.clone();
                    async move { jwt.login_handler(&req, &body).await }
                }
            }))
            .service(
                web::scope("/auth")
                    .wrap(jwt_arc.middleware())
                    .route("/hello", web::get().to(|req: HttpRequest| async move {
                        HttpResponse::Ok().json(json!({"message": "Hello World."}))
                    })),
            )
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
```

---

## Demo

Run the example server:

```sh
cargo run --example basic
```

Install [httpie](https://github.com/jkbrzt/httpie) for easy API testing.

### Login

```sh
http -v --json POST localhost:8000/login username=admin password=admin
```

### Refresh Token

Using RFC 6749 compliant refresh tokens (default behavior):

```sh
# First login to get refresh token
http -v --json POST localhost:8000/login username=admin password=admin

# Method 1: With cookies enabled (automatic - recommended for browsers)
# The refresh token cookie is automatically sent, no need to manually include it
http -v POST localhost:8000/refresh --session=./session.json

# Method 2: Send refresh token in JSON body
http -v --json POST localhost:8000/refresh refresh_token=your_refresh_token_here

# Method 3: Use refresh token from response via form data
http -v --form POST localhost:8000/refresh refresh_token=your_refresh_token_here
```

**Security Note**: When `send_cookie` is enabled, refresh tokens are automatically stored in httpOnly cookies. Browser-based applications can simply call the refresh endpoint without manually including the token - it's handled automatically by the cookie mechanism.

**Important**: Query parameters are NOT supported for refresh tokens as they expose tokens in server logs, proxy logs, browser history, and Referer headers. Use cookies (recommended), JSON body, or form data instead.

### Hello World

Login as `admin`/`admin` and call:

```sh
http -f GET localhost:8000/auth/hello "Authorization:Bearer xxxxxxxxx"  "Content-Type: application/json"
```

**Response:**

```json
{
  "text": "Hello World.",
  "userID": "admin"
}
```

### Authorization Example

Login as `test`/`test` and call:

```sh
http -f GET localhost:8000/auth/hello "Authorization:Bearer xxxxxxxxx"  "Content-Type: application/json"
```

**Response:**

```json
{
  "code": 403,
  "message": "You don't have permission to access."
}
```

---

## Understanding the Authorizer

The `authorizer` callback is a crucial component for implementing role-based access control in your application. It determines whether an authenticated user has permission to access specific protected routes.

### How Authorizer Works

The `authorizer` is called **automatically** during the JWT middleware processing for any route that uses `.wrap(jwt_arc.middleware())`. Here is the execution flow:

1. **Token Validation**: JWT middleware validates the token
2. **Identity Extraction**: `identity_handler` extracts user identity from token claims
3. **Authorization Check**: `authorizer` determines if the user can access the resource
4. **Route Access**: If authorized, request proceeds; otherwise, `unauthorized` is called

### Authorizer Function Signature

```rust
Fn(&HttpRequest, &Value) -> bool
```

- `&HttpRequest`: The actix-web request containing request information
- `&Value`: User identity data returned by `identity_handler`
- Returns `bool`: `true` for authorized access, `false` to deny access

### Basic Usage Examples

#### Example 1: Role-Based Authorization

```rust
jwt.authorizer = Arc::new(|_req: &HttpRequest, data: &Value| {
    // Only allow "admin" users
    data.as_str() == Some("admin")
});
```

#### Example 2: Path-Based Authorization

```rust
jwt.authorizer = Arc::new(|req: &HttpRequest, data: &Value| {
    let role = data.get("role").and_then(|r| r.as_str()).unwrap_or("");
    let path = req.path();

    // Admin can access all routes
    if role == "admin" {
        return true;
    }

    // Regular users can only access specific routes
    let allowed = ["/auth/profile", "/auth/hello"];
    allowed.contains(&path)
});
```

#### Example 3: Method and Path Based Authorization

```rust
jwt.authorizer = Arc::new(|req: &HttpRequest, data: &Value| {
    let role = data.get("role").and_then(|r| r.as_str()).unwrap_or("");
    let path = req.path();
    let method = req.method().as_str();

    // Admins have full access
    if role == "admin" {
        return true;
    }

    // Users can only GET their own profile
    if path == "/auth/profile" && method == "GET" {
        return true;
    }

    // Users cannot modify or delete resources
    if matches!(method, "POST" | "PUT" | "DELETE") {
        return false;
    }

    true // Allow other GET requests
});
```

### Setting Up Different Authorization for Different Routes

#### Method 1: Multiple Middleware Instances

```rust
// Admin-only middleware
let mut admin_jwt = ActixJwtMiddleware::new();
// ... shared config ...
admin_jwt.authorizer = Arc::new(|_req, data| {
    data.get("role").and_then(|r| r.as_str()) == Some("admin")
});
admin_jwt.init().unwrap();
let admin_arc = Arc::new(admin_jwt);

// Regular user middleware
let mut user_jwt = ActixJwtMiddleware::new();
// ... shared config ...
user_jwt.authorizer = Arc::new(|_req, data| {
    matches!(
        data.get("role").and_then(|r| r.as_str()),
        Some("user") | Some("admin")
    )
});
user_jwt.init().unwrap();
let user_arc = Arc::new(user_jwt);

// Route setup
App::new()
    .service(
        web::scope("/admin")
            .wrap(admin_arc.middleware())
            // admin-only routes
    )
    .service(
        web::scope("/user")
            .wrap(user_arc.middleware())
            // user routes
    )
```

#### Method 2: Single Authorizer with Path Logic

```rust
jwt.authorizer = Arc::new(|req: &HttpRequest, data: &Value| {
    let role = data.get("role").and_then(|r| r.as_str()).unwrap_or("");
    let path = req.path();

    // Admin routes - only admins allowed
    if path.starts_with("/admin/") {
        return role == "admin";
    }

    // User routes - users and admins allowed
    if path.starts_with("/user/") {
        return role == "user" || role == "admin";
    }

    // Public authenticated routes - all authenticated users
    true
});
```

### Advanced Authorization Patterns

#### Using Claims for Fine-Grained Control

```rust
use actix_jwt::extract_claims;

jwt.authorizer = Arc::new(|req: &HttpRequest, _data: &Value| {
    // Extract additional claims
    let claims = extract_claims(req);

    // Get user permissions from claims
    let permissions = claims.get("permissions")
        .and_then(|v| v.as_array());

    let permissions = match permissions {
        Some(p) => p,
        None => return false,
    };

    // Check if user has required permission for this route
    let required = get_required_permission(req.path());

    permissions.iter()
        .any(|p| p.as_str() == Some(required))
});

fn get_required_permission(path: &str) -> &str {
    match path {
        "/auth/users" => "read_users",
        "/auth/reports" => "read_reports",
        "/auth/settings" => "admin",
        _ => "none",
    }
}
```

### Common Patterns and Best Practices

1. **Always validate the data type**: Check that the `Value` contains expected fields before accessing
2. **Use claims for additional context**: Access JWT claims using `extract_claims(&req)`
3. **Consider the request context**: Use `req.path()`, `req.method()`, etc.
4. **Fail securely**: Return `false` by default and explicitly allow access
5. **Log authorization failures**: Add logging for debugging authorization issues

### Complete Example

See the [authorization example](examples/authorization.rs) for a complete implementation showing different authorization scenarios.

### Logout

Login first, then call the logout endpoint with the JWT token:

```sh
# First login to get the JWT token
http -v --json POST localhost:8000/login username=admin password=admin

# Use the returned JWT token to logout (replace xxxxxxxxx with actual token)
http -f POST localhost:8000/auth/logout "Authorization:Bearer xxxxxxxxx" "Content-Type: application/json"
```

**Response:**

```json
{
  "code": 200,
  "logged_out_user": "admin",
  "message": "Successfully logged out",
  "user_info": "admin"
}
```

The logout response demonstrates that JWT claims are accessible during logout through `extract_claims(&req)`, allowing developers to access user information for logging, auditing, or cleanup purposes.

---

## Cookie Token

To set the JWT in a cookie, use these options (see [MDN docs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Cookies#Secure_and_HttpOnly_cookies)):

```rust
jwt.send_cookie = true;
jwt.secure_cookie = false;                          // for non-HTTPS dev environments (access token only)
jwt.cookie_http_only = true;                        // JS can't modify
jwt.cookie_domain = Some("localhost".to_string());
jwt.cookie_name = "token".to_string();              // default: "jwt"
jwt.refresh_token_cookie_name = "refresh_token".to_string(); // default: "refresh_token"
jwt.token_lookup = "cookie:token".to_string();
jwt.cookie_same_site = SameSite::Lax;              // Lax, Strict, or None
```

### Refresh Token Cookie Support

When `send_cookie` is enabled, the middleware automatically stores both access and refresh tokens as httpOnly cookies:

- **Access Token Cookie**: Stored with the name specified in `cookie_name` (default: `"jwt"`)
- **Refresh Token Cookie**: Stored with the name specified in `refresh_token_cookie_name` (default: `"refresh_token"`)

The refresh token cookie:

- Uses the `refresh_token_timeout` duration (default: 30 days)
- Is always set with `httpOnly: true` for security
- Is always set with `secure: true` (HTTPS only) regardless of the `secure_cookie` setting
- Is automatically sent with refresh requests
- Is cleared on logout

**Automatic Token Extraction**: The `refresh_handler` automatically extracts refresh tokens from cookies, form data, query parameters, or JSON body, in that order. This means you don't need to manually include the refresh token when using cookie-based authentication - it's handled automatically.

### Login request flow (using login_handler)

**PROVIDED: `login_handler`**

This is a provided method to be called on any login endpoint, which will trigger the flow described below.

**REQUIRED: `authenticator`**

This callback should verify the user credentials given the `HttpRequest` and raw body bytes (i.e. password matches hashed password for a given user email, and any other authentication logic). Then the authenticator should return a `serde_json::Value` that contains the user data that will be embedded in the JWT token. This might be something like an account id, role, is_verified, etc. After having successfully authenticated, the data returned from the authenticator is passed in as a parameter into the `payload_func`, which is used to embed the user identifiers mentioned above into the JWT token. If an error is returned, the `unauthorized` callback is used.

**OPTIONAL: `payload_func`**

This function is called after having successfully authenticated (logged in). It should take whatever was returned from `authenticator` and convert it into a `HashMap<String, Value>`. A typical use case of this function is for when `authenticator` returns a `Value` which holds the user identifiers, and those fields need to be extracted into a map. The map should include one element that is `[identity_key (default "identity"): some_user_identity]`. The elements of the map returned in `payload_func` will be embedded within the JWT token (as token claims). When users pass in their token on subsequent requests, you can get these claims back by using `extract_claims`.

**Standard JWT Claims (RFC 7519):** You can set standard JWT claims in `payload_func` for better interoperability:

- `sub` (Subject) - The user identifier (e.g., user ID)
- `iss` (Issuer) - The issuer of the token (e.g., your app name)
- `aud` (Audience) - The intended audience (e.g., your API)
- `nbf` (Not Before) - Token is not valid before this time
- `iat` (Issued At) - When the token was issued
- `jti` (JWT ID) - Unique identifier for the token

**Note:** The `exp` (Expiration) and `orig_iat` claims are managed by the framework and cannot be overwritten.

```rust
jwt.payload_func = Some(Arc::new(|data: &Value| {
    let mut claims = HashMap::new();
    if let Some(user) = data.as_object() {
        if let Some(id) = user.get("id") {
            claims.insert("sub".to_string(), id.clone());          // Standard: Subject
        }
        claims.insert("iss".to_string(), json!("my-app"));         // Standard: Issuer
        claims.insert("aud".to_string(), json!("my-api"));         // Standard: Audience
        if let Some(name) = user.get("username") {
            claims.insert("identity".to_string(), name.clone());   // Custom claim
        }
        if let Some(role) = user.get("role") {
            claims.insert("role".to_string(), role.clone());       // Custom claim
        }
    }
    claims
}));
```

**OPTIONAL: `login_response`**

After having successfully authenticated with `authenticator`, created the JWT token using the identifiers from the map returned from `payload_func`, and set cookies if `send_cookie` is enabled, this function is called. This function receives the complete token information as a structured `Token` object and should return the appropriate `HttpResponse`.

Signature: `Fn(&HttpRequest, &Token) -> HttpResponse`

### Subsequent requests on endpoints requiring jwt token (using middleware)

**PROVIDED: `middleware()`**

This returns an actix-web middleware (`Transform`) that should be used via `.wrap()` on any scope or resource that requires JWT authentication. This middleware will parse the request headers for the token if it exists, and check that the JWT token is valid (not expired, correct signature). Then it will call `identity_handler` followed by `authorizer`. If `authorizer` passes and all of the previous token validity checks passed, the middleware will continue the request. If any of these checks fail, the `unauthorized` function is used.

**OPTIONAL: `identity_handler`**

The default of this function is likely sufficient for your needs. The purpose of this function is to fetch the user identity from claims embedded within the JWT token, and pass this identity value to `authorizer`. This function assumes `[identity_key: some_user_identity]` is one of the attributes embedded within the claims of the JWT token (determined by `payload_func`).

**OPTIONAL: `authorizer`**

Given the user identity value (`data` parameter) and the `HttpRequest`, this function should check if the user is authorized to be reaching this endpoint (on the endpoints where the middleware applies). This function should likely use `extract_claims` to check if the user has the sufficient permissions to reach this endpoint, as opposed to hitting the database on every request. This function should return `true` if the user is authorized to continue through with the request, or `false` if they are not authorized (where `unauthorized` will be called).

### Logout request flow (using logout_handler)

**PROVIDED: `logout_handler`**

This is a provided method to be called on any logout endpoint. The handler performs the following actions:

1. Extracts JWT claims to make them available in `logout_response` (for logging/auditing)
2. Attempts to revoke the refresh token from the server-side store if provided
3. Clears authentication cookies if `send_cookie` is enabled:
   - **Access Token Cookie**: Named according to `cookie_name`
   - **Refresh Token Cookie**: Named according to `refresh_token_cookie_name`
4. Calls `logout_response` to return the response

The logout handler tries to extract the refresh token from multiple sources (cookie, form, query, JSON body) to ensure it can be properly revoked.

**OPTIONAL: `logout_response`**

This function is called after logout processing is complete. It should return the appropriate `HttpResponse` to indicate logout success or failure. Since logout doesn't generate new tokens, this function only receives the `HttpRequest`. You can access JWT claims and user identity via `extract_claims(&req)` and `get_identity(&req)` for logging or auditing purposes.

Signature: `Fn(&HttpRequest) -> HttpResponse`

### Refresh request flow (using refresh_handler)

**PROVIDED: `refresh_handler`**

This is a provided method to be called on any refresh token endpoint. The handler expects a `refresh_token` parameter (RFC 6749 compliant) from multiple sources and validates it against the server-side token store. The handler automatically extracts the refresh token from the following sources in order of priority:

1. **Cookie** (most common for browser-based apps): `refresh_token_cookie_name` cookie (default: `"refresh_token"`)
2. **POST Form**: `refresh_token` form field
3. **JSON Body**: `refresh_token` field in request body

**Security Note**: Query parameters are NOT supported for refresh tokens to prevent token leakage through server logs, proxy logs, browser history, and Referer headers. Only secure delivery methods are supported.

If the refresh token is valid and not expired, the handler will:

- Create a new access token and refresh token
- Revoke the old refresh token (token rotation)
- Set both tokens as cookies (if `send_cookie` is enabled)
- Pass the new tokens into `refresh_response`

This follows OAuth 2.0 security best practices by rotating refresh tokens and supporting multiple secure delivery methods.

**Cookie-Based Authentication**: When using cookies (recommended for browser apps), the refresh token is automatically sent with the request, so you don't need to manually include it. Simply call the refresh endpoint and the middleware handles everything.

**OPTIONAL: `refresh_response`**

This function is called after successfully refreshing tokens. It receives the complete new token information as a structured `Token` object and should return an `HttpResponse` containing the new `access_token`, `token_type`, `expires_in`, and `refresh_token` fields, following RFC 6749 token response format. Note that when using cookies, the tokens are already set as httpOnly cookies before this function is called.

Signature: `Fn(&HttpRequest, &Token) -> HttpResponse`

### Failures with logging in, bad tokens, or lacking privileges

**OPTIONAL: `unauthorized`**

On any error logging in, authorizing the user, or when there was no token or an invalid token passed in with the request, the following will happen. `http_status_message_func` is called which by default converts the error into a string. Finally the `unauthorized` callback will be called. This function should likely return an `HttpResponse` containing the HTTP error code and error message to the user.

**Note:** When a 401 Unauthorized response is returned, the middleware automatically adds a `WWW-Authenticate` header with the `Bearer` authentication scheme, as defined in [RFC 6750](https://tools.ietf.org/html/rfc6750) (OAuth 2.0 Bearer Token Usage), [RFC 7235](https://tools.ietf.org/html/rfc7235) (HTTP Authentication), and the [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/401):

```txt
WWW-Authenticate: Bearer realm="<your-realm>"
```

This header informs HTTP clients that Bearer token authentication is required, ensuring compatibility with standard HTTP authentication mechanisms.
