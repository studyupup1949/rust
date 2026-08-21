pub mod config;

#[cfg(test)]
mod tests {
    use super::config::ProjectConfig;
    use std::path::Path;

    #[test]
    fn starter_yaml_is_valid() {
        let yaml = ProjectConfig::starter_yaml();
        let _cfg: ProjectConfig = serde_yaml::from_str(yaml)
            .expect("starter YAML must be valid");
    }
}
