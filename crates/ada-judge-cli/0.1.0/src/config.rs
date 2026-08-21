use std::path::PathBuf;

use anyhow::Ok;
use directories::ProjectDirs;
use serde::Deserialize;
use tokio::fs::read_to_string;

use crate::Cli;

#[derive(Deserialize)]
pub struct Config {
    pub db_url: String,
}

fn default_config_path() -> Option<PathBuf> {
    let proj = ProjectDirs::from("com", "oneprogboy", "ada-judge-cli")?;
    let dir = proj.config_dir();
    println!("{}", dir.display());

    Some(dir.join("config.toml"))
}

pub async fn load_config(cli: &Cli) -> anyhow::Result<Config> {
    let cfg_path = cli
        .config
        .clone()
        .map_or_else(|| default_config_path().filter(|path| path.exists()), Some);

    Ok(match cfg_path {
        Some(path) => toml::from_str(&read_to_string(path).await.expect("Invalid config path"))
            .map_err(|e| panic!("Invalid config: {e}"))?,
        None => {
            panic!("No config was provided!")
        }
    })
}
