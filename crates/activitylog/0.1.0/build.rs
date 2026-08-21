mod config;

use std::{fs, io::Write};
use config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let home = if cfg!(target_os = "linux") ||  cfg!(target_os = "macos") {
        std::env::var("HOME")?
    } else {
        std::env::var("HOMEPATH")?
    };
    let config_dir_path = format!("{home}/.activitylog");
    let config_info_path = format!("{config_dir_path}/config.toml");
    let config_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_info_path);
    let config = Config::new();
    let content = config.to_toml()?;
    if let Ok(mut f) = config_file {
        f.write_all(content.as_bytes())?;
    }
    config.create_elements()?;
    Ok(())
}