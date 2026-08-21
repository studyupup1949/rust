// crates/adminx/src/helpers/resource_helper.rs - Working Version
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use serde_json::Value;
use std::sync::Arc;
use tera::Context;
use tracing::{info, warn, error};
use std::collections::HashSet;
use actix_session::Session;
use futures::TryStreamExt;

use crate::AdmixResource;
use crate::helpers::form_helper::extract_fields_for_form;
use crate::helpers::template_helper::render_template;
use crate::configs::initializer::AdminxConfig;
use crate::utils::auth::extract_claims_from_session;
use crate::utils::structs::Claims;
use crate::registry::get_registered_menus;

/// Check authentication and return user claims or redirect response
pub async fn check_authentication(
    session: &Session,
    config: &AdminxConfig,
    resource_name: &str,
    action: &str,
) -> Result<Claims, HttpResponse> {
    match extract_claims_from_session(session, config).await {
        Ok(claims) => {
            info!("🔐 Authenticated user {} accessing {} action on resource {}", 
                  claims.email, action, resource_name);
            Ok(claims)
        }
        Err(_) => {
            warn!("⚠️  Unauthenticated access attempt to {} action on resource {}", action, resource_name);
            Err(HttpResponse::Found()
                .append_header(("Location", "/adminx/login"))
                .finish())
        }
    }
}

/// Check if user has permission for resource action - Enhanced version
pub async fn check_resource_permission(
    session: &Session,
    config: &AdminxConfig,
    resource: &dyn AdmixResource,
    action: &str,
) -> Result<Claims, HttpResponse> {
    match extract_claims_from_session(session, config).await {
        Ok(claims) => {
            let user_roles: HashSet<String> = {
                let mut roles = claims.roles.clone();
                roles.push(claims.role.clone());
                roles.into_iter().collect()
            };
            
            let allowed_roles: HashSet<String> = 
                resource.allowed_roles().into_iter().collect();
            
            if user_roles.intersection(&allowed_roles).next().is_some() {
                info!("User {} has permission for {} action on resource {}", 
                      claims.email, action, resource.resource_name());
                Ok(claims)
            } else {
                warn!("User {} lacks permission for {} action on resource {} (user roles: {:?}, required: {:?})", 
                      claims.email, action, resource.resource_name(), claims.roles, resource.allowed_roles());
                Err(HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "Insufficient permissions",
                    "required_roles": resource.allowed_roles(),
                    "user_roles": claims.roles,
                    "action": action,
                    "resource": resource.resource_name()
                })))
            }
        }
        Err(_) => {
            Err(HttpResponse::Found()
                .append_header(("Location", "/adminx/login"))
                .finish())
        }
    }
}

/// Create template context for UI routes with common data
pub fn create_base_template_context(
    resource_name: &str,
    base_path: &str,
    claims: &Claims,
) -> Context {
    let mut ctx = Context::new();
    ctx.insert("resource_name", resource_name);
    ctx.insert("base_path", &format!("/adminx/{}", base_path));
    ctx.insert("menus", &get_registered_menus());
    ctx.insert("current_user", claims);
    ctx.insert("is_authenticated", &true);
    ctx
}

/// Handle form data conversion from HTML form to JSON
pub fn convert_form_data_to_json(
    form_data: std::collections::HashMap<String, String>
) -> Value {
    let mut json_data = serde_json::Map::new();
    for (key, value) in form_data {
        if !value.is_empty() { // Skip empty fields
            json_data.insert(key, serde_json::Value::String(value));
        }
    }
    serde_json::Value::Object(json_data)
}

/// Handle resource creation response and return appropriate redirect
pub fn handle_create_response(
    response: HttpResponse,
    base_path: &str,
    resource_name: &str,
) -> HttpResponse {
    if response.status().is_success() {
        info!("✅ Resource '{}' created successfully, redirecting to list", resource_name);
        let location = format!("/adminx/{}/list?success=created", base_path);
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    } else {
        error!("❌ Resource '{}' creation failed with status: {}", resource_name, response.status());
        let location = format!("/adminx/{}/new?error=create_failed", base_path);
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    }
}

/// Handle resource update response and return appropriate redirect
pub fn handle_update_response(
    response: HttpResponse,
    base_path: &str,
    item_id: &str,
    resource_name: &str,
) -> HttpResponse {
    if response.status().is_success() {
        info!("✅ Resource '{}' item '{}' updated successfully, redirecting to view", resource_name, item_id);
        let location = format!("/adminx/{}/view/{}?success=updated", base_path, item_id);
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    } else {
        error!("❌ Resource '{}' item '{}' update failed with status: {}", resource_name, item_id, response.status());
        let location = format!("/adminx/{}/edit/{}?error=update_failed", base_path, item_id);
        HttpResponse::Found()
            .append_header(("Location", location))
            .finish()
    }
}

