use std::path::Path;

use crate::{Config, Engine, Result};

pub struct EngineBuilder {
    config: Config,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder {
    pub fn new() -> Self {
        let mut config = Config::default();
        let file = Path::new("config/actflow.toml");

        if file.exists() {
            config = Config::create(file);
        } else {
            if let Ok(env_config_path) = std::env::var("ACTFLOW_CONFIG") {
                let env_file = Path::new(&env_config_path);
                if env_file.exists() {
                    config = Config::create(env_file);
                }
            }
        }

        Self {
            config,
        }
    }

    pub fn set_config_source<T: AsRef<Path>>(
        mut self,
        source: T,
    ) -> Self {
        self.config = Config::create(source);
        self
    }

    pub fn config(
        mut self,
        config: Config,
    ) -> Self {
        self.config = config;
        self
    }

    pub fn build(&self) -> Result<Engine> {
        let engine = Engine::new_with_config(self.config.clone());

        Ok(engine)
    }
}
