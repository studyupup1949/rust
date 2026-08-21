use super::*;

pub(super) fn collect_code_files(
    repo: &Path,
    coverage: &mut Coverage,
) -> AdvisoryResult<Vec<CodeFile>> {
    let mut paths = Vec::new();
    collect_paths(repo, repo, &mut paths, coverage)?;
    paths.sort();
    paths.dedup();
    let mut files = Vec::new();
    for relative_path in paths {
        let full_path = repo.join(&relative_path);
        let contents = fs::read_to_string(&full_path)?;
        let kind = classify_file(&relative_path);
        coverage.parsed_files += 1;
        if kind == CodeFileKind::ApiRoute {
            coverage.api_route_files += 1;
        }
        if kind == CodeFileKind::Test {
            coverage.test_files += 1;
        }
        files.push(CodeFile {
            source_id: format!("source:code-{}", path_slug(&relative_path)),
            relative_path,
            contents,
            kind,
        });
    }
    Ok(files)
}

pub(super) fn collect_paths(
    repo: &Path,
    dir: &Path,
    paths: &mut Vec<String>,
    coverage: &mut Coverage,
) -> AdvisoryResult<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(repo)
            .unwrap_or(path.as_path())
            .to_string_lossy()
            .replace('\\', "/");
        if should_skip(&relative) {
            coverage.skipped_files += 1;
            continue;
        }
        if path.is_dir() {
            collect_paths(repo, &path, paths, coverage)?;
            continue;
        }
        if is_supported_file(&relative) {
            paths.push(relative);
        } else if let Some(extension) = Path::new(&relative)
            .extension()
            .and_then(|value| value.to_str())
        {
            *coverage
                .unsupported_extensions
                .entry(extension.to_string())
                .or_default() += 1;
        }
    }
    Ok(())
}

pub(super) fn should_skip(relative_path: &str) -> bool {
    relative_path.split('/').any(|part| {
        matches!(
            part,
            ".git"
                | ".next"
                | ".turbo"
                | "build"
                | "coverage"
                | "dist"
                | "node_modules"
                | "target"
                | "vendor"
        )
    })
}

pub(super) fn is_supported_file(relative_path: &str) -> bool {
    matches!(relative_path, "package.json" | "tsconfig.json")
        || matches!(
            Path::new(relative_path)
                .extension()
                .and_then(|value| value.to_str()),
            Some("js" | "jsx" | "ts" | "tsx")
        )
}

pub(super) fn classify_file(relative_path: &str) -> CodeFileKind {
    if matches!(relative_path, "package.json" | "tsconfig.json") {
        CodeFileKind::Manifest
    } else if is_api_route(relative_path) {
        CodeFileKind::ApiRoute
    } else if is_test_file(relative_path) {
        CodeFileKind::Test
    } else {
        CodeFileKind::Source
    }
}

pub(super) fn is_api_route(relative_path: &str) -> bool {
    (relative_path.starts_with("app/api/") || relative_path.starts_with("src/app/api/"))
        && (relative_path.ends_with("/route.ts")
            || relative_path.ends_with("/route.tsx")
            || relative_path.ends_with("/route.js")
            || relative_path.ends_with("/route.jsx"))
}

pub(super) fn is_test_file(relative_path: &str) -> bool {
    relative_path.contains(".test.")
        || relative_path.contains(".spec.")
        || relative_path.starts_with("__tests__/")
        || relative_path.contains("/__tests__/")
}
