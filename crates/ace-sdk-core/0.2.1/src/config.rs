//! Configuration loading with XDG Base Directory support.
//!
//! Loads ACE configuration from multiple sources with priority order:
//! 1. Override options (passed by caller)
//! 2. Environment variables
//! 3. Config file (autodiscovered or specified)
//! 4. Default values

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::errors::AceError;
use crate::logger::ILogger;
use crate::types::AceConfig;

/// Options to override configuration values.
#[derive(Debug, Clone, Default)]
pub struct ConfigOverrides {
    pub config_path: Option<String>,
    pub server_url: Option<String>,
    pub api_token: Option<String>,
    pub project_id: Option<String>,
    pub org_id: Option<String>,
    pub cache_ttl_minutes: Option<u32>,
}

/// Get XDG config path for ACE.
///
/// Returns `$XDG_CONFIG_HOME/ace/config.json` or `~/.config/ace/config.json`.
pub fn get_xdg_config_path() -> PathBuf {
    let xdg_home = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        });
    xdg_home.join("ace").join("config.json")
}

/// Get legacy config path.
pub fn get_legacy_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ace")
        .join("config.json")
}

/// Autodiscover config file path.
///
/// Priority:
/// 1. XDG path: `~/.config/ace/config.json`
/// 2. Legacy path: `~/.ace/config.json`
/// 3. Default: XDG path (even if doesn't exist)
pub fn autodiscover_config_path() -> PathBuf {
    let xdg_path = get_xdg_config_path();
    if xdg_path.exists() {
        return xdg_path;
    }

    let legacy_path = get_legacy_config_path();
    if legacy_path.exists() {
        return legacy_path;
    }

    xdg_path
}

/// Load configuration with priority resolution.
///
/// Priority:
/// 1. Override options (from caller)
/// 2. Environment variables: `ACE_SERVER_URL`, `ACE_API_TOKEN`, `ACE_PROJECT_ID`
/// 3. Config file (autodiscovered via XDG or specified)
/// 4. Default values
pub fn load_config(
    overrides: ConfigOverrides,
    logger: Option<&dyn ILogger>,
) -> Result<AceConfig, AceError> {
    // Determine config file path
    let config_path = if let Some(ref path) = overrides.config_path {
        PathBuf::from(shellexpand(path))
    } else if let Ok(path) = env::var("ACE_CONFIG_PATH") {
        PathBuf::from(shellexpand(&path))
    } else {
        autodiscover_config_path()
    };

    // Start with defaults
    let mut config = AceConfig::default();

    // Load config file if it exists
    if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => match serde_json::from_str::<serde_json::Value>(&contents) {
                Ok(file_config) => {
                    if let Some(url) = file_config.get("serverUrl").and_then(|v| v.as_str()) {
                        config.server_url = url.to_string();
                    }
                    if let Some(token) = file_config.get("apiToken").and_then(|v| v.as_str()) {
                        config.api_token = token.to_string();
                    }
                    if let Some(pid) = file_config.get("projectId").and_then(|v| v.as_str()) {
                        config.project_id = pid.to_string();
                    }
                    if let Some(ttl) = file_config.get("cacheTtlMinutes").and_then(|v| v.as_u64())
                    {
                        config.cache_ttl_minutes = ttl as u32;
                    }
                    if let Some(v) = file_config.get("verbosity").and_then(|v| v.as_str()) {
                        config.verbosity = serde_json::from_value(
                            serde_json::Value::String(v.to_lowercase()),
                        )
                        .ok();
                    }
                    // Load user auth if present
                    if let Some(auth) = file_config.get("auth") {
                        config.auth = serde_json::from_value(auth.clone()).ok();
                    }
                    if let Some(org_id) =
                        file_config.get("default_org_id").and_then(|v| v.as_str())
                    {
                        config.default_org_id = Some(org_id.to_string());
                    }
                    if let Some(device_id) =
                        file_config.get("device_id").and_then(|v| v.as_str())
                    {
                        config.device_id = Some(device_id.to_string());
                    }

                    if let Some(log) = logger {
                        log.debug(&format!("Loaded config from: {}", config_path.display()));
                    }
                }
                Err(e) => {
                    if let Some(log) = logger {
                        log.warn(&format!("Failed to parse config: {}", e));
                    }
                }
            },
            Err(e) => {
                if let Some(log) = logger {
                    log.warn(&format!("Failed to read config file: {}", e));
                }
            }
        }
    }

    // Override with environment variables
    if let Ok(url) = env::var("ACE_SERVER_URL") {
        config.server_url = url;
    }
    if let Ok(token) = env::var("ACE_API_TOKEN") {
        config.api_token = token;
    }
    if let Ok(pid) = env::var("ACE_PROJECT_ID") {
        config.project_id = pid;
    }
    if let Ok(ttl) = env::var("ACE_CACHE_TTL_MINUTES") {
        if let Ok(v) = ttl.parse::<u32>() {
            config.cache_ttl_minutes = v;
        }
    }

    // Override with explicit overrides (highest priority)
    if let Some(url) = overrides.server_url {
        config.server_url = url;
    }
    if let Some(token) = overrides.api_token {
        config.api_token = token;
    }
    if let Some(pid) = overrides.project_id {
        config.project_id = pid;
    }
    if let Some(ttl) = overrides.cache_ttl_minutes {
        config.cache_ttl_minutes = ttl;
    }

    // Populate api_token from user auth if not set
    if config.api_token.is_empty() {
        if let Some(ref auth) = config.auth {
            config.api_token = auth.token.clone();
        }
    }

    if let Some(log) = logger {
        log.debug(&format!("Server URL: {}", config.server_url));
        log.debug(&format!(
            "API Token: {}...",
            if config.api_token.len() > 15 {
                &config.api_token[..15]
            } else {
                &config.api_token
            }
        ));
        log.debug(&format!("Project ID: {}", config.project_id));
    }

    Ok(config)
}