/// Get default list structure for resources that don't define one
pub fn get_default_list_structure() -> Value {
    serde_json::json!({
        "columns": [],
        "actions": ["view", "edit", "delete"]
    })
}

/// Fetch and prepare list data directly from database
// pub async fn fetch_list_data(
//     resource: &Arc<Box<dyn AdmixResource>>,
//     req: &HttpRequest,
//     _query_string: String,
// ) -> Result<(Vec<String>, Vec<serde_json::Map<String, Value>>, Value), Box<dyn std::error::Error + Send + Sync>> {
//     let collection = resource.get_collection();
    
//     // Parse query parameters for pagination
//     let query_params: std::collections::HashMap<String, String> = 
//         serde_urlencoded::from_str(req.query_string()).unwrap_or_default();
    
//     let page: u64 = query_params.get("page")
//         .and_then(|p| p.parse().ok())
//         .unwrap_or(1);
//     let per_page: u64 = query_params.get("per_page")
//         .and_then(|p| p.parse().ok())
//         .unwrap_or(10);
    
//     let skip = (page - 1) * per_page;
    
//     // Get total count
//     let total = collection.count_documents(mongodb::bson::doc! {}, None).await
//         .unwrap_or(0);
    
//     // Fetch documents with pagination
//     let mut find_options = mongodb::options::FindOptions::default();
//     find_options.skip = Some(skip);
//     find_options.limit = Some(per_page as i64);
//     find_options.sort = Some(mongodb::bson::doc! { "created_at": -1 });
    
//     let mut cursor = collection.find(mongodb::bson::doc! {}, find_options).await
//         .map_err(|e| format!("Database query failed: {}", e))?;
    
//     let mut documents = Vec::new();
//     while let Some(doc) = cursor.try_next().await.unwrap_or(None) {
//         documents.push(doc);
//     }
    
//     // Convert MongoDB documents to the format expected by the template
//     let headers = vec![
//         "id".to_string(),
//         "name".to_string(), 
//         "email".to_string(),
//         "created_at".to_string()
//     ];
    
//     let rows: Vec<serde_json::Map<String, Value>> = documents
//         .into_iter()
//         .map(|doc| {
//             let mut row = serde_json::Map::new();
            
//             // Handle MongoDB ObjectId
//             if let Ok(oid) = doc.get_object_id("_id") {
//                 row.insert("id".to_string(), Value::String(oid.to_hex()));
//             }
            
//             // Extract other fields
//             if let Ok(name) = doc.get_str("name") {
//                 row.insert("name".to_string(), Value::String(name.to_string()));
//             }
            
//             if let Ok(email) = doc.get_str("email") {
//                 row.insert("email".to_string(), Value::String(email.to_string()));
//             }
            
//             // Handle created_at timestamp
//             if let Ok(created_at) = doc.get_datetime("created_at") {
//                 let timestamp_ms = created_at.timestamp_millis();
//                 if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
//                     row.insert("created_at".to_string(), 
//                              Value::String(datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
//                 } else {
//                     row.insert("created_at".to_string(), Value::String("N/A".to_string()));
//                 }
//             } else {
//                 row.insert("created_at".to_string(), Value::String("N/A".to_string()));
//             }
            
//             row
//         })
//         .collect();
    
//     let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 1 }; // Ceiling division
//     let pagination = serde_json::json!({
//         "current": page,
//         "total": total_pages,
//         "prev": if page > 1 { Some(page - 1) } else { None },
//         "next": if page < total_pages { Some(page + 1) } else { None }
//     });
    
//     info!("Fetched {} items for list view (page {} of {})", rows.len(), page, total_pages);
//     Ok((headers, rows, pagination))
// }

