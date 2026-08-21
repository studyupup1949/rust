use std::{net::IpAddr, str::FromStr};

use abuseipdb2::Client;

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("ABUSEIPDB_KEY").unwrap());

    let check = client
        .check(IpAddr::from_str("127.0.0.1").unwrap(), None, Some(true))
        .await
        .unwrap();

    println!("{:?}", check);
}
