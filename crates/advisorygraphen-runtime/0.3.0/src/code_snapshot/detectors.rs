use super::*;

pub(super) fn http_methods(contents: &str) -> Vec<String> {
    ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"]
        .into_iter()
        .filter(|method| {
            contents.contains(&format!("function {method}"))
                || contents.contains(&format!("const {method}"))
                || contents.contains(&format!("export async function {method}"))
                || contents.contains(&format!("export function {method}"))
        })
        .map(str::to_string)
        .collect()
}

pub(super) fn route_path(relative_path: &str) -> String {
    let route = relative_path
        .trim_start_matches("src/")
        .trim_start_matches("app/api/")
        .trim_end_matches("/route.ts")
        .trim_end_matches("/route.tsx")
        .trim_end_matches("/route.js")
        .trim_end_matches("/route.jsx");
    format!("/api/{route}")
}

pub(super) fn has_auth_check(contents: &str) -> bool {
    [
        "auth(",
        "getServerSession",
        "requireAuth",
        "currentUser",
        "organization_memberships",
        "verifyToken",
        "withAuth",
    ]
    .iter()
    .any(|needle| contents.contains(needle))
}

pub(super) fn has_db_access(file: &CodeFile) -> bool {
    !db_detectors(&file.contents).is_empty()
        || file.relative_path.contains("/database/")
        || file.relative_path.contains("/db/")
}

pub(super) fn db_detectors(contents: &str) -> Vec<String> {
    [
        ("prisma", "prisma."),
        ("sql_tag", "sql`"),
        ("query_call", ".query("),
        ("execute_call", ".execute("),
        ("supabase", "supabase."),
        ("database_client", "database"),
        ("db_client", "db."),
    ]
    .into_iter()
    .filter(|(_, needle)| contents.contains(needle))
    .map(|(name, _)| name.to_string())
    .collect()
}

pub(super) fn env_var_names(contents: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    for segment in contents.split("process.env.").skip(1) {
        let name = segment
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect::<String>();
        if !name.is_empty() {
            names.insert(name);
        }
    }
    names.into_iter().collect()
}

pub(super) fn language_for_path(relative_path: &str) -> &str {
    match Path::new(relative_path)
        .extension()
        .and_then(|value| value.to_str())
    {
        Some("ts") | Some("tsx") => "typescript",
        Some("js" | "jsx") => "javascript",
        Some("json") => "json",
        _ => "unknown",
    }
}

pub(super) fn coverage_json(coverage: &Coverage) -> Value {
    json!({
        "parsed_files": coverage.parsed_files,
        "skipped_files": coverage.skipped_files,
        "unsupported_extensions": coverage.unsupported_extensions,
        "api_route_files": coverage.api_route_files,
        "test_files": coverage.test_files,
        "db_access_files": coverage.db_access_files,
        "env_usage_files": coverage.env_usage_files,
        "confidence_model": {
            "file_detection": "high",
            "api_route_detection": "high_for_nextjs_app_router_paths",
            "db_access_detection": "medium_lexical",
            "auth_detection": "medium_lexical",
            "env_usage_detection": "high_for_process_env_dot_access"
        }
    })
}
