// adminx/src/controllers/auth_controller.rs
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use tera::Context;
use tracing::{error, info, warn};
use crate::helpers::template_helper::render_template;
use crate::models::adminx_model::get_admin_by_email;
use crate::registry::get_registered_menus;
use crate::utils::jwt::create_jwt_token;
use crate::utils::structs::LoginForm;
use crate::configs::initializer::AdminxConfig;
use crate::utils::auth::{is_rate_limited, reset_rate_limit};
use std::time::Duration;

/// GET /adminx/login - Show login page
pub async fn login_form() -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("menus", &get_registered_menus());
    render_template("login.html.tera", ctx).await
}

/// POST /adminx/login - Authenticate and store token in session
pub async fn login_action(
    form: web::Form<LoginForm>,
    session: Session,
    config: web::Data<AdminxConfig>, // Inject config
) -> impl Responder {
    let email = form.email.trim();
    let password = form.password.trim();
    
    info!("Attempting login for: {}", email);
    
    // Rate limiting check (optional security enhancement)
    if is_rate_limited(email, 5, Duration::from_secs(900)) {
        warn!("Rate limit exceeded for: {}", email);
        return HttpResponse::TooManyRequests()
            .body("Too many login attempts. Please try again later.");
    }
    
    // Dummy hash to prevent timing attacks
    let dummy_hash = "$2b$12$dummy.hash.to.prevent.timing.attacks.abcdefghijklmnopqrstuvwxy";
    
    match get_admin_by_email(email).await {
        Some(admin) => {
            if admin.verify_password(password) {
                let admin_id = match &admin.id {
                    Some(id) => id.to_string(),
                    None => {
                        error!("Admin has no ID: {}", email);
                        return HttpResponse::InternalServerError().body("Missing Admin ID");
                    }
                };
                
                // Use config for JWT creation
                match create_jwt_token(&admin_id, email, "admin", &config) {
                    Ok(token) => {
                        info!("Login successful for: {}", email);
                        
                        // Reset rate limit on successful login
                        reset_rate_limit(email);
                        
                        if let Err(err) = session.insert("admintoken", &token) {
                            error!("Session insertion failed: {}", err);
                            return HttpResponse::InternalServerError().body("Session storage failed");
                        }
                        HttpResponse::Found()
                            .append_header(("Location", "/adminx"))
                            .finish()
                    }
                    Err(err) => {
                        error!("JWT generation failed for {}: {}", email, err);
                        HttpResponse::InternalServerError().body("Token generation failed")
                    }
                }
            } else {
                // Perform dummy verification to maintain consistent timing
                bcrypt::verify(password, dummy_hash).ok();
                info!("Invalid password for: {}", email);
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        None => {
            // Perform dummy verification to maintain consistent timing
            bcrypt::verify(password, dummy_hash).ok();
            info!("Admin not found: {}", email);
            HttpResponse::Unauthorized().body("Invalid credentials")
        }
    }
}

/// POST /adminx/logout - Clear session and redirect
pub async fn logout_action(session: Session) -> impl Responder {
    // session.clear() returns (), not a Result, so we can't use if let Err
    session.clear();
    
    info!("User logged out successfully");
    HttpResponse::Found()
        .append_header(("Location", "/adminx/login"))
        .finish()
}

/// GET /adminx/profile - Show user profile (example of protected route)
pub async fn profile_view(
    session: Session,
    config: web::Data<AdminxConfig>,
) -> impl Responder {
    use crate::utils::auth::extract_claims_from_session;
    
    match extract_claims_from_session(&session, &config).await {
        Ok(claims) => {
            let mut ctx = Context::new();
            ctx.insert("user_email", &claims.email);
            ctx.insert("user_role", &claims.role);
            ctx.insert("menus", &get_registered_menus());
            render_template("profile.html.tera", ctx).await
        }
        Err(_) => {
            HttpResponse::Found()
                .append_header(("Location", "/adminx/login"))
                .finish()
        }
    }
}

/// Helper function for error responses with consistent format
fn auth_error_response(message: &str, status: actix_web::http::StatusCode) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("application/json")
        .json(serde_json::json!({
            "error": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
}

/// Enhanced login with better error handling and security
pub async fn enhanced_login_action(
    form: web::Form<LoginForm>,
    session: Session,
    config: web::Data<AdminxConfig>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let email = form.email.trim();
    let password = form.password.trim();
    
    // Get request metadata for logging - fix the lifetime issue
    let connection_info = req.connection_info();
    let ip = connection_info.peer_addr().unwrap_or("unknown");
    let user_agent = req.headers().get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    info!(
        email = %email,
        ip = %ip,
        user_agent = %user_agent,
        "Login attempt"
    );
    
    // Input validation
    if email.is_empty() || password.is_empty() {
        return auth_error_response("Email and password are required", 
            actix_web::http::StatusCode::BAD_REQUEST);
    }
    
    if !email.contains('@') {
        return auth_error_response("Invalid email format", 
            actix_web::http::StatusCode::BAD_REQUEST);
    }
    
    // Rate limiting
    if is_rate_limited(email, 5, Duration::from_secs(900)) {
        warn!(
            email = %email,
            ip = %ip,
            "Rate limit exceeded"
        );
        return auth_error_response("Too many login attempts", 
            actix_web::http::StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Dummy hash for timing attack prevention
    let dummy_hash = "$2b$12$dummy.hash.to.prevent.timing.attacks.abcdefghijklmnopqrstuvwxy";
    
    match get_admin_by_email(email).await {
        Some(admin) => {
            if admin.verify_password(password) {
                let admin_id = match &admin.id {
                    Some(id) => id.to_string(),
                    None => {
                        error!("Admin has no ID: {}", email);
                        return auth_error_response("Authentication failed", 
                            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };
                
                match create_jwt_token(&admin_id, email, "admin", &config) {
                    Ok(token) => {
                        info!(
                            email = %email,
                            ip = %ip,
                            "Login successful"
                        );
                        
                        reset_rate_limit(email);
                        
                        if let Err(err) = session.insert("admintoken", &token) {
                            error!("Session insertion failed: {}", err);
                            return auth_error_response("Session creation failed", 
                                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);
                        }
                        
                        HttpResponse::Ok().json(serde_json::json!({
                            "success": true,
                            "redirect": "/adminx",
                            "message": "Login successful"
                        }))
                    }
                    Err(err) => {
                        error!("JWT generation failed for {}: {}", email, err);
                        auth_error_response("Authentication failed", 
                            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                bcrypt::verify(password, dummy_hash).ok();
                warn!(
                    email = %email,
                    ip = %ip,
                    "Invalid password"
                );
                auth_error_response("Invalid credentials", 
                    actix_web::http::StatusCode::UNAUTHORIZED)
            }
        }
        None => {
            bcrypt::verify(password, dummy_hash).ok();
            warn!(
                email = %email,
                ip = %ip,
                "Admin not found"
            );
            auth_error_response("Invalid credentials", 
                actix_web::http::StatusCode::UNAUTHORIZED)
        }
    }
}