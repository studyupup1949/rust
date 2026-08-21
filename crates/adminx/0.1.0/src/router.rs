// crates/adminx/src/router.rs
use actix_web::{web, Scope, HttpRequest, HttpResponse, HttpMessage};
use serde_json::Value;
use crate::nested::AdmixNestedResource;
use crate::registry::all_resources;
use crate::resource::AdmixResource;
use crate::actions::CustomAction;
use crate::menu::MenuAction;
use crate::utils::rbac::has_permission;
use crate::controllers::{
    dashboard_controller::{
        adminx_home
    },
    resource_controller::{
        register_admix_resource_routes
    }
};
use crate::controllers::auth_controller::{login_form, login_action, logout_action};
use crate::utils::{
    structs::{
        RoleGuard
    },
};
use tracing::{info, warn, error};

fn extract_roles_from_request(req: &HttpRequest) -> Vec<String> {
    // Extract roles from request extensions (set by middleware)
    if let Some(claims) = req.extensions().get::<crate::utils::structs::Claims>() {
        let mut roles = claims.roles.clone();
        roles.push(claims.role.clone());
        roles
    } else {
        Vec::new()
    }
}

pub fn register_all_admix_routes() -> Scope {
    info!("🔧 Starting AdminX route registration...");
    
    let mut scope = web::scope("/adminx")
        .route("/login", web::get().to(login_form))
        .route("/login", web::post().to(login_action))
        .route("/logout", web::post().to(logout_action))
        .route("", web::get().to(adminx_home))
        .route("/", web::get().to(adminx_home));

    // Debug: Check if we have any resources
    let resources = all_resources();
    info!("📋 Found {} resources to register", resources.len());
    
    if resources.is_empty() {
        warn!("⚠️  No resources found! Make sure you've called register_resource() before starting the server.");
        return scope;
    }

    // Register resource routes with role guards
    for resource in resources {
        let resource_name = resource.resource_name();
        let base_path = resource.base_path();
        let allowed_roles = resource.allowed_roles();
        
        info!("📝 Registering resource: '{}' at path: '{}'", resource_name, base_path);
        info!("🔐 Allowed roles for {}: {:?}", resource_name, allowed_roles);
        
        // Create the resource scope with the base path
        let resource_scope = web::scope(&format!("/{}", base_path))
            .service(register_admix_resource_routes(resource))
            .wrap(RoleGuard { allowed_roles });
        
        scope = scope.service(resource_scope);
        
        info!("✅ Successfully registered resource: '{}'", resource_name);
        info!("🌐 Available URLs:");
        info!("   - GET  /adminx/{}/list", base_path);
        info!("   - GET  /adminx/{}/new", base_path);
        info!("   - GET  /adminx/{}/view/{{id}}", base_path);
        info!("   - GET  /adminx/{}/edit/{{id}}", base_path);
        info!("   - GET  /adminx/{} (API list)", base_path);
        info!("   - POST /adminx/{} (API create)", base_path);
        info!("   - GET  /adminx/{}/{{id}} (API get)", base_path);
        info!("   - PUT  /adminx/{}/{{id}} (API update)", base_path);
        info!("   - DELETE /adminx/{}/{{id}} (API delete)", base_path);
    }
    
    info!("🎉 AdminX route registration completed!");
    scope
}

// Alternative version without middleware (for testing)
pub fn register_all_admix_routes_debug() -> Scope {
    info!("🔧 Starting AdminX route registration (DEBUG MODE - NO AUTH)...");
    
    let mut scope = web::scope("/adminx")
        .route("/login", web::get().to(login_form))
        .route("/login", web::post().to(login_action))
        .route("/logout", web::post().to(logout_action))
        .route("", web::get().to(adminx_home))
        .route("/", web::get().to(adminx_home));

    // Debug: Check if we have any resources
    let resources = all_resources();
    info!("📋 Found {} resources to register", resources.len());
    
    if resources.is_empty() {
        warn!("⚠️  No resources found! Make sure you've called register_resource() before starting the server.");
        return scope;
    }

    // Register resource routes WITHOUT role guards for debugging
    for resource in resources {
        let resource_name = resource.resource_name();
        let base_path = resource.base_path();
        
        info!("📝 Registering resource: '{}' at path: '{}'", resource_name, base_path);
        
        // Create the resource scope with the base path - NO MIDDLEWARE
        let resource_scope = web::scope(&format!("/{}", base_path))
            .service(register_admix_resource_routes(resource));
        
        scope = scope.service(resource_scope);
        
        info!("✅ Successfully registered resource: '{}'", resource_name);
    }
    
    info!("🎉 AdminX route registration completed (DEBUG MODE)!");
    scope
}