pub async fn fetch_list_data(
    resource: &Arc<Box<dyn AdmixResource>>,
    req: &HttpRequest,
    _query_string: String,
) -> Result<(Vec<String>, Vec<serde_json::Map<String, Value>>, Value), Box<dyn std::error::Error + Send + Sync>> {
    let collection = resource.get_collection();
    
    // Parse query parameters for pagination and filters
    let query_params: std::collections::HashMap<String, String> = 
        serde_urlencoded::from_str(req.query_string()).unwrap_or_default();
    
    let page: u64 = query_params.get("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let per_page: u64 = query_params.get("per_page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(10);
    
    let skip = (page - 1) * per_page;
    
    // Build filter document from query parameters
    let mut filter_doc = mongodb::bson::doc! {};
    
    // Get permitted query fields for security
    let permitted_fields: HashSet<&str> = resource.permit_keys().into_iter().collect();
    
    // Build filters based on query parameters
    for (key, value) in &query_params {
        if !value.is_empty() && permitted_fields.contains(key.as_str()) {
            match key.as_str() {
                "name" | "email" | "username" => {
                    // Text search with regex (case-insensitive)
                    filter_doc.insert(key, mongodb::bson::doc! {
                        "$regex": value,
                        "$options": "i"
                    });
                }
                "status" => {
                    // Exact match for status
                    filter_doc.insert(key, value);
                }
                _ => {
                    // Default: exact match for other fields
                    filter_doc.insert(key, value);
                }
            }
        }
    }
    
    info!("Applied filters: {:?}", filter_doc);
    
    // Get total count with filters
    let total = collection.count_documents(filter_doc.clone(), None).await
        .unwrap_or(0);
    
    // Fetch documents with pagination and filters
    let mut find_options = mongodb::options::FindOptions::default();
    find_options.skip = Some(skip);
    find_options.limit = Some(per_page as i64);
    find_options.sort = Some(mongodb::bson::doc! { "created_at": -1 });
    
    let mut cursor = collection.find(filter_doc, find_options).await
        .map_err(|e| format!("Database query failed: {}", e))?;
    
    let mut documents = Vec::new();
    while let Some(doc) = cursor.try_next().await.unwrap_or(None) {
        documents.push(doc);
    }
    
    // Convert MongoDB documents to the format expected by the template
    let headers = vec![
        "id".to_string(),
        "name".to_string(), 
        "email".to_string(),
        "created_at".to_string()
    ];
    
    let rows: Vec<serde_json::Map<String, Value>> = documents
        .into_iter()
        .map(|doc| {
            let mut row = serde_json::Map::new();
            
            // Handle MongoDB ObjectId
            if let Ok(oid) = doc.get_object_id("_id") {
                row.insert("id".to_string(), Value::String(oid.to_hex()));
            }
            
            // Extract other fields
            if let Ok(name) = doc.get_str("name") {
                row.insert("name".to_string(), Value::String(name.to_string()));
            }
            
            if let Ok(email) = doc.get_str("email") {
                row.insert("email".to_string(), Value::String(email.to_string()));
            }
            
            // Handle status if it exists
            if let Ok(status) = doc.get_str("status") {
                row.insert("status".to_string(), Value::String(status.to_string()));
            }
            
            // Handle created_at timestamp
            if let Ok(created_at) = doc.get_datetime("created_at") {
                let timestamp_ms = created_at.timestamp_millis();
                if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
                    row.insert("created_at".to_string(), 
                             Value::String(datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
                } else {
                    row.insert("created_at".to_string(), Value::String("N/A".to_string()));
                }
            } else {
                row.insert("created_at".to_string(), Value::String("N/A".to_string()));
            }
            
            row
        })
        .collect();
    
    let total_pages = if per_page > 0 { (total + per_page - 1) / per_page } else { 1 };
    
    // Build pagination with current filters
    let mut base_url = req.path().to_string();
    let mut filter_params = Vec::new();
    for (key, value) in &query_params {
        if key != "page" && !value.is_empty() {
            filter_params.push(format!("{}={}", key, urlencoding::encode(value)));
        }
    }
    let filter_string = if filter_params.is_empty() {
        String::new()
    } else {
        format!("&{}", filter_params.join("&"))
    };
    
    let pagination = serde_json::json!({
        "current": page,
        "total": total_pages,
        "prev": if page > 1 { Some(page - 1) } else { None },
        "next": if page < total_pages { Some(page + 1) } else { None },
        "filter_params": filter_string
    });
    
    info!("Fetched {} items for list view (page {} of {}) with filters", rows.len(), page, total_pages);
    Ok((headers, rows, pagination))
}

/// Get filters data and current filter values for the template
pub fn get_filters_data(
    resource: &Arc<Box<dyn AdmixResource>>,
    query_params: &std::collections::HashMap<String, String>
) -> (Option<Value>, serde_json::Map<String, Value>) {
    let filters = resource.filters();
    let mut current_filters = serde_json::Map::new();
    
    // Extract current filter values from query parameters
    if let Some(filter_config) = &filters {
        if let Some(filter_array) = filter_config.get("filters").and_then(|f| f.as_array()) {
            for filter in filter_array {
                if let Some(field) = filter.get("field").and_then(|f| f.as_str()) {
                    if let Some(value) = query_params.get(field) {
                        if !value.is_empty() {
                            current_filters.insert(field.to_string(), Value::String(value.clone()));
                        }
                    }
                    
                    // Handle range filters (date_range, number_range)
                    let from_key = format!("{}_from", field);
                    let to_key = format!("{}_to", field);
                    let min_key = format!("{}_min", field);
                    let max_key = format!("{}_max", field);
                    
                    if let Some(from_value) = query_params.get(&from_key) {
                        if !from_value.is_empty() {
                            current_filters.insert(from_key, Value::String(from_value.clone()));
                        }
                    }
                    
                    if let Some(to_value) = query_params.get(&to_key) {
                        if !to_value.is_empty() {
                            current_filters.insert(to_key, Value::String(to_value.clone()));
                        }
                    }
                    
                    if let Some(min_value) = query_params.get(&min_key) {
                        if !min_value.is_empty() {
                            current_filters.insert(min_key, Value::String(min_value.clone()));
                        }
                    }
                    
                    if let Some(max_value) = query_params.get(&max_key) {
                        if !max_value.is_empty() {
                            current_filters.insert(max_key, Value::String(max_value.clone()));
                        }
                    }
                }
            }
        }
    }
    
    (filters, current_filters)
}



/// Fetch single item data for view/edit pages
pub async fn fetch_single_item_data(
    resource: &Arc<Box<dyn AdmixResource>>,
    req: &HttpRequest,
    id: &str,
) -> Result<serde_json::Map<String, Value>, Box<dyn std::error::Error + Send + Sync>> {
    let collection = resource.get_collection();
    
    // Parse ObjectId
    let oid = mongodb::bson::oid::ObjectId::parse_str(id)
        .map_err(|e| format!("Invalid ObjectId: {}", e))?;
    
    // Find the document
    let doc = collection.find_one(mongodb::bson::doc! { "_id": oid }, None).await
        .map_err(|e| format!("Database query failed: {}", e))?
        .ok_or("Document not found")?;
    
    // Convert to template-friendly format
    let mut record = serde_json::Map::new();
    
    // Handle MongoDB ObjectId
    if let Ok(oid) = doc.get_object_id("_id") {
        record.insert("id".to_string(), Value::String(oid.to_hex()));
    }
    
    // Extract other fields
    if let Ok(name) = doc.get_str("name") {
        record.insert("name".to_string(), Value::String(name.to_string()));
    }
    
    if let Ok(email) = doc.get_str("email") {
        record.insert("email".to_string(), Value::String(email.to_string()));
    }
    
    // Handle created_at timestamp
    if let Ok(created_at) = doc.get_datetime("created_at") {
        let timestamp_ms = created_at.timestamp_millis();
        if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
            record.insert("created_at".to_string(), 
                         Value::String(datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
        } else {
            record.insert("created_at".to_string(), Value::String("N/A".to_string()));
        }
    }
    
    // Handle updated_at timestamp
    if let Ok(updated_at) = doc.get_datetime("updated_at") {
        let timestamp_ms = updated_at.timestamp_millis();
        if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
            record.insert("updated_at".to_string(), 
                         Value::String(datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
        } else {
            record.insert("updated_at".to_string(), Value::String("N/A".to_string()));
        }
    }
    
    info!("Fetched single item with id: {} for resource: {}", id, resource.resource_name());
    Ok(record)
}
pub fn get_default_form_structure() -> Value {
    serde_json::json!({
        "groups": [
            {
                "title": "Details",
                "fields": []
            }
        ]
    })
}

/// Get default view structure for resources that don't define one
pub fn get_default_view_structure() -> Value {
    serde_json::json!({
        "sections": [
            {
                "title": "Details",
                "fields": []
            }
        ]
    })
}

/// Register API-only routes without UI components
pub fn register_api_only_routes(resource: Box<dyn AdmixResource>) -> Scope {
    let resource_name = resource.resource_name().to_string();
    info!("Registering API-only routes for resource: {}", resource_name);
    
    let mut scope = web::scope("");

    // GET / - List all items
    let list_resource = resource.clone_box();
    scope = scope.route("", web::get().to(move |req: HttpRequest, query: web::Query<String>| {
        let resource = list_resource.clone_box();
        async move { 
            info!("📡 List API endpoint called for resource: {}", resource.resource_name());
            resource.list(&req, query.into_inner()).await 
        }
    }));

    // POST / - Create new item
    let create_resource = resource.clone_box();
    scope = scope.route("", web::post().to(move |req: HttpRequest, body: web::Json<Value>| {
        let resource = create_resource.clone_box();
        async move { 
            info!("📡 Create API endpoint called for resource: {}", resource.resource_name());
            resource.create(&req, body.into_inner()).await 
        }
    }));

    // GET /{id} - Get single item
    let get_resource = resource.clone_box();
    scope = scope.route("/{id}", web::get().to(move |req: HttpRequest, path: web::Path<String>| {
        let resource = get_resource.clone_box();
        async move { 
            let id = path.into_inner();
            info!("📡 Get API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
            resource.get(&req, id).await 
        }
    }));

    // PUT /{id} - Update item
    let update_resource = resource.clone_box();
    scope = scope.route("/{id}", web::put().to(move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>| {
        let resource = update_resource.clone_box();
        async move { 
            let id = path.into_inner();
            info!("📡 Update API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
            resource.update(&req, id, body.into_inner()).await 
        }
    }));

    // DELETE /{id} - Delete item
    let delete_resource = resource.clone_box();
    scope = scope.route("/{id}", web::delete().to(move |req: HttpRequest, path: web::Path<String>| {
        let resource = delete_resource.clone_box();
        async move { 
            let id = path.into_inner();
            info!("📡 Delete API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
            resource.delete(&req, id).await 
        }
    }));

    // Add custom actions
    for action in resource.custom_actions() {
        let path = format!("/{{id}}/{}", action.name);
        info!("Adding custom action: {} {} for resource: {}", action.method, path, resource_name);
        
        match action.method {
            "POST" => {
                scope = scope.route(&path, web::post().to(action.handler));
            }
            "GET" => {
                scope = scope.route(&path, web::get().to(action.handler));
            }
            "PUT" => {
                scope = scope.route(&path, web::put().to(action.handler));
            }
            "DELETE" => {
                scope = scope.route(&path, web::delete().to(action.handler));
            }
            "PATCH" => {
                scope = scope.route(&path, web::patch().to(action.handler));
            }
            method => {
                error!("Unsupported HTTP method: {} for action: {} in resource: {}", method, action.name, resource_name);
            }
        }
    }

    scope
}

/// Register protected routes with role-based access control
pub fn register_protected_resource_routes(resource: Box<dyn AdmixResource>) -> Scope {
    let resource_name = resource.resource_name().to_string();
    let allowed_roles = resource.allowed_roles();
    
    info!("Registering protected routes for resource: {} with roles: {:?}", resource_name, allowed_roles);
    
    let mut scope = web::scope("");

    // GET / - List with role check
    let list_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::get().to(move |req: HttpRequest, query: web::Query<String>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = list_resource.clone_box();
            async move {
                match check_resource_permission(&session, &config, resource.as_ref(), "list").await {
                    Ok(_claims) => resource.list(&req, query.into_inner()).await,
                    Err(response) => response,
                }
            }
        }),
    );

    // POST / - Create with role check
    let create_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::post().to(move |req: HttpRequest, body: web::Json<Value>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = create_resource.clone_box();
            async move {
                match check_resource_permission(&session, &config, resource.as_ref(), "create").await {
                    Ok(_claims) => resource.create(&req, body.into_inner()).await,
                    Err(response) => response,
                }
            }
        }),
    );

    // GET /{id} - Get with role check
    let get_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::get().to(move |req: HttpRequest, path: web::Path<String>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = get_resource.clone_box();
            async move {
                let id = path.into_inner();
                match check_resource_permission(&session, &config, resource.as_ref(), "view").await {
                    Ok(_claims) => resource.get(&req, id).await,
                    Err(response) => response,
                }
            }
        }),
    );

    // PUT /{id} - Update with role check
    let update_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::put().to(move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = update_resource.clone_box();
            async move {
                let id = path.into_inner();
                match check_resource_permission(&session, &config, resource.as_ref(), "update").await {
                    Ok(_claims) => resource.update(&req, id, body.into_inner()).await,
                    Err(response) => response,
                }
            }
        }),
    );

    // DELETE /{id} - Delete with role check
    let delete_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::delete().to(move |req: HttpRequest, path: web::Path<String>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = delete_resource.clone_box();
            async move {
                let id = path.into_inner();
                match check_resource_permission(&session, &config, resource.as_ref(), "delete").await {
                    Ok(_claims) => resource.delete(&req, id).await,
                    Err(response) => response,
                }
            }
        }),
    );

    scope
}