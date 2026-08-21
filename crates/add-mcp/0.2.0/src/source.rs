use crate::error::{AddMcpError, Result};
use crate::types::{PackageManager, Source, Transport};
use std::path::Path;

/// Parse a source string into a `Source` variant.
///
/// Heuristics (when `package_manager` is None):
/// - Starts with `http://` or `https://` → URL
/// - Starts with `@` → Package(Npm)
/// - First path segment contains `.` and has `/` → Package(Go) (e.g. `github.com/user/repo`)
/// - Bare name, doesn't exist on disk → Package(Npm)
/// - Otherwise → Command (split on whitespace for args)
///
/// When `package_manager` is Some, the source is treated as a package name for that manager
/// (unless it's a URL).
pub fn parse_source(
    input: &str,
    transport: Option<Transport>,
    package_manager: Option<PackageManager>,
) -> Result<Source> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AddMcpError::InvalidSource("empty source".into()));
    }

    // URL
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let t = transport.unwrap_or(Transport::Sse);
        return Ok(Source::Url {
            url: trimmed.to_string(),
            transport: t,
        });
    }

    // Explicit package manager
    if let Some(manager) = package_manager {
        return Ok(Source::Package {
            manager,
            package: trimmed.to_string(),
        });
    }

    // npm scoped package: starts with @
    if trimmed.starts_with('@') {
        return Ok(Source::Package {
            manager: PackageManager::Npm,
            package: trimmed.to_string(),
        });
    }

    // Go module: first segment contains a dot and has slashes (e.g. github.com/user/repo)
    if looks_like_go_module(trimmed) {
        return Ok(Source::Package {
            manager: PackageManager::Go,
            package: trimmed.to_string(),
        });
    }

    // Bare package name (no path separators, no whitespace, doesn't exist on disk) → npm
    if is_package_like(trimmed) && !Path::new(trimmed).exists() {
        return Ok(Source::Package {
            manager: PackageManager::Npm,
            package: trimmed.to_string(),
        });
    }

    // Command: split on whitespace
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let command = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    Ok(Source::Command { command, args })
}

/// Resolve a `Source::Package` into a `Source::Command` using the appropriate runner.
///
/// - npm: `npx -y <package>`
/// - pip: `uvx <package>`
/// - go: `go run <module>@latest`
/// - cargo: `~/.cargo/bin/<name>` (absolute path, since AI clients don't expand ~)
pub fn resolve_package(source: &Source) -> Source {
    match source {
        Source::Package { manager, package } => match manager {
            PackageManager::Npm => Source::Command {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), package.clone()],
            },
            PackageManager::Pip => Source::Command {
                command: "uvx".to_string(),
                args: vec![package.clone()],
            },
            PackageManager::Go => Source::Command {
                command: "go".to_string(),
                args: vec!["run".to_string(), format!("{package}@latest")],
            },
            PackageManager::Cargo => {
                let bin_name = package.rsplit('/').next().unwrap_or(package);
                let path = dirs::home_dir()
                    .map(|h| h.join(".cargo/bin").join(bin_name))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!(".cargo/bin/{bin_name}"));
                Source::Command {
                    command: path,
                    args: vec![],
                }
            }
        },
        other => other.clone(),
    }
}

/// Check if a string looks like a package name (no path separators, no whitespace).
fn is_package_like(s: &str) -> bool {
    !s.contains('/') && !s.contains('\\') && !s.contains(' ') && !s.starts_with('.')
}

/// Check if a string looks like a Go module path (e.g. `github.com/user/repo`).
fn looks_like_go_module(s: &str) -> bool {
    if let Some(first_segment) = s.split('/').next() {
        // Must have at least one slash AND the first segment must contain a dot
        s.contains('/') && first_segment.contains('.')
    } else {
        false
    }
}