/// Get config file path (XDG-compliant).
pub fn get_config_path() -> PathBuf {
    get_xdg_config_path()
}

/// Check if configuration is complete (has token and project ID).
pub fn is_configured(overrides: ConfigOverrides, logger: Option<&dyn ILogger>) -> bool {
    match load_config(overrides, logger) {
        Ok(config) => !config.api_token.is_empty() && !config.project_id.is_empty(),
        Err(_) => false,
    }
}

// =============================================================================
// Config Helper Functions
// =============================================================================

/// Gets the API token for a specific organization.
///
/// Priority:
/// 1. User auth token (works for all orgs user belongs to)
/// 2. Multi-org mode: Looks up token by org ID in config.orgs
/// 3. Single-org mode: Returns root-level apiToken
pub fn get_token_for_org(config: &crate::types::AceConfig, org_id: &str) -> Result<String, AceError> {
    // User auth token (works for all orgs)
    if let Some(ref auth) = config.auth {
        if !auth.token.is_empty() {
            return Ok(auth.token.clone());
        }
    }

    // Multi-org mode
    if let Some(ref orgs) = config.orgs {
        if let Some(org_config) = orgs.get(org_id) {
            return Ok(org_config.api_token.clone());
        }
    }

    // Single-org mode fallback
    if !config.api_token.is_empty() {
        return Ok(config.api_token.clone());
    }

    Err(AceError::Config(format!(
        "No API token found for organization {}",
        org_id
    )))
}

/// Gets the organization name for display purposes.
pub fn get_org_name(config: &crate::types::AceConfig, org_id: &str) -> String {
    if let Some(ref orgs) = config.orgs {
        if let Some(org_config) = orgs.get(org_id) {
            if !org_config.org_name.is_empty() {
                return org_config.org_name.clone();
            }
        }
    }
    org_id.to_string()
}

/// Lists all configured organizations.
pub fn list_organizations(
    config: &crate::types::AceConfig,
) -> Vec<(String, String, usize)> {
    let mut orgs = Vec::new();

    if let Some(ref org_map) = config.orgs {
        for (org_id, org_config) in org_map {
            let name = if org_config.org_name.is_empty() {
                org_id.clone()
            } else {
                org_config.org_name.clone()
            };
            orgs.push((org_id.clone(), name, org_config.projects.len()));
        }
    } else if !config.api_token.is_empty() {
        let org_id = extract_org_id_from_token(&config.api_token)
            .unwrap_or_else(|| "default".to_string());
        orgs.push((org_id, "Default Organization".to_string(), 0));
    }

    orgs
}

/// Checks if a project belongs to a specific organization.
pub fn project_belongs_to_org(
    config: &crate::types::AceConfig,
    org_id: &str,
    project_id: &str,
) -> bool {
    if let Some(ref orgs) = config.orgs {
        if let Some(org_config) = orgs.get(org_id) {
            return org_config.projects.contains(&project_id.to_string());
        }
    }
    // In single-org mode, assume any project is valid
    true
}

/// Extracts organization ID from an API token.
///
/// ACE tokens have format: ace_{orgId8chars}{random}
pub fn extract_org_id_from_token(token: &str) -> Option<String> {
    if !token.starts_with("ace_") {
        return None;
    }
    let after_prefix = &token[4..];
    if after_prefix.len() >= 8 {
        Some(format!("org_{}", &after_prefix[..8]))
    } else {
        None
    }
}

/// Gets all projects for a specific organization.
pub fn get_projects_for_org(config: &crate::types::AceConfig, org_id: &str) -> Vec<String> {
    if let Some(ref orgs) = config.orgs {
        if let Some(org_config) = orgs.get(org_id) {
            return org_config.projects.clone();
        }
    }
    Vec::new()
}

/// Checks if the configuration is in multi-org mode.
pub fn is_multi_org_mode(config: &crate::types::AceConfig) -> bool {
    config
        .orgs
        .as_ref()
        .map(|o| !o.is_empty())
        .unwrap_or(false)
}

/// Validates configuration completeness.
///
/// Returns a vector of error messages (empty if valid).
pub fn validate_config(config: &crate::types::AceConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if config.server_url.is_empty() {
        errors.push("Missing serverUrl in configuration".to_string());
    }

    let has_multi_org = config
        .orgs
        .as_ref()
        .map(|o| !o.is_empty())
        .unwrap_or(false);
    let has_single_org = !config.api_token.is_empty();

    if !has_multi_org && !has_single_org && config.auth.is_none() {
        errors.push(
            "No API token configured (neither multi-org nor single-org mode)".to_string(),
        );
    }

    if let Some(ref orgs) = config.orgs {
        for (org_id, org_config) in orgs {
            if org_config.api_token.is_empty() {
                errors.push(format!("Organization {} is missing apiToken", org_id));
            }
            if !org_id.starts_with("org_") {
                errors.push(format!(
                    "Invalid organization ID format: {} (should start with 'org_')",
                    org_id
                ));
            }
        }
    }

    errors
}

/// Convert an AceConfig to an AceContext.
pub fn config_to_context(config: &crate::types::AceConfig) -> crate::types::AceContext {
    let org_id = config
        .default_org_id
        .clone()
        .or_else(|| {
            config
                .auth
                .as_ref()
                .and_then(|a| a.organizations.first())
                .map(|o| o.org_id.clone())
        });

    crate::types::AceContext {
        server_url: config.server_url.clone(),
        api_token: config.api_token.clone(),
        project_id: config.project_id.clone(),
        org_id,
        cache_ttl_minutes: config.cache_ttl_minutes,
        runtime_settings: crate::types::default_runtime_settings(),
    }
}

/// Validates that an org ID matches the expected format.
pub fn is_valid_org_id(org_id: &str) -> bool {
    let re = regex::Regex::new(r"^org_[a-zA-Z0-9_-]+$").unwrap();
    re.is_match(org_id)
}

