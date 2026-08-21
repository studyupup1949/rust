use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

// TODO: in the documentation for ClientOptions, explain the builder pattern used.
// TODO: in the documentation for ClientOptions, explain that with_* calls overwrite previous values.
// TODO: need to write tests.
// TODO: config for custom http endpoint.

const DIRS_QUALIFIER: &str = "com";
const DIRS_ORG: &str = "smacdo";
const DIRS_APP: &str = "advent_of_code_data";

const CONFIG_FILENAME: &str = "aoc_settings.toml";
const EXAMPLE_CONFIG_FILENAME: &str = "aoc_settings.example.toml";
const HOME_DIR_CONFIG_FILENAME: &str = ".aoc_settings.toml";

const EXAMPLE_CONFIG_TEXT: &str = r#"[client]
# passphrase = "REPLACE_ME"  # Used to encrypt/decrypt the puzzle cache.
# session_id = "REPLACE_ME"  # See "Finding your Advent of Code session cookie" in the README for help.
"#;

/// Errors that can occur when configuring client settings.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(
        "session cookie required; use the advent-of-code-data README to learn how to obtain this"
    )]
    SessionIdRequired,
    #[error("an passphrase for encrypting puzzle inputs is required")]
    PassphraseRequired,
    #[error("a puzzle cache directory is required")]
    PuzzleCacheDirRequired,
    #[error("a session cache directory is required")]
    SessionCacheDirRequired,
    #[error("{}", .0)]
    IoError(#[from] std::io::Error),
    #[error("{}", .0)]
    TomlError(#[from] toml::de::Error),
}

/// Configuration for the Advent of Code client.
///
/// Created from `ClientOptions` and used internally by `WebClient`. All fields are public to allow
/// inspection and advanced use cases, but typically you should not modify these directly after
/// client creation.
#[derive(Default, Debug)]
pub struct Config {
    /// Your Advent of Code session cookie (from the browser's cookies).
    pub session_id: String,
    /// Directory where puzzle inputs and answers are stored.
    pub puzzle_dir: PathBuf,
    /// Directory where per-session state (e.g., submission timeouts) is cached.
    pub sessions_dir: PathBuf,
    /// Passphrase used to encrypt puzzle inputs on disk.
    pub passphrase: String,
    /// Current time (usually UTC now, but can be overridden for testing).
    pub start_time: chrono::DateTime<chrono::Utc>,
}

//
pub struct ConfigBuilder {
    pub session_id: Option<String>,
    pub puzzle_dir: Option<PathBuf>,
    pub sessions_dir: Option<PathBuf>,
    pub passphrase: Option<String>,
    pub fake_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        // TODO: new should set these to empty, and then there should be a check at the end to
        //       validate cache dirs were provided.
        let project_dir = directories::ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP)
            .expect("TODO: implement default fallback cache directories if this fails");

        Self {
            session_id: None,
            puzzle_dir: Some(project_dir.cache_dir().join("puzzles").to_path_buf()),
            sessions_dir: Some(project_dir.cache_dir().join("sessions").to_path_buf()),
            passphrase: None,
            fake_time: None,
        }
    }

    /// Loads configuration values from string containing TOML formatted text. Configuration values
    /// loaded here will overwrite previously loaded values.
    pub fn use_toml(mut self, config_text: &str) -> Result<Self, ConfigError> {
        const CLIENT_TABLE_NAME: &str = "client";
        const SESSIONS_DIR_KEY: &str = "sessions_dir";
        const SESSION_ID_KEY: &str = "session_id";
        const PUZZLE_DIR_KEY: &str = "puzzle_dir";
        const PASSPHRASE_KEY: &str = "passphrase";
        const REPLACE_ME: &str = "REPLACE_ME";

        fn try_read_key<F: FnOnce(&str)>(table: &toml::Table, key: &str, setter: F) {
            match table.get(key).as_ref() {
                Some(toml::Value::String(s)) => {
                    if s == REPLACE_ME {
                        tracing::debug!("ignoring TOML key {key} because value is `{REPLACE_ME}`");
                    } else {
                        tracing::debug!("found TOML key `{key}` with value `{s}`");
                        setter(s)
                    }
                }
                None => {
                    tracing::debug!("TOML key {key} not present, or its value was not a string");
                }
                _ => {
                    tracing::warn!("TOML key {key} must be string value");
                }
            };
        }

        let toml: toml::Table = config_text.parse::<toml::Table>()?;

        match toml.get(CLIENT_TABLE_NAME) {
            Some(toml::Value::Table(client_config)) => {
                try_read_key(client_config, PASSPHRASE_KEY, |v| {
                    self.passphrase = Some(v.to_string())
                });

                try_read_key(client_config, SESSION_ID_KEY, |v| {
                    self.session_id = Some(v.to_string())
                });

                try_read_key(client_config, PUZZLE_DIR_KEY, |v| {
                    self.puzzle_dir = Some(PathBuf::from(v))
                });

                try_read_key(client_config, SESSIONS_DIR_KEY, |v| {
                    self.sessions_dir = Some(PathBuf::from(v))
                });
            }
            _ => {
                tracing::warn!(
                    "TOML table {CLIENT_TABLE_NAME} was missing; this config will be skipped!"
                );
            }
        }

        Ok(self)
    }

    pub fn with_session_id<S: Into<String>>(mut self, session_id: S) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn with_puzzle_dir<P: Into<PathBuf>>(mut self, puzzle_dir: P) -> Self {
        self.puzzle_dir = Some(puzzle_dir.into());
        self
    }

    pub fn with_passphrase<S: Into<String>>(mut self, passphrase: S) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn with_fake_time(mut self, fake_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.fake_time = Some(fake_time);
        self
    }

    pub fn build(self) -> Result<Config, ConfigError> {
        Ok(Config {
            session_id: self.session_id.ok_or(ConfigError::SessionIdRequired)?,
            puzzle_dir: self.puzzle_dir.ok_or(ConfigError::PuzzleCacheDirRequired)?,
            sessions_dir: self
                .sessions_dir
                .ok_or(ConfigError::SessionCacheDirRequired)?,
            passphrase: self.passphrase.ok_or(ConfigError::PassphraseRequired)?,
            start_time: self.fake_time.unwrap_or(chrono::Utc::now()),
        })
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Loads client options in the following order:
///   1. User's shared configuration directory (ie, XDG_CONFIG_HOME or %LOCALAPPDATA%).
///   2. Current directory.
///   3. Environment variables.
pub fn load_config() -> Result<ConfigBuilder, ConfigError> {
    let mut config: ConfigBuilder = Default::default();

    config = read_config_from_user_config_dirs(Some(config))?;
    config = read_config_from_current_dir(Some(config))?;
    config = read_config_from_env_vars(Some(config));

    Ok(config)
}

/// Loads configuration values from a TOML file.
pub fn read_config_from_file<P: AsRef<Path>>(
    config: Option<ConfigBuilder>,
    path: P,
) -> Result<ConfigBuilder, ConfigError> {
    let config = config.unwrap_or_default();
    let config_text = fs::read_to_string(&path)?;

    config.use_toml(&config_text)
}

/// Loads configuration values from a TOML file in the working directory.
pub fn read_config_from_current_dir(
    config: Option<ConfigBuilder>,
) -> Result<ConfigBuilder, ConfigError> {
    let mut config = config.unwrap_or_default();

    match std::env::current_dir() {
        Ok(current_dir) => {
            let local_config_path = current_dir.join(CONFIG_FILENAME);
            tracing::debug!("loading current directory config values from: {local_config_path:?}");

            if local_config_path.exists() {
                config = read_config_from_file(Some(config), local_config_path)?;
            } else {
                tracing::warn!("loading config from current directory will be skipped because {local_config_path:?} does not exist")
            }
        }
        Err(e) => {
            tracing::error!("loading config from current directory will be skipped because {e}")
        }
    }

    Ok(config)
}

/// Loads configuration data from a user's config directory relative to their home directory.
/// Any option values loaded here will overwrite values loaded previously.
pub fn read_config_from_user_config_dirs(
    config: Option<ConfigBuilder>,
) -> Result<ConfigBuilder, ConfigError> {
    let mut config = config.unwrap_or_default();

    // Read custom configuration path from `AOC_CONFIG_PATH` if it is set. Do not continue
    // looking for user config if this was set.
    const CUSTOM_CONFIG_ENV_KEY: &str = "AOC_CONFIG_PATH";

    if let Ok(custom_config_path) = std::env::var(CUSTOM_CONFIG_ENV_KEY) {
        if std::fs::exists(&custom_config_path).unwrap_or(false) {
            tracing::debug!("loading user config at: {custom_config_path:?}");
            config = read_config_from_file(Some(config), custom_config_path)?;
        } else {
            tracing::debug!("no user config found at: {custom_config_path:?}");
        }

        return Ok(config);
    } else {
        tracing::debug!(
            "skipping custom user config because env var `{CUSTOM_CONFIG_ENV_KEY}` is not set"
        );
    }

    // Try reading from the $XDG_CONFIG_HOME / %LOCALAPPDATA%.
    if let Some(project_dir) = directories::ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP) {
        let config_dir = project_dir.config_dir();
        let example_config_path = config_dir.join(EXAMPLE_CONFIG_FILENAME);

        // Create the application's config dir if its missing.
        if !std::fs::exists(config_dir).unwrap_or(false) {
            std::fs::create_dir_all(config_dir).unwrap_or_else(|e| {
                tracing::debug!("failed to create app config dir: {e:?}");
            });
        }

        // Create an example config file in the user config dir to illustrate some of the
        // configuration options users can set.
        if !std::fs::exists(&example_config_path).unwrap_or(false) {
            tracing::debug!("created example config at {example_config_path:?}");

            std::fs::write(example_config_path, EXAMPLE_CONFIG_TEXT).unwrap_or_else(|e| {
                tracing::debug!("failed to create example config: {e:?}");
            });
        }

        // Load the user config if it exists.
        let config_path = config_dir.join(CONFIG_FILENAME);

        if std::fs::exists(&config_path).unwrap_or(false) {
            tracing::debug!("loading user config at: {config_path:?}");
            return read_config_from_file(Some(config), config_path);
        } else {
            tracing::debug!("no user config found at: {config_path:?}");
        }
    } else {
        tracing::debug!("could not calculate user config dir on this machine");
    }

    // Try reading from the home directory.
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let home_config_path = base_dirs.home_dir().join(HOME_DIR_CONFIG_FILENAME);

        if std::fs::exists(&home_config_path).unwrap_or(false) {
            tracing::debug!("loading user config at: {home_config_path:?}");
            config = read_config_from_file(Some(config), home_config_path)?;
        } else {
            tracing::debug!("no user config found at: {home_config_path:?}");
        }
    }

    Ok(config)
}

