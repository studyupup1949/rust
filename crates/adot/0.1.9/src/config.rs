use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum DotfileType {
    /// Single absolute symlink (default)
    Link,
    /// Plain copy
    Copy,
    /// Run through template engine then copy
    Template,
    /// Symlink each child in directory individually
    LinkChildren,
}

#[derive(Debug, Clone)]
pub struct Dotfile {
    pub dst: PathBuf,
    pub src: PathBuf,
    pub dtype: DotfileType,
}

#[derive(Debug, Clone)]
pub struct Profile {
    pub dotfiles: Vec<String>,
    pub include: Vec<String>,
    pub variables: HashMap<String, Variable>,
    pub dynvariables: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Value(String),
    Nested(HashMap<String, Variable>),
}

#[derive(Debug, Clone)]
pub struct Config {
    /// Relative path from config.yaml to dotfile sources (default: "dotfiles/")
    pub dotpath: PathBuf,
    pub dotfiles: HashMap<String, Dotfile>,
    pub profiles: HashMap<String, Profile>,
    pub variables: HashMap<String, Variable>,
    pub dynvariables: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dotpath: PathBuf::from("dotfiles"),
            dotfiles: HashMap::new(),
            profiles: HashMap::new(),
            variables: HashMap::new(),
            dynvariables: HashMap::new(),
        }
    }
}

impl Config {
    /// Load config from file.
    /// Priority: overwrite path > $XDG_CONFIG_HOME/adot/config.yaml > ~/.config/adot/config.yaml > ~/.adot/config.yaml
    /// Returns (Config, config_dir) where config_dir is the parent directory of config.yaml
    pub fn load(overwrite: Option<&PathBuf>) -> Result<(Self, PathBuf), String> {
        let path = resolve_config_path(overwrite)?;
        let config_dir = path
            .parent()
            .ok_or_else(|| "config path has no parent".to_string())?
            .canonicalize()
            .map_err(|e| format!("failed to resolve config dir: {e}"))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

        let config = crate::parser::parse(&content)?;
        config.validate()?;
        Ok((config, config_dir))
    }

    /// Validate config integrity after parsing.
    pub fn validate(&self) -> Result<(), String> {
        for (name, dotfile) in &self.dotfiles {
            // src must never equal dst
            if dotfile.src == dotfile.dst {
                return Err(format!(
                    "dotfile '{name}': src and dst are the same path: {}",
                    dotfile.src.display()
                ));
            }

            // dst must not point inside dotpath (would delete source files)
            if dotfile.dst.starts_with(&self.dotpath) {
                return Err(format!(
                    "dotfile '{name}': dst '{}' is inside dotpath '{}' — this would overwrite source files",
                    dotfile.dst.display(),
                    self.dotpath.display()
                ));
            }

            // dst must be set
            if dotfile.dst.as_os_str().is_empty() {
                return Err(format!("dotfile '{name}': dst is empty"));
            }

            // src must be set
            if dotfile.src.as_os_str().is_empty() {
                return Err(format!("dotfile '{name}': src is empty"));
            }
        }

        // validate profile references
        for (profile_name, profile) in &self.profiles {
            for dotfile_ref in &profile.dotfiles {
                if !self.dotfiles.contains_key(dotfile_ref) {
                    return Err(format!(
                        "profile '{profile_name}': references unknown dotfile '{dotfile_ref}'"
                    ));
                }
            }

            for include_ref in &profile.include {
                if !self.profiles.contains_key(include_ref) {
                    return Err(format!(
                        "profile '{profile_name}': includes unknown profile '{include_ref}'"
                    ));
                }
            }
        }

        Ok(())
    }
}

pub fn resolve_config_path(overwrite: Option<&PathBuf>) -> Result<PathBuf, String> {
    if let Some(path) = overwrite {
        if !path.exists() {
            return Err(format!("config not found: {}", path.display()));
        }
        return Ok(path.clone());
    }

    let candidates = build_candidates();
    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    Err(format!(
        "no config found, searched:\n{}",
        candidates
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn build_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        candidates.push(PathBuf::from(xdg).join("adot/config.yaml"));
    }

    if let Some(home) = home_dir() {
        candidates.push(home.join(".config/adot/config.yaml"));
        candidates.push(home.join(".adot/config.yaml"));
    }

    candidates
}

fn home_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}