/// Validates that a project ID matches the expected format.
pub fn is_valid_project_id(project_id: &str) -> bool {
    let re = regex::Regex::new(r"^prj_[a-f0-9]+$").unwrap();
    re.is_match(project_id)
}

/// Resolve org and project context using 3-tier precedence.
pub fn resolve_context(
    options: &crate::types::ResolveContextOptions,
) -> Result<crate::types::ResolvedContext, AceError> {
    // Tier 1: CLI flags (highest priority)
    if let (Some(ref org), Some(ref project)) = (&options.org, &options.project) {
        return Ok(crate::types::ResolvedContext {
            org_id: org.clone(),
            project_id: project.clone(),
            source: crate::types::ContextSource::Flags,
        });
    }

    let mut org_id = options.org.clone();
    let mut project_id = options.project.clone();
    let mut source = crate::types::ContextSource::Flags;

    // Tier 2: Environment variables
    if org_id.is_none() {
        if let Ok(env_org) = std::env::var("ACE_ORG_ID") {
            if !env_org.is_empty() {
                org_id = Some(env_org);
                source = crate::types::ContextSource::Env;
            }
        }
    }
    if project_id.is_none() {
        if let Ok(env_project) = std::env::var("ACE_PROJECT_ID") {
            if !env_project.is_empty() {
                project_id = Some(env_project);
                if org_id.is_none() || source != crate::types::ContextSource::Flags {
                    source = crate::types::ContextSource::Env;
                }
            }
        }
    }

    if let (Some(org), Some(project)) = (org_id, project_id) {
        return Ok(crate::types::ResolvedContext {
            org_id: org,
            project_id: project,
            source,
        });
    }

    Err(AceError::Config(
        "Could not resolve organization ID and/or project ID".to_string(),
    ))
}

// =============================================================================
// Path Expansion
// =============================================================================

/// Expand shell variables in a path string.
///
/// Supports:
/// - Tilde expansion: `~/path` -> `/home/user/path`
/// - Simple variables: `$HOME` -> `/home/user`
/// - Braced variables: `${HOME}` -> `/home/user`
/// - Default values: `${VAR:-default}` -> `default` (if VAR not set)
pub fn expand_path(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    let mut result = path.to_string();

    // Step 1: Expand tilde to home directory
    if result.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            result = format!("{}{}", home.display(), &result[1..]);
        }
    } else if result == "~" {
        if let Some(home) = dirs::home_dir() {
            result = home.display().to_string();
        }
    }

    // Step 2: Expand environment variables (iteratively to handle nesting)
    let max_iterations = 10;
    for _ in 0..max_iterations {
        let previous = result.clone();

        // Match ${VAR:-default}
        let re_default = regex::Regex::new(r"\$\{([^}:]+):-([^}]*)\}").unwrap();
        result = re_default
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                let default_val = &caps[2];
                env::var(var_name).unwrap_or_else(|_| default_val.to_string())
            })
            .to_string();

        // Match ${VAR}
        let re_braced = regex::Regex::new(r"\$\{([^}:]+)\}").unwrap();
        result = re_braced
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
            })
            .to_string();

        // Match $VAR (without braces)
        let re_simple = regex::Regex::new(r"\$([A-Z_][A-Z0-9_]*)").unwrap();
        result = re_simple
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = &caps[1];
                env::var(var_name).unwrap_or_else(|_| caps[0].to_string())
            })
            .to_string();

        if result == previous {
            break;
        }
    }

    result
}

/// Simple shell expansion (alias for backward compat, used internally).
fn shellexpand(path: &str) -> String {
    expand_path(path)
}

// =============================================================================
// Config Auth Management
// =============================================================================

/// Save user authentication credentials to config file.
///
/// Loads existing config (if any), updates the `auth` block, writes back.
pub fn save_auth_credentials(
    config_path: &std::path::Path,
    auth: &crate::types::UserAuth,
) -> Result<(), AceError> {
    ensure_config_directory(config_path)?;

    let mut config = load_config_json(config_path);

    // Update auth block
    let mut auth_obj = serde_json::Map::new();
    auth_obj.insert("token".into(), serde_json::json!(auth.token));
    auth_obj.insert("user_id".into(), serde_json::json!(auth.user_id));
    auth_obj.insert("email".into(), serde_json::json!(auth.email));
    auth_obj.insert("organizations".into(), serde_json::json!(auth.organizations));
    auth_obj.insert(
        "authenticated_at".into(),
        serde_json::json!(auth.authenticated_at.as_deref().unwrap_or_else(|| {
            // We can't call a closure here that returns &str easily,
            // so we just use a default
            "unknown"
        })),
    );
    if let Some(ref rt) = auth.refresh_token {
        auth_obj.insert("refresh_token".into(), serde_json::json!(rt));
    }
    if let Some(ref ea) = auth.expires_at {
        auth_obj.insert("expires_at".into(), serde_json::json!(ea));
    }
    if let Some(ref rea) = auth.refresh_expires_at {
        auth_obj.insert("refresh_expires_at".into(), serde_json::json!(rea));
    }
    if let Some(ref aea) = auth.absolute_expires_at {
        auth_obj.insert("absolute_expires_at".into(), serde_json::json!(aea));
    }

    config["auth"] = serde_json::Value::Object(auth_obj);

    write_config_json(config_path, &config)?;
    set_secure_permissions(config_path);
    Ok(())
}

/// Clear user authentication credentials from config file.
///
/// Removes `auth` and `default_org_id` from config.
pub fn clear_auth(config_path: &std::path::Path) -> Result<(), AceError> {
    if !config_path.exists() {
        return Ok(());
    }

    let mut config = load_config_json(config_path);

    if let Some(obj) = config.as_object_mut() {
        obj.remove("auth");
        obj.remove("default_org_id");
    }

    write_config_json(config_path, &config)
}

