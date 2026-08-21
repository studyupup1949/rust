// crates/adminx/src/controllers/resource_controller.rs - UPDATED VERSION
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use serde_json::Value;
use std::sync::Arc;
use tera::Context;
use tracing::{info, warn, error};
use actix_session::Session;

use crate::AdmixResource;
use crate::helpers::form_helper::extract_fields_for_form;
use crate::helpers::template_helper::render_template;
use crate::helpers::resource_helper::{
    check_authentication,
    create_base_template_context,
    convert_form_data_to_json,
    handle_create_response,
    handle_update_response,
    get_default_list_structure,
    get_default_form_structure,
    get_default_view_structure,
    fetch_list_data,
    fetch_single_item_data,
};
use crate::configs::initializer::AdminxConfig;

/// Register all UI + API routes for a resource
pub fn register_admix_resource_routes(resource: Box<dyn AdmixResource>) -> Scope {
    let base_path = resource.base_path().to_string();
    let resource_name = resource.resource_name().to_string();
    
    info!("Registering routes for resource: {} at path: {}", resource_name, base_path);
    
    let mut scope = web::scope("");

    // ========================
    // UI Routes (HTML pages) - REGISTER THESE FIRST!
    // ========================

    let resource_arc = Arc::new(resource.clone_box());
    let ui_resource_name = resource_arc.resource_name().to_string();
    let ui_base_path = resource_arc.base_path().to_string();

    // GET /list - List view page
    // scope = scope.route("/list", web::get().to({
    //     let resource = Arc::clone(&resource_arc);
    //     let resource_name = ui_resource_name.clone();
    //     move |req: HttpRequest, session: Session, config: web::Data<AdminxConfig>| {
    //         let query_string = req.query_string().to_string();
    //         let resource = Arc::clone(&resource);
    //         let resource_name = resource_name.clone();
    //         async move {
    //             match check_authentication(&session, &config, &resource_name, "list").await {
    //                 Ok(claims) => {
    //                     info!("✅ List UI accessed by: {} for resource: {}", claims.email, resource_name);
                        
    //                     let mut ctx = create_base_template_context(&resource_name, &resource.base_path(), &claims);
                        
    //                     // Check for success/error messages from query parameters
    //                     let query_params: std::collections::HashMap<String, String> = 
    //                         serde_urlencoded::from_str(&query_string).unwrap_or_default();
                        
    //                     if query_params.contains_key("success") {
    //                         match query_params.get("success").unwrap().as_str() {
    //                             "created" => ctx.insert("toast_message", &"Successfully created new item!"),
    //                             "updated" => ctx.insert("toast_message", &"Successfully updated item!"),
    //                             "deleted" => ctx.insert("toast_message", &"Successfully deleted item!"),
    //                             _ => {}
    //                         }
    //                         ctx.insert("toast_type", &"success");
    //                     }
                        
    //                     if query_params.contains_key("error") {
    //                         match query_params.get("error").unwrap().as_str() {
    //                             "create_failed" => ctx.insert("toast_message", &"Failed to create item. Please try again."),
    //                             "update_failed" => ctx.insert("toast_message", &"Failed to update item. Please try again."),
    //                             "delete_failed" => ctx.insert("toast_message", &"Failed to delete item. Please try again."),
    //                             _ => {}
    //                         }
    //                         ctx.insert("toast_type", &"error");
    //                     }
                        
    //                     // Fetch actual data from the resource
    //                     match fetch_list_data(&resource, &req, query_string).await {
    //                         Ok((headers, rows, pagination)) => {
    //                             ctx.insert("headers", &headers);
    //                             ctx.insert("rows", &rows);
    //                             ctx.insert("pagination", &pagination);
                                
    //                             info!("📊 Loaded {} items for {} list view", rows.len(), resource_name);
    //                         }
    //                         Err(e) => {
    //                             error!("❌ Failed to fetch list data for {}: {}", resource_name, e);
    //                             // Provide empty data as fallback
    //                             let headers = vec!["id", "name", "email", "created_at"];
    //                             let rows: Vec<serde_json::Map<String, serde_json::Value>> = vec![];
    //                             let pagination = serde_json::json!({
    //                                 "current": 1,
    //                                 "total": 1,
    //                                 "prev": null,
    //                                 "next": null
    //                             });
                                
    //                             ctx.insert("headers", &headers);
    //                             ctx.insert("rows", &rows);
    //                             ctx.insert("pagination", &pagination);
    //                             ctx.insert("toast_message", &"Failed to load data. Please refresh the page.");
    //                             ctx.insert("toast_type", &"error");
    //                         }
    //                     }

    //                     render_template("list.html.tera", ctx).await
    //                 }
    //                 Err(response) => response
    //             }
    //         }
    //     }
    // }));
    scope = scope.route("/list", web::get().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        move |req: HttpRequest, session: Session, config: web::Data<AdminxConfig>| {
            let query_string = req.query_string().to_string();
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "list").await {
                    Ok(claims) => {
                        info!("✅ List UI accessed by: {} for resource: {}", claims.email, resource_name);
                        
                        let mut ctx = create_base_template_context(&resource_name, &resource.base_path(), &claims);
                        
                        // Parse query parameters
                        let query_params: std::collections::HashMap<String, String> = 
                            serde_urlencoded::from_str(&query_string).unwrap_or_default();
                        
                        // Check for success/error messages from query parameters
                        if query_params.contains_key("success") {
                            match query_params.get("success").unwrap().as_str() {
                                "created" => ctx.insert("toast_message", &"Successfully created new item!"),
                                "updated" => ctx.insert("toast_message", &"Successfully updated item!"),
                                "deleted" => ctx.insert("toast_message", &"Successfully deleted item!"),
                                _ => {}
                            }
                            ctx.insert("toast_type", &"success");
                        }
                        
                        if query_params.contains_key("error") {
                            match query_params.get("error").unwrap().as_str() {
                                "create_failed" => ctx.insert("toast_message", &"Failed to create item. Please try again."),
                                "update_failed" => ctx.insert("toast_message", &"Failed to update item. Please try again."),
                                "delete_failed" => ctx.insert("toast_message", &"Failed to delete item. Please try again."),
                                _ => {}
                            }
                            ctx.insert("toast_type", &"error");
                        }
                        
                        // Get filters configuration and current values
                        let (filters, current_filters) = crate::helpers::resource_helper::get_filters_data(&resource, &query_params);
                        ctx.insert("filters", &filters);
                        ctx.insert("current_filters", &current_filters);
                        ctx.insert("has_active_filters", &(!current_filters.is_empty()));
                        
                        // Fetch actual data from the resource (with filters applied)
                        match fetch_list_data(&resource, &req, query_string).await {
                            Ok((headers, rows, pagination)) => {
                                ctx.insert("headers", &headers);
                                ctx.insert("rows", &rows);
                                ctx.insert("pagination", &pagination);
                                
                                info!("📊 Loaded {} items for {} list view", rows.len(), resource_name);
                            }
                            Err(e) => {
                                error!("❌ Failed to fetch list data for {}: {}", resource_name, e);
                                // Provide empty data as fallback
                                let headers = vec!["id", "name", "email", "created_at"];
                                let rows: Vec<serde_json::Map<String, serde_json::Value>> = vec![];
                                let pagination = serde_json::json!({
                                    "current": 1,
                                    "total": 1,
                                    "prev": null,
                                    "next": null,
                                    "filter_params": ""
                                });
                                
                                ctx.insert("headers", &headers);
                                ctx.insert("rows", &rows);
                                ctx.insert("pagination", &pagination);
                                ctx.insert("toast_message", &"Failed to load data. Please refresh the page.");
                                ctx.insert("toast_type", &"error");
                            }
                        }

                        render_template("list.html.tera", ctx).await
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // GET /new - New item form page
    scope = scope.route("/new", web::get().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        let base_path = ui_base_path.clone();
        move |_req: HttpRequest, session: Session, config: web::Data<AdminxConfig>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            let base_path = base_path.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "create").await {
                    Ok(claims) => {
                        info!("✅ New form UI accessed by: {} for resource: {}", claims.email, resource_name);
                        
                        let form = resource.form_structure()
                            .unwrap_or_else(|| {
                                warn!("No form structure defined for resource: {}", resource_name);
                                get_default_form_structure()
                            });

                        let mut ctx = create_base_template_context(&resource_name, &base_path, &claims);
                        ctx.insert("fields", &extract_fields_for_form(&form));
                        ctx.insert("form_structure", &form);
                        ctx.insert("form", &form);
                        ctx.insert("is_edit_mode", &false);

                        render_template("new.html.tera", ctx).await
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // GET /view/{id} - View single item page
    scope = scope.route("/view/{id}", web::get().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        move |req: HttpRequest, id: web::Path<String>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "view").await {
                    Ok(claims) => {
                        let item_id = id.into_inner();
                        info!("✅ View UI accessed by: {} for resource: {} item: {}", claims.email, resource_name, item_id);
                        
                        let mut ctx = create_base_template_context(&resource_name, &resource.base_path(), &claims);
                        
                        // Check for success messages from query parameters
                        let query_params: std::collections::HashMap<String, String> = 
                            serde_urlencoded::from_str(&req.query_string()).unwrap_or_default();
                        
                        if query_params.contains_key("success") {
                            match query_params.get("success").unwrap().as_str() {
                                "updated" => ctx.insert("toast_message", &"Successfully updated item!"),
                                _ => {}
                            }
                            ctx.insert("toast_type", &"success");
                        }
                        
                        // Fetch the actual record data
                        match fetch_single_item_data(&resource, &req, &item_id).await {
                            Ok(record) => {
                                let view_structure = resource.view_structure()
                                    .unwrap_or_else(|| get_default_view_structure());

                                ctx.insert("view_structure", &view_structure);
                                ctx.insert("item_id", &item_id);
                                ctx.insert("record", &record);
                                
                                render_template("view.html.tera", ctx).await
                            }
                            Err(e) => {
                                error!("❌ Failed to fetch item {} for {}: {}", item_id, resource_name, e);
                                HttpResponse::NotFound().body(format!("Item not found: {}", e))
                            }
                        }
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // GET /edit/{id} - Edit item form page
    scope = scope.route("/edit/{id}", web::get().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        let base_path = ui_base_path.clone();
        move |_req: HttpRequest, id: web::Path<String>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            let base_path = base_path.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "edit").await {
                    Ok(claims) => {
                        let item_id = id.into_inner();
                        info!("✅ Edit form UI accessed by: {} for resource: {} item: {}", claims.email, resource_name, item_id);
                        
                        let mut ctx = create_base_template_context(&resource_name, &base_path, &claims);
                        
                        // Fetch the actual record data for editing
                        let req = actix_web::test::TestRequest::get().to_http_request();
                        match fetch_single_item_data(&resource, &req, &item_id).await {
                            Ok(record) => {
                                let form = resource.form_structure()
                                    .unwrap_or_else(|| get_default_form_structure());

                                ctx.insert("fields", &extract_fields_for_form(&form));
                                ctx.insert("form_structure", &form);
                                ctx.insert("form", &form);
                                ctx.insert("item_id", &item_id);
                                ctx.insert("is_edit_mode", &true);
                                ctx.insert("record", &record);

                                render_template("edit.html.tera", ctx).await
                            }
                            Err(e) => {
                                error!("❌ Failed to fetch item {} for edit: {}", item_id, e);
                                HttpResponse::NotFound().body(format!("Item not found: {}", e))
                            }
                        }
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // POST /create - Handle HTML form submission for new items
    scope = scope.route("/create", web::post().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        move |req: HttpRequest, form_data: web::Form<std::collections::HashMap<String, String>>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "create").await {
                    Ok(claims) => {
                        info!("✅ Create form submitted by: {} for resource: {}", claims.email, resource_name);
                        
                        let json_payload = convert_form_data_to_json(form_data.into_inner());
                        tracing::debug!("Converted form data to JSON: {:?}", json_payload);
                        
                        let create_response = resource.create(&req, json_payload).await;
                        handle_create_response(create_response, &resource.base_path(), &resource_name)
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // POST /update/{id} - Handle HTML form submission for updates
    scope = scope.route("/update/{id}", web::post().to({
        let resource = Arc::clone(&resource_arc);
        let resource_name = ui_resource_name.clone();
        move |req: HttpRequest, id: web::Path<String>, form_data: web::Form<std::collections::HashMap<String, String>>, session: Session, config: web::Data<AdminxConfig>| {
            let resource = Arc::clone(&resource);
            let resource_name = resource_name.clone();
            async move {
                match check_authentication(&session, &config, &resource_name, "update").await {
                    Ok(claims) => {
                        let item_id = id.into_inner();
                        info!("✅ Update form submitted by: {} for resource: {} item: {}", claims.email, resource_name, item_id);
                        
                        let json_payload = convert_form_data_to_json(form_data.into_inner());
                        tracing::debug!("Converted form data to JSON: {:?}", json_payload);
                        
                        let update_response = resource.update(&req, item_id.clone(), json_payload).await;
                        handle_update_response(update_response, &resource.base_path(), &item_id, &resource_name)
                    }
                    Err(response) => response
                }
            }
        }
    }));

    // ========================
    // API Routes (JSON endpoints) - AFTER UI ROUTES!
    // ========================
    
    // GET / - List all items (JSON API)
    let list_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::get().to(move |req: HttpRequest, query: web::Query<String>| {
            let resource = list_resource.clone_box();
            async move {
                info!("📡 List API endpoint called for resource: {}", resource.resource_name());
                resource.list(&req, query.into_inner()).await
            }
        }),
    );

    // POST / - Create new item (JSON API)
    let create_resource = resource.clone_box();
    scope = scope.route(
        "",
        web::post().to(move |req: HttpRequest, body: web::Json<Value>| {
            let resource = create_resource.clone_box();
            async move {
                info!("📡 Create API endpoint called for resource: {}", resource.resource_name());
                resource.create(&req, body.into_inner()).await
            }
        }),
    );

    // GET /{id} - Get single item (JSON API) - AFTER specific UI routes
    let get_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = get_resource.clone_box();
            async move {
                let id = path.into_inner();
                info!("📡 Get API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
                resource.get(&req, id).await
            }
        }),
    );

    // PUT /{id} - Update item (JSON API)
    let update_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::put().to(move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>| {
            let resource = update_resource.clone_box();
            async move {
                let id = path.into_inner();
                info!("📡 Update API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
                resource.update(&req, id, body.into_inner()).await
            }
        }),
    );

    // DELETE /{id} - Delete item (JSON API)
    let delete_resource = resource.clone_box();
    scope = scope.route(
        "/{id}",
        web::delete().to(move |req: HttpRequest, path: web::Path<String>| {
            let resource = delete_resource.clone_box();
            async move {
                let id = path.into_inner();
                info!("📡 Delete API endpoint called for resource: {} with id: {}", resource.resource_name(), id);
                resource.delete(&req, id).await
            }
        }),
    );

    // ========================
    // Custom Actions
    // ========================
    for action in resource_arc.custom_actions() {
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

    info!("✅ Successfully registered all routes for resource: {}", resource_name);
    scope
}