// crates/adminx/src/helpers/downloads/csv_download.rs
use actix_web::{HttpRequest, HttpResponse};
use std::sync::Arc;
use tracing::{info, error};
use std::collections::HashSet;
use futures::TryStreamExt;
use crate::AdmixResource;
use chrono::Utc;

/// Export ALL data as CSV (ignoring pagination, respecting filters)
pub async fn export_data_as_csv(
    resource: &Arc<Box<dyn AdmixResource>>,
    req: &HttpRequest,
    _query_string: String,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let collection = resource.get_collection();
    
    // Parse query parameters for filters (same as JSON export)
    let query_params: std::collections::HashMap<String, String> = 
        serde_urlencoded::from_str(req.query_string()).unwrap_or_default();
    
    // Build filter document (same logic as JSON export)
    let mut filter_doc = mongodb::bson::doc! {};
    let permitted_fields: HashSet<&str> = resource.permit_keys().into_iter().collect();
    
    for (key, value) in &query_params {
        if !value.is_empty() && (permitted_fields.contains(key.as_str()) || key == "search") && key != "download" {
            match key.as_str() {
                "name" | "email" | "username" | "key" | "title" | "description" | "search" => {
                    if key == "search" {
                        let search_fields = vec!["name", "email", "username", "key", "title", "description"];
                        let mut search_conditions = Vec::new();
                        
                        for field in search_fields {
                            if permitted_fields.contains(field) {
                                search_conditions.push(mongodb::bson::doc! {
                                    field: {
                                        "$regex": value,
                                        "$options": "i"
                                    }
                                });
                            }
                        }
                        
                        if !search_conditions.is_empty() {
                            filter_doc.insert("$or", search_conditions);
                        }
                    } else {
                        filter_doc.insert(key, mongodb::bson::doc! {
                            "$regex": value,
                            "$options": "i"
                        });
                    }
                }
                "status" | "data_type" | "deleted" | "active" | "enabled" => {
                    if value == "true" || value == "false" {
                        let bool_val = value == "true";
                        filter_doc.insert(key, bool_val);
                    } else {
                        filter_doc.insert(key, value);
                    }
                }
                _ => {
                    filter_doc.insert(key, value);
                }
            }
        }
    }
    
    info!("Exporting CSV with filters: {:?}", filter_doc);
    
    // Fetch ALL documents
    let mut find_options = mongodb::options::FindOptions::default();
    find_options.sort = Some(mongodb::bson::doc! { "created_at": -1 });
    
    let mut cursor = collection.find(filter_doc, find_options).await
        .map_err(|e| format!("Database query failed: {}", e))?;
    
    // Build CSV headers
    let mut headers = vec!["id".to_string()];
    for field in resource.permit_keys() {
        headers.push(field.to_string());
    }
    headers.push("created_at".to_string());
    headers.push("updated_at".to_string());
    
    // Start building CSV content
    let mut csv_content = headers.join(",") + "\n";
    
    let mut record_count = 0;
    while let Some(doc) = cursor.try_next().await.unwrap_or(None) {
        let mut row = Vec::new();
        
        // Add ID
        if let Ok(oid) = doc.get_object_id("_id") {
            row.push(escape_csv_field(&oid.to_hex()));
        } else {
            row.push("".to_string());
        }
        
        // Add permitted fields
        for field_name in resource.permit_keys() {
            let field_value = if let Some(bson_val) = doc.get(field_name) {
                match bson_val {
                    mongodb::bson::Bson::String(s) => escape_csv_field(s),
                    mongodb::bson::Bson::Boolean(b) => b.to_string(),
                    mongodb::bson::Bson::Int32(i) => i.to_string(),
                    mongodb::bson::Bson::Int64(i) => i.to_string(),
                    mongodb::bson::Bson::Double(d) => d.to_string(),
                    mongodb::bson::Bson::DateTime(dt) => {
                        let timestamp_ms = dt.timestamp_millis();
                        if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
                            escape_csv_field(&datetime.format("%Y-%m-%d %H:%M:%S").to_string())
                        } else {
                            "".to_string()
                        }
                    }
                    mongodb::bson::Bson::Null => "".to_string(),
                    _ => escape_csv_field(&format!("{:?}", bson_val)),
                }
            } else {
                "".to_string()
            };
            row.push(field_value);
        }
        
        // Add timestamps
        if let Ok(created_at) = doc.get_datetime("created_at") {
            let timestamp_ms = created_at.timestamp_millis();
            if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
                row.push(escape_csv_field(&datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
            } else {
                row.push("".to_string());
            }
        } else {
            row.push("".to_string());
        }
        
        if let Ok(updated_at) = doc.get_datetime("updated_at") {
            let timestamp_ms = updated_at.timestamp_millis();
            if let Some(datetime) = chrono::DateTime::from_timestamp_millis(timestamp_ms) {
                row.push(escape_csv_field(&datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
            } else {
                row.push("".to_string());
            }
        } else {
            row.push("".to_string());
        }
        
        csv_content.push_str(&(row.join(",") + "\n"));
        record_count += 1;
    }
    
    let filename = format!("{}_{}.csv", 
                          resource.resource_name(), 
                          Utc::now().format("%Y%m%d_%H%M%S"));
    
    info!("✅ Exported {} records as CSV", record_count);
    
    Ok(HttpResponse::Ok()
        .content_type("text/csv")
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(csv_content))
}

/// Helper function to properly escape CSV fields
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}