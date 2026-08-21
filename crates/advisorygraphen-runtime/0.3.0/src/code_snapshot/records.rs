use super::*;

pub(super) fn code_source(file: &CodeFile, captured_at: &str) -> Value {
    json!({
        "id": file.source_id,
        "source_type": match file.kind {
            CodeFileKind::Manifest => "code_manifest",
            CodeFileKind::ApiRoute => "api_route_file",
            CodeFileKind::Test => "test_file",
            CodeFileKind::Source => "code_file"
        },
        "title": file.relative_path,
        "uri": file.relative_path,
        "captured_at": captured_at,
        "classification": "public",
        "metadata": {
            "relative_path": file.relative_path,
            "byte_len": file.contents.len(),
            "language": language_for_path(&file.relative_path)
        }
    })
}

pub(super) fn code_records(files: &[CodeFile], coverage: &mut Coverage) -> Vec<Value> {
    let mut records = Vec::new();
    if let Some(package_file) = files
        .iter()
        .find(|file| file.relative_path == "package.json")
    {
        records.push(record_owned(OwnedRecordSpec {
            id: "record:package-manifest".to_string(),
            record_type: "component".to_string(),
            title: "Node package manifest".to_string(),
            summary: "package.json declares the JavaScript/TypeScript application boundary."
                .to_string(),
            source_ids: vec![package_file.source_id.clone()],
            context_hints: vec!["code".to_string(), "manifest".to_string()],
            relation: None,
            metadata: json!({"component_type": "manifest"}),
        }));
    }

    let has_any_db_access = files.iter().any(has_db_access);
    if has_any_db_access {
        records.push(record(RecordSpec {
            id: "record:application-database",
            record_type: "data_store",
            title: "Application database",
            summary:
                "Detected database access through Prisma, SQL, database service, or query helpers.",
            source_ids: &[],
            context_hints: &["code", "data"],
            relation: None,
            metadata: json!({"store_type": "database", "confidence": "medium"}),
        }));
    }

    for file in files {
        if has_db_access(file) {
            coverage.db_access_files += 1;
        }
        if !env_var_names(&file.contents).is_empty() {
            coverage.env_usage_files += 1;
        }
        match file.kind {
            CodeFileKind::ApiRoute => records.extend(api_route_records(file, has_any_db_access)),
            CodeFileKind::Test => records.push(test_record(file)),
            CodeFileKind::Manifest | CodeFileKind::Source => {}
        }
        records.extend(env_records(file));
    }

    let mut seen = BTreeSet::new();
    records.retain(|record| {
        record
            .get("id")
            .and_then(Value::as_str)
            .map(|id| seen.insert(id.to_string()))
            .unwrap_or(false)
    });
    records
}

pub(super) fn api_route_records(file: &CodeFile, has_db_store: bool) -> Vec<Value> {
    let route_id = format!("record:api-route-{}", path_slug(&file.relative_path));
    let methods = http_methods(&file.contents);
    let mut records = vec![record_owned(OwnedRecordSpec {
        id: route_id.clone(),
        record_type: "component".to_string(),
        title: format!("API route {}", route_path(&file.relative_path)),
        summary: format!(
            "Next.js API route exposing {}.",
            if methods.is_empty() {
                "an unknown HTTP method".to_string()
            } else {
                methods.join(", ")
            }
        ),
        source_ids: vec![file.source_id.clone()],
        context_hints: vec!["code".to_string(), "api".to_string()],
        relation: None,
        metadata: json!({
            "component_type": "api_endpoint",
            "route_path": route_path(&file.relative_path),
            "http_methods": methods,
            "auth_detected": has_auth_check(&file.contents),
            "db_access_detected": has_db_access(file),
            "env_var_names": env_var_names(&file.contents),
            "confidence": "medium"
        }),
    })];
    if has_db_store && has_db_access(file) {
        records.push(record_owned(OwnedRecordSpec {
            id: format!(
                "record:{}-accesses-application-database",
                route_id.trim_start_matches("record:")
            ),
            record_type: "access_relation".to_string(),
            title: format!(
                "{} accesses application database",
                route_path(&file.relative_path)
            ),
            summary: "Route file contains database access signals.".to_string(),
            source_ids: vec![file.source_id.clone()],
            context_hints: vec!["code".to_string(), "api".to_string(), "data".to_string()],
            relation: Some(json!({
                "relation_type": "accesses",
                "from_record_id": route_id,
                "to_record_id": "record:application-database"
            })),
            metadata: json!({
                "access_type": "database_access",
                "detectors": db_detectors(&file.contents),
                "confidence": "medium"
            }),
        }));
    }
    records
}

