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
    #[error("an passphrase for encrypting puzzle inputs is required")]
    PassphraseRequired,
    #[error("failed to get the default cache directory for puzzles - this OS is not supported by the `directories` crate")]
    DefaultPuzzleDirError,
    #[error("failed to get the default cache directory for sessions - this OS is not supported by the `directories` crate")]
    DefaultSessonsDirError,
    #[error("{}", .0)]
    IoError(#[from] std::io::Error),
    #[error("{}", .0)]
    TomlError(#[from] toml::de::Error),
}

/// Configuration for the Advent of Code client.
///
/// Most users of this crate do not need to worry about how to initialize `Config`, or how to use
/// `ConfigBuilder` to create new `Config`s. Just use the `load_config()` function in this module to
/// get the behavior that is detailed in this crate's README.md.
#[derive(Clone, Default, Debug)]
pub struct Config {
    /// Your Advent of Code session cookie. TODO: rename all instances to `session`.
    pub session_id: Option<String>,
    /// Directory where puzzle inputs and answers are stored.
    pub puzzle_dir: PathBuf,
    /// Directory where per-session state (e.g., submission timeouts) is cached.
    pub sessions_dir: PathBuf,
    /// Passphrase used to encrypt puzzle inputs on disk.
    pub passphrase: String,
    /// Current time (usually UTC now, but can be overridden for testing).
    pub start_time: chrono::DateTime<chrono::Utc>,
}

/// A builder interface for specifying configuration settings to the Advent of Client client.
/// Configuration settings have sensible default values, and should only be changed when the
/// user wants custom behavior.
///
/// # Default Values
/// - `session_id`: The user's Advent of Code session cookie. **This is required for getting input
///   and submitting answers.**
/// - `passphrase`: The hostname of the current machine. **A custom passphrase is required if
///   `puzzle_dir` is changed.**
/// - `puzzle_dir`: A directory in the local user's cache dir (e.g., XDG_CACHE_HOME on Linux).
/// - `sessions_dir`: A directory in the local user's cache dir (e.g., XDG_CACHE_HOME on Linux).
pub struct ConfigBuilder {
    pub session_id: Option<String>,
    pub puzzle_dir: Option<PathBuf>,
    pub sessions_dir: Option<PathBuf>,
    pub passphrase: Option<String>,
    pub fake_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ConfigBuilder {
    /// Create a new `ConfigBuilder` object with all fields initialized to `None`.`
    pub fn new() -> Self {
        Self {
            session_id: None,
            puzzle_dir: None,
            sessions_dir: None,
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

    pub fn with_sessions_dir<P: Into<PathBuf>>(mut self, sessions_dir: P) -> Self {
        self.sessions_dir = Some(sessions_dir.into());
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

    /// Generate a `Config` object from the settings in this `ConfigBuilder` object.
    pub fn build(self) -> Result<Config, ConfigError> {
        // Use a default passphrase if the puzzle directory and the passphrase was not specified.
        let passphrase = self.passphrase.unwrap_or_else(|| {
            if self.puzzle_dir.is_none() {
                gethostname::gethostname().to_string_lossy().to_string()
            } else {
                String::new()
            }
        });

        // There must be a passphrase given when building the config.
        if passphrase.is_empty() {
            Err(ConfigError::PassphraseRequired)
        } else {
            let maybe_project_dir =
                directories::ProjectDirs::from(DIRS_QUALIFIER, DIRS_ORG, DIRS_APP);

            Ok(Config {
                session_id: self.session_id,
                puzzle_dir: self
                    .puzzle_dir
                    .or(maybe_project_dir
                        .as_ref()
                        .map(|p| p.cache_dir().join("puzzles").to_path_buf()))
                    .ok_or(ConfigError::DefaultPuzzleDirError)?,
                sessions_dir: self
                    .sessions_dir
                    .or(maybe_project_dir
                        .as_ref()
                        .map(|p| p.cache_dir().join("sessions").to_path_buf()))
                    .ok_or(ConfigError::DefaultPuzzleDirError)?,
                start_time: self.fake_time.unwrap_or(chrono::Utc::now()),
                passphrase,
            })
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Loads client options from the local machine.
///
/// The behavior of this function is covered in the `advent-of-code-data` [README.md](../README.md).
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

    // Read custom configuration path from `AOC_CONFIG_FILE` if it is set. Skip searching other
    // config paths if this environment variable is set.
    //
    // NOTE: Please keep this name consistent with README.md!
    const CUSTOM_CONFIG_ENV_KEY: &str = "AOC_CONFIG_FILE";

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
    fn config_uses_hostname_default_passphrase() {
        let config: Config = ConfigBuilder::new()
            .with_session_id("54321")
            .build()
            .unwrap();
        assert_eq!(
            config.passphrase,
            gethostname::gethostname().into_string().unwrap()
        );
    }

    #[test]
    fn config_must_specify_passphrase_if_puzzle_dir_changed() {
        let config = ConfigBuilder::new().with_session_id("my_session");
        assert!(config.build().is_ok());

        let config = ConfigBuilder::new()
            .with_session_id("my_session")
            .with_puzzle_dir("/tmp/puzzles");
        assert!(matches!(
            config.build(),
            Err(ConfigError::PassphraseRequired)
        ));
    }

    #[test]
    fn configs_are_built_with_config_builder() {
        let config: Config = ConfigBuilder::new()
            .with_session_id("54321")
            .with_puzzle_dir("/tmp/puzzle/dir")
            .with_sessions_dir("/tmp/path/to/sessions")
            .with_passphrase("this is my password")
            .build()
            .unwrap();

        assert_eq!(config.session_id, Some("54321".to_string()));
        assert_eq!(&config.passphrase, "this is my password");

        assert_eq!(
            config.puzzle_dir,
            PathBuf::from_str("/tmp/puzzle/dir").unwrap()
        );

        assert_eq!(
            config.sessions_dir,
            PathBuf::from_str("/tmp/path/to/sessions").unwrap()
        );
    }

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
        session_idX = "12345"
        "#;

        let options = ConfigBuilder::new().use_toml(config_text).unwrap();

        assert!(options.session_id.is_none());
    }

    #[test]
    fn set_client_options_from_toml_ignores_replace_me_values() {
        let config_text = r#"
        [client]
        session_id = "REPLACE_ME"
        puzzle_dir = "path/to/puzzle/dir"
        "#;

        let options = ConfigBuilder::new().use_toml(config_text).unwrap();

        assert!(options.session_id.is_none());
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("path/to/puzzle/dir").unwrap())
        );
    }
}
