use clap::Parser;
use std::{net::IpAddr, sync::LazyLock};

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::parse);

#[derive(Debug, clap::Parser)]
#[command(version, about, arg_required_else_help = true)]
pub struct Config {
    #[arg(help = "Secret key", env = "AA_SECRET")]
    secret: String,

    #[arg(help = "Mirror domain", env = "AA_DOMAIN")]
    domain: String,

    #[arg(help = "Bind IP", env = "AA_BIND_IP", default_value = "127.0.0.1")]
    bind_ip: IpAddr,

    #[arg(help = "Bind port", env = "AA_BIND_PORT", default_value_t = 3030)]
    bind_port: u16,

    #[arg(
        help = "Debug logging",
        env = "AA_DEBUG_LOGGING",
        default_value_t = false
    )]
    debug_logging: bool,

    // limit max. bytes when receiving book URL/hash.
    // increase when using a longer mirror URL than 'annas-archive.org'
    #[arg(
        help = "Maximum HTTP request body size (in bytes)",
        env = "AA_MAX_BODY_SIZE",
        default_value_t = 96
    )]
    max_body_size: u64,
}

impl Config {
    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn bind_ip(&self) -> IpAddr {
        self.bind_ip
    }

    pub fn bind_port(&self) -> u16 {
        self.bind_port
    }

    pub fn debug_logging(&self) -> bool {
        self.debug_logging
    }

    pub fn max_body_size(&self) -> u64 {
        self.max_body_size
    }
}