pub(super) fn test_record(file: &CodeFile) -> Value {
    record_owned(OwnedRecordSpec {
        id: format!("record:test-{}", path_slug(&file.relative_path)),
        record_type: "test_or_verification".to_string(),
        title: format!("Test {}", file.relative_path),
        summary: "Detected test/spec file in the repository.".to_string(),
        source_ids: vec![file.source_id.clone()],
        context_hints: vec!["code".to_string(), "testing".to_string()],
        relation: None,
        metadata: json!({
            "test_type": "code_test",
            "confidence": "high"
        }),
    })
}

pub(super) fn env_records(file: &CodeFile) -> Vec<Value> {
    env_var_names(&file.contents)
        .into_iter()
        .map(|name| {
            record_owned(OwnedRecordSpec {
                id: format!(
                    "record:env-{}-{}",
                    slug(&name),
                    path_slug(&file.relative_path)
                ),
                record_type: "requirement".to_string(),
                title: format!("Environment variable {name} is used"),
                summary: format!("{name} is referenced from {}.", file.relative_path),
                source_ids: vec![file.source_id.clone()],
                context_hints: vec!["code".to_string(), "configuration".to_string()],
                relation: None,
                metadata: json!({
                    "require_verification": true,
                    "verification_required": true,
                    "criticality": "medium",
                    "requirement_type": "configuration_validation",
                    "env_var_name": name,
                    "confidence": "high"
                }),
            })
        })
        .collect()
}
pub(super) struct RecordSpec<'a> {
    id: &'a str,
    record_type: &'a str,
    title: &'a str,
    summary: &'a str,
    source_ids: &'a [&'a str],
    context_hints: &'a [&'a str],
    relation: Option<Value>,
    metadata: Value,
}

pub(super) struct OwnedRecordSpec {
    id: String,
    record_type: String,
    title: String,
    summary: String,
    source_ids: Vec<String>,
    context_hints: Vec<String>,
    relation: Option<Value>,
    metadata: Value,
}

pub(super) fn record(spec: RecordSpec<'_>) -> Value {
    json!({
        "id": spec.id,
        "record_type": spec.record_type,
        "title": spec.title,
        "summary": spec.summary,
        "source_ids": spec.source_ids,
        "context_hints": spec.context_hints,
        "relation": spec.relation,
        "provenance": provenance(),
        "metadata": spec.metadata
    })
}

pub(super) fn record_owned(spec: OwnedRecordSpec) -> Value {
    json!({
        "id": spec.id,
        "record_type": spec.record_type,
        "title": spec.title,
        "summary": spec.summary,
        "source_ids": spec.source_ids,
        "context_hints": spec.context_hints,
        "relation": spec.relation,
        "provenance": provenance(),
        "metadata": spec.metadata
    })
}

pub(super) fn provenance() -> Value {
    json!({
        "origin": "source_backed",
        "actor": "source-adapter:code-repo-snapshot",
        "confidence": 1.0,
        "review_status": "accepted"
    })
}

pub(super) fn slug(value: &str) -> String {
    let mut slug = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}

pub(super) fn path_slug(value: &str) -> String {
    format!("{}-{:08x}", slug(value), stable_hash(value))
}

pub(super) fn stable_hash(value: &str) -> u32 {
    let mut hash = 0x811c9dc5_u32;
    for byte in value.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}
