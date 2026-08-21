use clap::Parser;
use sha2::{Sha256, Digest};
use bs58;

#[derive(Parser)]
struct Cli {
    /// The account name in PascalCase (e.g. `StakeEscrow`)
    account_name: String,
}

fn main() {
    let args = Cli::parse();

    // Create the string to hash
    let to_hash = format!("account:{}", args.account_name);

    // Compute the hash
    let result = Sha256::digest(to_hash.as_bytes());
    
    // Return the first 8 bytes in base58
    let base58_string = bs58::encode(&result[..8]).into_string();
    
    println!("{}", base58_string);
}