/// Set default organization for API requests.
///
/// Validates that the org exists in the user's organizations before setting.
pub fn set_default_org(
    config_path: &std::path::Path,
    org_id: &str,
) -> Result<(), AceError> {
    ensure_config_directory(config_path)?;

    let mut config = load_config_json(config_path);

    // Validate org exists in user's organizations
    if let Some(auth) = config.get("auth") {
        if let Some(orgs) = auth.get("organizations").and_then(|v| v.as_array()) {
            let org_exists = orgs.iter().any(|org| {
                org.get("org_id").and_then(|v| v.as_str()) == Some(org_id)
            });
            if !org_exists {
                return Err(AceError::Config(format!(
                    "Organization '{}' not found in your organizations",
                    org_id
                )));
            }
        }
    }

    config["default_org_id"] = serde_json::json!(org_id);

    write_config_json(config_path, &config)
}

/// Get default organization ID from config.
pub fn get_default_org_id(config_path: &std::path::Path) -> Option<String> {
    if !config_path.exists() {
        return None;
    }
    let config = load_config_json(config_path);
    config
        .get("default_org_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Update organizations list in config (after refresh).
///
/// Also validates that `default_org_id` still exists in the updated list.
pub fn update_organizations(
    config_path: &std::path::Path,
    orgs: &[crate::types::OrgMembership],
) -> Result<(), AceError> {
    if !config_path.exists() {
        return Err(AceError::Config(
            "No config file found, cannot update organizations".to_string(),
        ));
    }

    let mut config = load_config_json(config_path);

    if config.get("auth").is_none() {
        return Err(AceError::Config(
            "No auth block in config, cannot update organizations".to_string(),
        ));
    }

    config["auth"]["organizations"] = serde_json::json!(orgs);

    // Validate default_org_id still exists
    if let Some(default_org) = config.get("default_org_id").and_then(|v| v.as_str()) {
        let org_exists = orgs.iter().any(|org| org.org_id == default_org);
        if !org_exists {
            if let Some(obj) = config.as_object_mut() {
                obj.remove("default_org_id");
            }
        }
    }

    write_config_json(config_path, &config)
}

/// Load user auth from config file.
pub fn load_user_auth(config_path: &std::path::Path) -> Option<crate::types::UserAuth> {
    if !config_path.exists() {
        return None;
    }
    let config = load_config_json(config_path);
    config
        .get("auth")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
}

/// Get or create device ID (persists across logins).
///
/// Device ID is stored at config root level, NOT in the auth block.
/// Format: `dev_{22 random chars}` (base64url safe).
pub fn get_or_create_device_id(config_path: &std::path::Path) -> Result<String, AceError> {
    ensure_config_directory(config_path)?;

    let mut config = load_config_json(config_path);

    // Return existing device_id if present
    if let Some(device_id) = config.get("device_id").and_then(|v| v.as_str()) {
        return Ok(device_id.to_string());
    }

    // Generate new device_id
    let device_id = generate_device_id();
    config["device_id"] = serde_json::json!(&device_id);

    write_config_json(config_path, &config)?;
    set_secure_permissions(config_path);
    Ok(device_id)
}

/// Get device ID from config (returns None if not set).
pub fn get_device_id(config_path: &std::path::Path) -> Option<String> {
    if !config_path.exists() {
        return None;
    }
    let config = load_config_json(config_path);
    config
        .get("device_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Update auth tokens in config (for token refresh).
pub fn update_auth_tokens(
    config_path: &std::path::Path,
    access_token: &str,
    refresh_token: &str,
    expires_at: &str,
    refresh_expires_at: &str,
    absolute_expires_at: Option<&str>,
) -> Result<(), AceError> {
    if !config_path.exists() {
        return Err(AceError::Config(
            "No config file found. Please login first.".to_string(),
        ));
    }

    let mut config = load_config_json(config_path);

    if config.get("auth").is_none() {
        return Err(AceError::Config(
            "No auth block in config. Please login first.".to_string(),
        ));
    }

    config["auth"]["token"] = serde_json::json!(access_token);
    config["auth"]["refresh_token"] = serde_json::json!(refresh_token);
    config["auth"]["expires_at"] = serde_json::json!(expires_at);
    config["auth"]["refresh_expires_at"] = serde_json::json!(refresh_expires_at);

    if let Some(aea) = absolute_expires_at {
        config["auth"]["absolute_expires_at"] = serde_json::json!(aea);
    }

    write_config_json(config_path, &config)?;
    set_secure_permissions(config_path);
    Ok(())
}

// =============================================================================
// Config Manipulation Helpers
// =============================================================================

/// Ensure config directory exists with secure permissions (0o700).
pub fn ensure_config_directory(config_path: &std::path::Path) -> Result<(), AceError> {
    if let Some(dir) = config_path.parent() {
        if !dir.exists() {
            fs::create_dir_all(dir).map_err(|e| {
                AceError::Config(format!("Failed to create config directory: {}", e))
            })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).ok();
            }
        }
    }
    Ok(())
}

/// Set secure permissions (0o600) on a config file.
pub fn set_secure_permissions(config_path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if config_path.exists() {
            fs::set_permissions(config_path, fs::Permissions::from_mode(0o600)).ok();
        }
    }
}

/// Auto-migrate config from legacy path to XDG path.
///
/// Migration strategy:
/// - Only migrate if legacy exists and XDG doesn't
/// - Create XDG directory with secure permissions (0o700)
/// - Copy config to XDG path with secure permissions (0o600)
/// - Rename legacy config to .bak (don't delete)
///
/// Returns the config path to use.
pub fn auto_migrate_config(logger: Option<&dyn ILogger>) -> PathBuf {
    let legacy_path = get_legacy_config_path();
    let xdg_path = get_xdg_config_path();

    // Only migrate if legacy exists and XDG doesn't
    if legacy_path.exists() && !xdg_path.exists() {
        if let Some(log) = logger {
            log.info("Migrating ACE config to XDG standard path...");
        }

        match migrate_config_impl(&legacy_path, &xdg_path) {
            Ok(()) => {
                // Rename legacy to .bak
                let backup_path = legacy_path.with_extension("json.bak");
                fs::rename(&legacy_path, &backup_path).ok();

                if let Some(log) = logger {
                    log.info(&format!(
                        "Config migrated from {} to {}",
                        legacy_path.display(),
                        xdg_path.display()
                    ));
                }
                return xdg_path;
            }
            Err(e) => {
                if let Some(log) = logger {
                    log.warn(&format!("Migration failed (non-critical): {}", e));
                }
                return legacy_path;
            }
        }
    }

    if xdg_path.exists() {
        return xdg_path;
    }
    if legacy_path.exists() {
        return legacy_path;
    }
    xdg_path
}

/// Migrate config from one path to another.
pub fn migrate_config(from: &std::path::Path, to: &std::path::Path) -> Result<(), AceError> {
    migrate_config_impl(from, to)
}

/// Internal migration implementation.
fn migrate_config_impl(
    from: &std::path::Path,
    to: &std::path::Path,
) -> Result<(), AceError> {
    ensure_config_directory(to)?;
    fs::copy(from, to).map_err(|e| {
        AceError::Config(format!(
            "Failed to copy config from {} to {}: {}",
            from.display(),
            to.display(),
            e
        ))
    })?;
    set_secure_permissions(to);
    Ok(())
}

/// Check if migration is needed and perform it.
pub fn check_and_migrate(logger: Option<&dyn ILogger>) {
    let legacy_path = get_legacy_config_path();
    let xdg_path = get_xdg_config_path();

    if legacy_path.exists() && !xdg_path.exists() {
        if let Some(log) = logger {
            log.info("Old config format detected. Migration recommended.");
        }
        auto_migrate_config(logger);
    }
}

/// Generate a unique device ID.
///
/// Format: `dev_{22 random chars}` (base64url safe).
fn generate_device_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Use timestamp + process ID for uniqueness
    let pid = std::process::id();
    let raw = format!("{:x}{:x}", timestamp, pid);
    // Take first 22 chars, base64url-safe
    let encoded: String = raw
        .chars()
        .take(22)
        .collect();
    format!("dev_{}", encoded)
}

