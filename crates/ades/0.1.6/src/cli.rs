pub use aoko::no_std::algebraic::sum::TimeUnit;
use clap::Parser;

/// AES & DES, Encryption and Decryption.

#[derive(Parser)]
#[clap(version = "0.1.6", author = "hzqd <hzqelf@yeah.net>")]
pub struct Args {
    /// Specify the input file name
    #[clap(short, long)]
    pub input: String,

    /// Specify the output file name
    #[clap(short, long)]
    pub output: String,

    /// Specify the AES key
    #[clap(short, long, default_value = "")]
    pub aes_key: String,

    /// Specify the DES key
    #[clap(short, long, default_value = "")]
    pub des_key: String,

    /// Specify the time unit, support nanos, micros, millis, secs
    #[clap(short, long, default_value = "millis")]
    pub time: TimeUnit,

    /// Set the Encryption and Decryption ways
    #[clap(subcommand)]
    pub subcmd: Algorithm,
}

#[derive(Parser)]
pub enum Algorithm {
    /// A subcommand for specify using AES and DES to Decrypt or Encrypt by -e
    M(Encryption),
    /// A subcommand for specify using AES to Decrypt or Encrypt by -e
    AES(Encryption),
    /// A subcommand for specify using DES to Decrypt or Encrypt by -e
    DES(Encryption),
}

#[derive(Parser)]
pub struct Encryption {
    #[clap(short, long)]
    pub encrypt: bool
}

pub fn get_args() -> Args {
    Args::parse()
}