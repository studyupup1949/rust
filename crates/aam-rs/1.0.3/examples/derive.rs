use aam_rs::aaml::AAML;
use std::path::Path;

fn main() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    std::env::set_current_dir(&examples_dir).expect("Cannot change dir to examples/");

    let child_path = examples_dir.join("derive_child.aam");

    let config = match AAML::load(&child_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            return;
        }
    };

    println!("=== AAM Derive Example ===\n");

    println!("--- Child-specific keys ---");
    println!("author  = {}", config.find_obj("author").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));
    println!("version = {}", config.find_obj("version").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));

    println!("\n--- Overridden keys (child wins) ---");
    println!("log_level = {} (base was: info)", config.find_obj("log_level").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));
    println!("theme     = {} (base was: light)", config.find_obj("theme").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));

    println!("\n--- Inherited keys from derive_base.aam ---");
    println!("app_name    = {}", config.find_obj("app_name").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));
    println!("max_retries = {}", config.find_obj("max_retries").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));
    println!("timeout     = {}", config.find_obj("timeout").map(|v| v.to_string()).unwrap_or_else(|| "<not found>".into()));

    println!("\n--- Inherited schemas ---");
    if let Some(entity_schema) = config.get_schema("Entity") {
        println!("Schema 'Entity' fields:");
        let mut fields: Vec<_> = entity_schema.fields.iter().collect();
        fields.sort_by_key(|(k, _)| k.as_str());
        for (field, ty) in fields {
            println!("  {field}: {ty}");
        }
    } else {
        println!("Schema 'Entity' not found");
    }

    println!("\n--- Child-defined schemas ---");
    if let Some(plugin_schema) = config.get_schema("Plugin") {
        println!("Schema 'Plugin' fields:");
        let mut fields: Vec<_> = plugin_schema.fields.iter().collect();
        fields.sort_by_key(|(k, _)| k.as_str());
        for (field, ty) in fields {
            println!("  {field}: {ty}");
        }
    } else {
        println!("Schema 'Plugin' not found");
    }

    println!("\nDone!");
}
