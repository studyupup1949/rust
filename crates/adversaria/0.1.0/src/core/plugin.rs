use crate::core::{AttackSuite, Result};
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn load_suites(&self) -> Result<Vec<AttackSuite>>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn load_all_suites(&self) -> Result<Vec<AttackSuite>> {
        let mut all_suites = Vec::new();

        for plugin in &self.plugins {
            match plugin.load_suites().await {
                Ok(suites) => all_suites.extend(suites),
                Err(e) => {
                    tracing::warn!("Failed to load suites from plugin {}: {}", plugin.name(), e);
                }
            }
        }

        Ok(all_suites)
    }

    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins
            .iter()
            .map(|p| (p.name(), p.version()))
            .collect()
    }

    pub async fn load_from_directory<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