/// Load config JSON from file path, returning empty object if missing/invalid.
fn load_config_json(config_path: &std::path::Path) -> serde_json::Value {
    if config_path.exists() {
        if let Ok(contents) = fs::read_to_string(config_path) {
            if let Ok(val) = serde_json::from_str(&contents) {
                return val;
            }
        }
    }
    serde_json::json!({})
}

/// Write config JSON to file with pretty printing.
fn write_config_json(
    config_path: &std::path::Path,
    config: &serde_json::Value,
) -> Result<(), AceError> {
    let contents = serde_json::to_string_pretty(config).map_err(|e| {
        AceError::Config(format!("Failed to serialize config: {}", e))
    })?;
    fs::write(config_path, contents).map_err(|e| {
        AceError::Config(format!("Failed to write config file: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shellexpand_tilde() {
        let expanded = shellexpand("~/test/path");
        assert!(!expanded.starts_with("~/"));
        assert!(expanded.ends_with("/test/path"));
    }

    #[test]
    fn test_shellexpand_no_tilde() {
        assert_eq!(shellexpand("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_load_config_defaults() {
        // Clear env vars that might interfere
        env::remove_var("ACE_SERVER_URL");
        env::remove_var("ACE_API_TOKEN");
        env::remove_var("ACE_PROJECT_ID");
        env::remove_var("ACE_CONFIG_PATH");

        let config = load_config(
            ConfigOverrides {
                config_path: Some("/nonexistent/path.json".to_string()),
                ..Default::default()
            },
            None,
        )
        .unwrap();

        assert_eq!(config.server_url, "https://ace-api.code-engine.app");
        assert_eq!(config.cache_ttl_minutes, 120);
    }

    #[test]
    fn test_load_config_overrides() {
        let config = load_config(
            ConfigOverrides {
                config_path: Some("/nonexistent/path.json".to_string()),
                server_url: Some("https://custom.example.com".to_string()),
                api_token: Some("ace_test_token".to_string()),
                project_id: Some("test-project".to_string()),
                ..Default::default()
            },
            None,
        )
        .unwrap();

        assert_eq!(config.server_url, "https://custom.example.com");
        assert_eq!(config.api_token, "ace_test_token");
        assert_eq!(config.project_id, "test-project");
    }

    #[test]
    fn test_extract_org_id_from_token_valid() {
        let org_id = extract_org_id_from_token("ace_34fYIlitYk4nyFuTvtsAzA6uUJF");
        assert_eq!(org_id, Some("org_34fYIlit".to_string()));
    }

    #[test]
    fn test_extract_org_id_from_token_invalid() {
        assert_eq!(extract_org_id_from_token("invalid_token"), None);
        assert_eq!(extract_org_id_from_token("ace_ab"), None);
    }

    #[test]
    fn test_get_token_for_org_with_auth() {
        let config = crate::types::AceConfig {
            auth: Some(crate::types::UserAuth {
                token: "ace_user_test".to_string(),
                user_id: "u1".to_string(),
                email: "test@example.com".to_string(),
                organizations: vec![],
                authenticated_at: None,
                refresh_token: None,
                expires_at: None,
                refresh_expires_at: None,
                absolute_expires_at: None,
            }),
            ..Default::default()
        };
        let token = get_token_for_org(&config, "org_test123");
        assert_eq!(token.unwrap(), "ace_user_test");
    }

    #[test]
    fn test_get_token_for_org_fallback() {
        let config = crate::types::AceConfig {
            api_token: "ace_12345678test".to_string(),
            ..Default::default()
        };
        let token = get_token_for_org(&config, "org_12345678");
        assert_eq!(token.unwrap(), "ace_12345678test");
    }

    #[test]
    fn test_get_token_for_org_no_token() {
        let config = crate::types::AceConfig::default();
        let result = get_token_for_org(&config, "org_test");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_org_name_with_orgs() {
        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "org_test".to_string(),
            crate::types::OrgConfig {
                org_name: "Test Org".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec![],
            },
        );
        let config = crate::types::AceConfig {
            orgs: Some(orgs),
            ..Default::default()
        };
        assert_eq!(get_org_name(&config, "org_test"), "Test Org");
    }

    #[test]
    fn test_get_org_name_fallback() {
        let config = crate::types::AceConfig::default();
        assert_eq!(get_org_name(&config, "org_test"), "org_test");
    }

    #[test]
    fn test_list_organizations_multi_org() {
        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "org_abc".to_string(),
            crate::types::OrgConfig {
                org_name: "ABC Corp".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec!["prj_1".to_string(), "prj_2".to_string()],
            },
        );
        let config = crate::types::AceConfig {
            orgs: Some(orgs),
            ..Default::default()
        };
        let result = list_organizations(&config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "org_abc");
        assert_eq!(result[0].1, "ABC Corp");
        assert_eq!(result[0].2, 2);
    }

    #[test]
    fn test_list_organizations_single_org() {
        let config = crate::types::AceConfig {
            api_token: "ace_12345678test".to_string(),
            ..Default::default()
        };
        let result = list_organizations(&config);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "org_12345678");
    }

    #[test]
    fn test_project_belongs_to_org_true() {
        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "org_test".to_string(),
            crate::types::OrgConfig {
                org_name: "Test".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec!["prj_abc".to_string()],
            },
        );
        let config = crate::types::AceConfig {
            orgs: Some(orgs),
            ..Default::default()
        };
        assert!(project_belongs_to_org(&config, "org_test", "prj_abc"));
        assert!(!project_belongs_to_org(&config, "org_test", "prj_xyz"));
    }

    #[test]
    fn test_project_belongs_to_org_single_mode() {
        let config = crate::types::AceConfig::default();
        // In single-org mode, any project is valid
        assert!(project_belongs_to_org(&config, "org_any", "prj_any"));
    }

    #[test]
    fn test_get_projects_for_org() {
        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "org_test".to_string(),
            crate::types::OrgConfig {
                org_name: "Test".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec!["prj_1".to_string(), "prj_2".to_string()],
            },
        );
        let config = crate::types::AceConfig {
            orgs: Some(orgs),
            ..Default::default()
        };
        let projects = get_projects_for_org(&config, "org_test");
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0], "prj_1");
    }

    #[test]
    fn test_get_projects_for_org_not_found() {
        let config = crate::types::AceConfig::default();
        let projects = get_projects_for_org(&config, "org_missing");
        assert!(projects.is_empty());
    }

    #[test]
    fn test_is_multi_org_mode() {
        let config = crate::types::AceConfig::default();
        assert!(!is_multi_org_mode(&config));

        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "org_test".to_string(),
            crate::types::OrgConfig {
                org_name: "Test".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec![],
            },
        );
        let config2 = crate::types::AceConfig {
            orgs: Some(orgs),
            ..Default::default()
        };
        assert!(is_multi_org_mode(&config2));
    }

    #[test]
    fn test_validate_config_valid() {
        let config = crate::types::AceConfig {
            server_url: "https://example.com".to_string(),
            api_token: "ace_12345678test".to_string(),
            ..Default::default()
        };
        let errors = validate_config(&config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_config_missing_server() {
        let config = crate::types::AceConfig {
            server_url: String::new(),
            ..Default::default()
        };
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("serverUrl")));
    }

    #[test]
    fn test_validate_config_missing_token() {
        let config = crate::types::AceConfig {
            server_url: "https://example.com".to_string(),
            api_token: String::new(),
            auth: None,
            orgs: None,
            ..Default::default()
        };
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("No API token")));
    }

    #[test]
    fn test_validate_config_invalid_org_id() {
        let mut orgs = std::collections::HashMap::new();
        orgs.insert(
            "bad_id".to_string(),
            crate::types::OrgConfig {
                org_name: "Test".to_string(),
                api_token: "ace_test".to_string(),
                projects: vec![],
            },
        );
        let config = crate::types::AceConfig {
            server_url: "https://example.com".to_string(),
            orgs: Some(orgs),
            ..Default::default()
        };
        let errors = validate_config(&config);
        assert!(errors.iter().any(|e| e.contains("Invalid organization ID")));
    }

    #[test]
    fn test_config_to_context() {
        let config = crate::types::AceConfig {
            server_url: "https://example.com".to_string(),
            api_token: "ace_test".to_string(),
            project_id: "prj_test".to_string(),
            default_org_id: Some("org_default".to_string()),
            ..Default::default()
        };
        let ctx = config_to_context(&config);
        assert_eq!(ctx.server_url, "https://example.com");
        assert_eq!(ctx.api_token, "ace_test");
        assert_eq!(ctx.project_id, "prj_test");
        assert_eq!(ctx.org_id, Some("org_default".to_string()));
    }

    #[test]
    fn test_config_to_context_from_auth() {
        let config = crate::types::AceConfig {
            server_url: "https://example.com".to_string(),
            auth: Some(crate::types::UserAuth {
                token: "ace_user_test".to_string(),
                user_id: "u1".to_string(),
                email: "test@test.com".to_string(),
                organizations: vec![crate::types::OrgMembership {
                    org_id: "org_from_auth".to_string(),
                    name: "Auth Org".to_string(),
                    role: "admin".to_string(),
                    created_at: None,
                }],
                authenticated_at: None,
                refresh_token: None,
                expires_at: None,
                refresh_expires_at: None,
                absolute_expires_at: None,
            }),
            ..Default::default()
        };
        let ctx = config_to_context(&config);
        assert_eq!(ctx.org_id, Some("org_from_auth".to_string()));
    }

    #[test]
    fn test_is_valid_org_id() {
        assert!(is_valid_org_id("org_abc123"));
        assert!(is_valid_org_id("org_test-123_abc"));
        assert!(!is_valid_org_id("bad_org"));
        assert!(!is_valid_org_id(""));
    }

    #[test]
    fn test_is_valid_project_id() {
        assert!(is_valid_project_id("prj_abc123def"));
        assert!(!is_valid_project_id("prj_UPPER"));
        assert!(!is_valid_project_id("bad_prj"));
    }

    #[test]
    fn test_resolve_context_from_flags() {
        let options = crate::types::ResolveContextOptions {
            org: Some("org_flag".to_string()),
            project: Some("prj_flag".to_string()),
            cwd: None,
        };
        let result = resolve_context(&options).unwrap();
        assert_eq!(result.org_id, "org_flag");
        assert_eq!(result.project_id, "prj_flag");
        assert_eq!(result.source, crate::types::ContextSource::Flags);
    }

    #[test]
    fn test_resolve_context_from_env() {
        std::env::set_var("ACE_ORG_ID", "org_env");
        std::env::set_var("ACE_PROJECT_ID", "prj_env");
        let options = crate::types::ResolveContextOptions::default();
        let result = resolve_context(&options).unwrap();
        assert_eq!(result.org_id, "org_env");
        assert_eq!(result.project_id, "prj_env");
        assert_eq!(result.source, crate::types::ContextSource::Env);
        std::env::remove_var("ACE_ORG_ID");
        std::env::remove_var("ACE_PROJECT_ID");
    }

    #[test]
    fn test_resolve_context_fails_without_context() {
        std::env::remove_var("ACE_ORG_ID");
        std::env::remove_var("ACE_PROJECT_ID");
        let options = crate::types::ResolveContextOptions::default();
        let result = resolve_context(&options);
        assert!(result.is_err());
    }

    // =========================================================================
    // expand_path tests
    // =========================================================================

    #[test]
    fn test_expand_path_empty() {
        assert_eq!(expand_path(""), "");
    }

    #[test]
    fn test_expand_path_tilde() {
        let expanded = expand_path("~/test/path");
        assert!(!expanded.starts_with("~/"));
        assert!(expanded.ends_with("/test/path"));
    }

    #[test]
    fn test_expand_path_tilde_alone() {
        let expanded = expand_path("~");
        assert!(!expanded.contains("~"));
        assert!(!expanded.is_empty());
    }

    #[test]
    fn test_expand_path_env_var() {
        std::env::set_var("ACE_TEST_EXPAND", "/custom/path");
        let expanded = expand_path("$ACE_TEST_EXPAND/sub");
        assert_eq!(expanded, "/custom/path/sub");
        std::env::remove_var("ACE_TEST_EXPAND");
    }

    #[test]
    fn test_expand_path_braced_env_var() {
        std::env::set_var("ACE_TEST_BRACED", "/braced");
        let expanded = expand_path("${ACE_TEST_BRACED}/dir");
        assert_eq!(expanded, "/braced/dir");
        std::env::remove_var("ACE_TEST_BRACED");
    }

    #[test]
    fn test_expand_path_default_value() {
        std::env::remove_var("ACE_TEST_MISSING_VAR");
        let expanded = expand_path("${ACE_TEST_MISSING_VAR:-/fallback}/dir");
        assert_eq!(expanded, "/fallback/dir");
    }

    #[test]
    fn test_expand_path_no_expansion_needed() {
        assert_eq!(expand_path("/absolute/path"), "/absolute/path");
    }

    // =========================================================================
    // Config auth management tests
    // =========================================================================

    fn make_test_auth() -> crate::types::UserAuth {
        crate::types::UserAuth {
            token: "ace_user_test_token".to_string(),
            user_id: "user_123".to_string(),
            email: "test@example.com".to_string(),
            organizations: vec![crate::types::OrgMembership {
                org_id: "org_test1".to_string(),
                name: "Test Org".to_string(),
                role: "admin".to_string(),
                created_at: None,
            }],
            authenticated_at: Some("2024-01-01T00:00:00Z".to_string()),
            refresh_token: Some("rt_refresh_abc".to_string()),
            expires_at: Some("2024-01-02T00:00:00Z".to_string()),
            refresh_expires_at: Some("2024-01-08T00:00:00Z".to_string()),
            absolute_expires_at: Some("2024-01-15T00:00:00Z".to_string()),
        }
    }

    #[test]
    fn test_save_auth_credentials() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let auth = make_test_auth();
        save_auth_credentials(&path, &auth).unwrap();

        assert!(path.exists());
        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["auth"]["token"], "ace_user_test_token");
        assert_eq!(config["auth"]["user_id"], "user_123");
        assert_eq!(config["auth"]["email"], "test@example.com");
        assert_eq!(config["auth"]["refresh_token"], "rt_refresh_abc");
        assert_eq!(config["auth"]["absolute_expires_at"], "2024-01-15T00:00:00Z");
    }

    #[test]
    fn test_save_auth_preserves_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"device_id": "dev_existing"}"#).unwrap();

        let auth = make_test_auth();
        save_auth_credentials(&path, &auth).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["device_id"], "dev_existing");
        assert_eq!(config["auth"]["token"], "ace_user_test_token");
    }

    #[test]
    fn test_clear_auth() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"token": "test"}, "default_org_id": "org_x", "device_id": "dev_1"}"#,
        )
        .unwrap();

        clear_auth(&path).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(config.get("auth").is_none());
        assert!(config.get("default_org_id").is_none());
        assert_eq!(config["device_id"], "dev_1"); // preserved
    }

    #[test]
    fn test_clear_auth_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        // Should not error
        clear_auth(&path).unwrap();
    }

    #[test]
    fn test_set_default_org() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"organizations": [{"org_id": "org_abc", "name": "ABC", "role": "admin"}]}}"#,
        )
        .unwrap();

        set_default_org(&path, "org_abc").unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["default_org_id"], "org_abc");
    }

    #[test]
    fn test_set_default_org_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"organizations": [{"org_id": "org_abc", "name": "ABC", "role": "admin"}]}}"#,
        )
        .unwrap();

        let result = set_default_org(&path, "org_nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_get_default_org_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"default_org_id": "org_default"}"#).unwrap();

        assert_eq!(get_default_org_id(&path), Some("org_default".to_string()));
    }

    #[test]
    fn test_get_default_org_id_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        assert_eq!(get_default_org_id(&path), None);
    }

    #[test]
    fn test_get_default_org_id_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert_eq!(get_default_org_id(&path), None);
    }

    #[test]
    fn test_update_organizations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"token": "t", "organizations": []}, "default_org_id": "org_old"}"#,
        )
        .unwrap();

        let new_orgs = vec![crate::types::OrgMembership {
            org_id: "org_new".to_string(),
            name: "New Org".to_string(),
            role: "member".to_string(),
            created_at: None,
        }];
        update_organizations(&path, &new_orgs).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let orgs = config["auth"]["organizations"].as_array().unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0]["org_id"], "org_new");
        // default_org_id "org_old" should be cleared since it's no longer in orgs
        assert!(config.get("default_org_id").is_none());
    }

    #[test]
    fn test_update_organizations_preserves_valid_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"token": "t", "organizations": []}, "default_org_id": "org_keep"}"#,
        )
        .unwrap();

        let new_orgs = vec![crate::types::OrgMembership {
            org_id: "org_keep".to_string(),
            name: "Keep Org".to_string(),
            role: "admin".to_string(),
            created_at: None,
        }];
        update_organizations(&path, &new_orgs).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["default_org_id"], "org_keep");
    }

    #[test]
    fn test_update_organizations_no_auth() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        let result = update_organizations(&path, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_user_auth() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let auth = make_test_auth();
        save_auth_credentials(&path, &auth).unwrap();

        let loaded = load_user_auth(&path).unwrap();
        assert_eq!(loaded.token, "ace_user_test_token");
        assert_eq!(loaded.user_id, "user_123");
        assert_eq!(loaded.email, "test@example.com");
    }

    #[test]
    fn test_load_user_auth_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert!(load_user_auth(&path).is_none());
    }

    #[test]
    fn test_load_user_auth_no_auth_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"device_id": "dev_1"}"#).unwrap();
        assert!(load_user_auth(&path).is_none());
    }

    #[test]
    fn test_get_or_create_device_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        let id1 = get_or_create_device_id(&path).unwrap();
        assert!(id1.starts_with("dev_"));

        // Should return same ID on second call
        let id2 = get_or_create_device_id(&path).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_get_or_create_device_id_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"device_id": "dev_custom123"}"#).unwrap();

        let id = get_or_create_device_id(&path).unwrap();
        assert_eq!(id, "dev_custom123");
    }

    #[test]
    fn test_get_device_id() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"device_id": "dev_existing"}"#).unwrap();

        assert_eq!(get_device_id(&path), Some("dev_existing".to_string()));
    }

    #[test]
    fn test_get_device_id_not_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        assert_eq!(get_device_id(&path), None);
    }

    #[test]
    fn test_get_device_id_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        assert_eq!(get_device_id(&path), None);
    }

    #[test]
    fn test_update_auth_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"token": "old", "user_id": "u1", "email": "e@e.com", "organizations": []}}"#,
        )
        .unwrap();

        update_auth_tokens(
            &path,
            "new_access",
            "new_refresh",
            "2025-01-01T00:00:00Z",
            "2025-01-08T00:00:00Z",
            Some("2025-02-01T00:00:00Z"),
        )
        .unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["auth"]["token"], "new_access");
        assert_eq!(config["auth"]["refresh_token"], "new_refresh");
        assert_eq!(config["auth"]["expires_at"], "2025-01-01T00:00:00Z");
        assert_eq!(config["auth"]["refresh_expires_at"], "2025-01-08T00:00:00Z");
        assert_eq!(config["auth"]["absolute_expires_at"], "2025-02-01T00:00:00Z");
        // Preserves existing fields
        assert_eq!(config["auth"]["user_id"], "u1");
    }

    #[test]
    fn test_update_auth_tokens_no_absolute() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(
            &path,
            r#"{"auth": {"token": "old", "user_id": "u1", "email": "e@e.com", "organizations": []}}"#,
        )
        .unwrap();

        update_auth_tokens(
            &path,
            "new_access",
            "new_refresh",
            "2025-01-01T00:00:00Z",
            "2025-01-08T00:00:00Z",
            None,
        )
        .unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(config["auth"]["token"], "new_access");
        assert!(config["auth"].get("absolute_expires_at").is_none());
    }

    #[test]
    fn test_update_auth_tokens_no_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = update_auth_tokens(&path, "t", "r", "e", "re", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No config file"));
    }

    #[test]
    fn test_update_auth_tokens_no_auth_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{}"#).unwrap();

        let result = update_auth_tokens(&path, "t", "r", "e", "re", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No auth block"));
    }

    // =========================================================================
    // Migration tests
    // =========================================================================

    #[test]
    fn test_migrate_config() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("old/config.json");
        let to = dir.path().join("new/config.json");
        fs::create_dir_all(from.parent().unwrap()).unwrap();
        fs::write(&from, r#"{"serverUrl": "https://example.com"}"#).unwrap();

        migrate_config(&from, &to).unwrap();

        assert!(to.exists());
        let content = fs::read_to_string(&to).unwrap();
        assert!(content.contains("example.com"));
    }

    #[test]
    fn test_ensure_config_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deep/nested/config.json");
        ensure_config_directory(&path).unwrap();
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_generate_device_id_format() {
        let id = generate_device_id();
        assert!(id.starts_with("dev_"));
        assert!(id.len() > 4);
    }
}