/// Returns a copy of `config` with settings that match any non-empty Advent of Code environment
/// variables.
pub fn read_config_from_env_vars(config: Option<ConfigBuilder>) -> ConfigBuilder {
    /// NOTE: Keep these environment variable names in sync with the README and other documentation!
    const SESSION_ID_ENV_KEY: &str = "AOC_SESSION";
    const PASSPHRASE_ENV_KEY: &str = "AOC_PASSPHRASE";
    const PUZZLE_DIR_KEY: &str = "AOC_PUZZLE_DIR";
    const SESSIONS_DIR_KEY: &str = "AOC_SESSIONS_DIR";

    let mut config = config.unwrap_or_default();

    fn try_read_env_var<F: FnOnce(String)>(name: &str, setter: F) {
        if let Ok(v) = std::env::var(name) {
            tracing::debug!("found env var `{name}` with value `{v}`");
            setter(v)
        }
    }

    try_read_env_var(SESSION_ID_ENV_KEY, |v| {
        config.session_id = Some(v);
    });

    try_read_env_var(PASSPHRASE_ENV_KEY, |v| {
        config.passphrase = Some(v);
    });

    try_read_env_var(PUZZLE_DIR_KEY, |v| {
        config.puzzle_dir = Some(PathBuf::from(v));
    });

    try_read_env_var(SESSIONS_DIR_KEY, |v| {
        config.sessions_dir = Some(PathBuf::from(v));
    });

    config
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn client_can_overwrite_options() {
        let mut options = ConfigBuilder::new().with_passphrase("12345");
        assert_eq!(options.passphrase, Some("12345".to_string()));

        options = options.with_passphrase("54321");
        assert_eq!(options.passphrase, Some("54321".to_string()));
    }

    #[test]
    fn set_client_options_with_builder_funcs() {
        let options = ConfigBuilder::new()
            .with_session_id("MY_SESSION_ID")
            .with_puzzle_dir("MY_CACHE_DIR")
            .with_passphrase("MY_PASSWORD");

        assert_eq!(options.session_id, Some("MY_SESSION_ID".to_string()));
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("MY_CACHE_DIR").unwrap())
        );
        assert_eq!(options.passphrase, Some("MY_PASSWORD".to_string()));
    }

    #[test]
    fn set_client_options_from_toml() {
        let config_text = r#"
        [client]
        session_id = "12345"
        puzzle_dir = "path/to/puzzle/dir"
        sessions_dir = "another/path/to/blah"
        passphrase = "foobar"
        "#;

        let options = ConfigBuilder::new().use_toml(config_text).unwrap();

        assert_eq!(options.session_id, Some("12345".to_string()));
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("path/to/puzzle/dir").unwrap())
        );
        assert_eq!(
            options.sessions_dir,
            Some(PathBuf::from_str("another/path/to/blah").unwrap())
        );
        assert_eq!(options.passphrase, Some("foobar".to_string()));
    }

    #[test]
    fn set_client_options_from_toml_ignores_missing_fields() {
        let config_text = r#"
        [client]
        session_id = "12345"
        passphrase_XXXX = "foobar"
        "#;

        let options = ConfigBuilder::new().use_toml(config_text).unwrap();

        assert_eq!(options.session_id, Some("12345".to_string()));
        assert!(options.passphrase.is_none());
    }

    #[test]
    fn set_client_options_from_toml_ignores_replace_me_values() {
        let config_text = r#"
        [client]
        session_id = "REPLACE_ME"
        puzzle_dir = "path/to/puzzle/dir"
        passphrase = "REPLACE_ME"
        "#;

        let options = ConfigBuilder::new().use_toml(config_text).unwrap();

        assert!(options.session_id.is_none());
        assert!(options.passphrase.is_none());
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("path/to/puzzle/dir").unwrap())
        );
    }
}
