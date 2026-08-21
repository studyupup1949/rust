use clap::Parser;
use std::{net::IpAddr, sync::LazyLock};

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::parse);

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Config {
    #[arg(help = "Secret key", env = "AA_SECRET")]
    pub secret: String,

    #[arg(help = "Mirror domain", env = "AA_DOMAIN")]
    pub domain: String,

    #[arg(help = "Bind IP", env = "AA_BIND_IP", default_value = "127.0.0.1")]
    pub bind_ip: IpAddr,

    #[arg(help = "Bind port", env = "AA_BIND_PORT", default_value_t = 3030)]
    pub bind_port: u16,

    #[arg(
        help = "Debug logging",
        env = "AA_DEBUG_LOGGING",
        default_value_t = false
    )]
    pub debug_logging: bool,

    // limit max. bytes when receiving book URL/hash.
    // increase when using a longer mirror URL than 'annas-archive.org'
    #[arg(
        help = "Maximum HTTP request body size (in bytes)",
        env = "AA_MAX_BODY_SIZE",
        default_value_t = 96
    )]
    pub max_body_size: u64,
}
