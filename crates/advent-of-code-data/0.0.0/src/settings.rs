use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

pub struct ClientOptions {
    pub session_id: Option<String>,
    pub puzzle_dir: Option<PathBuf>,
    pub encryption_token: Option<String>,
    pub fake_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ClientOptions {
    pub fn new() -> Self {
        Self {
            session_id: None,
            puzzle_dir: None,
            encryption_token: None,
            fake_time: None,
        }
    }

    pub fn with_config_file<P: AsRef<Path>>(self, path: P) -> Self {
        // TODO: raise error if the file does not exist
        // TODO: raise error if JSON parsing fails
        // TODO: add tests
        let config_text = fs::read_to_string(&path).expect("config file should exist");
        self.with_json_config(&config_text)
            .expect("json parsing failed")
    }

    pub fn with_local_dir_config(mut self) -> Self {
        // TODO: log if file is found/not found
        // TODO: add tests
        let local_config_path = std::env::current_dir()
            .expect("current_dir is expected to work")
            .join("aoc_settings.json");

        if local_config_path.exists() {
            self = self.with_config_file(local_config_path);
        }

        self
    }

    pub fn with_user_config(self) -> Self {
        // TODO: implement loading config from path relative to home directory.
        // TODO: check if env variable has changed the user config path.
        // TODO: log if file is found/not found
        // TODO: add tests
        self
    }

    pub fn with_env_vars(mut self) -> Self {
        // TODO: log when keys are found and used
        // TODO: tests
        const SESSION_ID_ENV_KEY: &str = "AOC_SESSION";

        if let Ok(v) = std::env::var(SESSION_ID_ENV_KEY) {
            tracing::debug!(
                "using session id `{}` from env var {}",
                v,
                SESSION_ID_ENV_KEY
            );
            self.session_id = Some(v);
        }

        // TODO: add key for puzzle directory.
        // TODO: add key for encryption token.

        self
    }

    pub fn with_json_config(mut self, json_config: &str) -> serde_json::Result<Self> {
        const SESSION_ID_KEY: &str = "session_id";
        const PUZZLE_DIR_KEY: &str = "puzzle_dir";
        const ENCRYPTION_TOKEN_KEY: &str = "encryption_token";

        fn try_get_key<F: FnOnce(&str)>(
            group: &serde_json::Map<String, serde_json::Value>,
            key: &str,
            setter: F,
        ) {
            if group.contains_key(key) {
                match &group[key] {
                    serde_json::Value::String(s) => setter(s),
                    _ => {
                        // TODO: convert to Error
                        panic!("{} key must be a string value", key)
                    }
                };
            }
        }

        let j: serde_json::Value = serde_json::from_str(json_config)?;

        // TODO: log each key that is found and used.

        match j {
            serde_json::Value::Object(group) => {
                try_get_key(&group, SESSION_ID_KEY, |v| {
                    self.session_id = Some(v.to_string())
                });

                try_get_key(&group, PUZZLE_DIR_KEY, |v| {
                    self.puzzle_dir = Some(PathBuf::from_str(v).unwrap())
                });

                try_get_key(&group, ENCRYPTION_TOKEN_KEY, |v| {
                    self.encryption_token = Some(v.to_string())
                });
            }
            _ => {
                // TODO: convert to Error
                panic!("expected json config to be an object");
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

    pub fn with_encryption_token<S: Into<String>>(mut self, encryption_token: S) -> Self {
        self.encryption_token = Some(encryption_token.into());
        self
    }

    pub fn with_fake_time(mut self, fake_time: chrono::DateTime<chrono::Utc>) -> Self {
        self.fake_time = Some(fake_time);
        self
    }
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self::new()
            .with_user_config()
            .with_local_dir_config()
            .with_env_vars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_options_are_none_by_default() {
        let options = ClientOptions::new();
        assert!(options.session_id.is_none());
        assert!(options.puzzle_dir.is_none());
        assert!(options.encryption_token.is_none());
    }

    #[test]
    fn client_can_overwrite_options() {
        let options = ClientOptions::new()
            .with_encryption_token("12345")
            .with_encryption_token("54321");

        assert!(options.session_id.is_none());
        assert!(options.puzzle_dir.is_none());
        assert_eq!(options.encryption_token, Some("54321".to_string()));
    }

    #[test]
    fn set_client_options_with_builder_funcs() {
        let options = ClientOptions::new()
            .with_session_id("MY_SESSION_ID")
            .with_puzzle_dir("MY_CACHE_DIR")
            .with_encryption_token("MY_PASSWORD");

        assert_eq!(options.session_id, Some("MY_SESSION_ID".to_string()));
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("MY_CACHE_DIR").unwrap())
        );
        assert_eq!(options.encryption_token, Some("MY_PASSWORD".to_string()));
    }

    #[test]
    fn set_client_options_from_json() {
        let json_data = r#"
        {
            "session_id": "12345",
            "puzzle_dir": "path/to/puzzle/dir",
            "encryption_token": "foobar"
        }
        "#;

        let options = ClientOptions::new().with_json_config(json_data).unwrap();

        assert_eq!(options.session_id, Some("12345".to_string()));
        assert_eq!(
            options.puzzle_dir,
            Some(PathBuf::from_str("path/to/puzzle/dir").unwrap())
        );
        assert_eq!(options.encryption_token, Some("foobar".to_string()));
    }

    #[test]
    fn set_client_options_from_json_ignores_missing_fields() {
        let json_data = r#"
        {
            "session_id": "12345",
            "encryption_token_XXXX": "foobar"
        }
        "#;

        let options = ClientOptions::new().with_json_config(json_data).unwrap();

        assert_eq!(options.session_id, Some("12345".to_string()));
        assert!(options.puzzle_dir.is_none());
        assert!(options.encryption_token.is_none());
    }
}
