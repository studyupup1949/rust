use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

fn main() {
    let schema_path = Path::new("schema.gql");
    if !schema_path.exists() {
        eprintln!();
        eprintln!("========================================");
        eprintln!("ERROR: schema.gql not found!");
        eprintln!();
        eprintln!("Generate it by running from src/mangrove/:");
        eprintln!(
            "  cargo run -p concorde --bin build-gql-schema 2>/dev/null > libs/adaptive-client-rust/schema.gql"
        );
        eprintln!("========================================");
        eprintln!();
        std::process::exit(1);
    }
    println!("cargo::rerun-if-changed=schema.gql");

    let content = std::fs::read_to_string(schema_path).expect("Failed to read schema.gql");
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = hasher.finish();
    println!("cargo::rustc-cfg=schema_hash_{hash:x}");

    // Also track the GraphQL query files that graphql_client proc macros read
    for entry in std::fs::read_dir("src/graphql").into_iter().flatten() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "graphql") {
                println!("cargo::rerun-if-changed={}", path.display());
            }
        }
    }
}
