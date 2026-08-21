//! Configuration file parsing.

use std::{fs::read_to_string, io};

use crate::device::*;

use lazy_static::lazy_static;
use lifxi::http::Client;

lazy_static! {
    /// The shared client to be used for all LIFX operations.
    pub static ref LIFX_CLIENT: Client = Client::new(LIFX_SECRET.to_string());
    /// The LIFX API token to be used.
    static ref LIFX_SECRET: String = CONFIG.lifx_secret.clone().expect("LIFX devices used without configuring a LIFX secret.");
    /// The parsed configuration file.
    pub static ref CONFIG: Config = Config::parse().expect("Failed to parse config file.");
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// The user's configured devices.
    pub devices: Vec<Device>,
    pub(crate) lifx_secret: Option<String>,
}

impl Config {
    /// Loads the config file from ~/.adm/config.toml.
    pub fn parse() -> Result<Self, io::Error> {
        let mut config_path = dirs::home_dir().unwrap();
        config_path.push(".adm/config.toml");
        let s = read_to_string(config_path)?;
        Ok(toml::from_str(&s).unwrap())
    }
    /// Finds the specified device in the list of configured devices.
    pub fn find<'a, S: ToString>(&'a self, s: S) -> Option<&'a Device> {
        let s = s.to_string();
        // TODO: Allow accessing devices by index as well.
        self.devices.iter().find(|device| {
            device.name.eq_ignore_ascii_case(&s)
                || device
                    .alternatives
                    .iter()
                    .map(|d| d.iter())
                    .flatten()
                    .any(|alt| alt.eq_ignore_ascii_case(&s))
        })
    }
}