/// Infer a server name from a source.
pub fn infer_name(source: &Source) -> Result<String> {
    match source {
        Source::Command { command, .. } => {
            let path = Path::new(command);
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| AddMcpError::CannotInferName(command.clone()))?;
            Ok(stem.to_string())
        }
        Source::Url { url, .. } => {
            // Use the hostname
            url::Url::parse(url)
                .ok()
                .and_then(|u| u.host_str().map(|h| h.to_string()))
                .ok_or_else(|| AddMcpError::CannotInferName(url.clone()))
        }
        Source::Package { manager, package } => {
            match manager {
                PackageManager::Npm => {
                    // @scope/name → name; bare-name → bare-name
                    let name = if let Some((_scope, name)) = package.split_once('/') {
                        name
                    } else {
                        package.as_str()
                    };
                    Ok(name.to_string())
                }
                PackageManager::Pip => {
                    // pip packages use the bare name
                    Ok(package.clone())
                }
                PackageManager::Go => {
                    // github.com/user/repo → repo
                    let name = package.rsplit('/').next().unwrap_or(package);
                    Ok(name.to_string())
                }
                PackageManager::Cargo => {
                    // crate name or path → last segment
                    let name = package.rsplit('/').next().unwrap_or(package);
                    Ok(name.to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let s = parse_source("https://example.com/mcp", None, None).unwrap();
        match s {
            Source::Url { url, transport } => {
                assert_eq!(url, "https://example.com/mcp");
                assert_eq!(transport, Transport::Sse);
            }
            _ => panic!("expected Url"),
        }
    }

    #[test]
    fn parse_command_with_args() {
        let s = parse_source("/usr/bin/mcp-watch serve --port 8080", None, None).unwrap();
        match s {
            Source::Command { command, args } => {
                assert_eq!(command, "/usr/bin/mcp-watch");
                assert_eq!(args, vec!["serve", "--port", "8080"]);
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn parse_npm_scoped() {
        let s = parse_source("@anthropic/mcp-server", None, None).unwrap();
        match s {
            Source::Package { manager, package } => {
                assert_eq!(manager, PackageManager::Npm);
                assert_eq!(package, "@anthropic/mcp-server");
            }
            _ => panic!("expected Package"),
        }
    }

    #[test]
    fn parse_go_module() {
        let s = parse_source("github.com/user/mcp-server", None, None).unwrap();
        match s {
            Source::Package { manager, package } => {
                assert_eq!(manager, PackageManager::Go);
                assert_eq!(package, "github.com/user/mcp-server");
            }
            _ => panic!("expected Package(Go)"),
        }
    }

    #[test]
    fn parse_explicit_pip() {
        let s = parse_source("mcp-server-fetch", None, Some(PackageManager::Pip)).unwrap();
        match s {
            Source::Package { manager, package } => {
                assert_eq!(manager, PackageManager::Pip);
                assert_eq!(package, "mcp-server-fetch");
            }
            _ => panic!("expected Package(Pip)"),
        }
    }

    #[test]
    fn parse_explicit_cargo() {
        let s = parse_source("my-mcp-server", None, Some(PackageManager::Cargo)).unwrap();
        match s {
            Source::Package { manager, package } => {
                assert_eq!(manager, PackageManager::Cargo);
                assert_eq!(package, "my-mcp-server");
            }
            _ => panic!("expected Package(Cargo)"),
        }
    }

    #[test]
    fn resolve_npm() {
        let src = Source::Package {
            manager: PackageManager::Npm,
            package: "@org/mcp-server".to_string(),
        };
        match resolve_package(&src) {
            Source::Command { command, args } => {
                assert_eq!(command, "npx");
                assert_eq!(args, vec!["-y", "@org/mcp-server"]);
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn resolve_pip() {
        let src = Source::Package {
            manager: PackageManager::Pip,
            package: "mcp-server-fetch".to_string(),
        };
        match resolve_package(&src) {
            Source::Command { command, args } => {
                assert_eq!(command, "uvx");
                assert_eq!(args, vec!["mcp-server-fetch"]);
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn resolve_go() {
        let src = Source::Package {
            manager: PackageManager::Go,
            package: "github.com/user/mcp".to_string(),
        };
        match resolve_package(&src) {
            Source::Command { command, args } => {
                assert_eq!(command, "go");
                assert_eq!(args, vec!["run", "github.com/user/mcp@latest"]);
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn resolve_cargo() {
        let src = Source::Package {
            manager: PackageManager::Cargo,
            package: "my-mcp".to_string(),
        };
        match resolve_package(&src) {
            Source::Command { command, args } => {
                assert!(command.ends_with(".cargo/bin/my-mcp"));
                assert!(args.is_empty());
            }
            _ => panic!("expected Command"),
        }
    }

    #[test]
    fn infer_name_from_command() {
        let s = Source::Command {
            command: "/home/user/bin/mcp-watch".into(),
            args: vec![],
        };
        assert_eq!(infer_name(&s).unwrap(), "mcp-watch");
    }

    #[test]
    fn infer_name_from_npm() {
        let s = Source::Package {
            manager: PackageManager::Npm,
            package: "@anthropic/mcp-server".into(),
        };
        assert_eq!(infer_name(&s).unwrap(), "mcp-server");
    }

    #[test]
    fn infer_name_from_pip() {
        let s = Source::Package {
            manager: PackageManager::Pip,
            package: "mcp-server-fetch".into(),
        };
        assert_eq!(infer_name(&s).unwrap(), "mcp-server-fetch");
    }

    #[test]
    fn infer_name_from_go() {
        let s = Source::Package {
            manager: PackageManager::Go,
            package: "github.com/user/mcp-server".into(),
        };
        assert_eq!(infer_name(&s).unwrap(), "mcp-server");
    }

    #[test]
    fn infer_name_from_url() {
        let s = Source::Url {
            url: "https://api.example.com/mcp".into(),
            transport: Transport::Sse,
        };
        assert_eq!(infer_name(&s).unwrap(), "api.example.com");
    }

    #[test]
    fn go_module_heuristic() {
        assert!(looks_like_go_module("github.com/user/repo"));
        assert!(looks_like_go_module("gitlab.com/org/tool"));
        assert!(!looks_like_go_module("some-package"));
        assert!(!looks_like_go_module("@scope/package"));
    }
}
