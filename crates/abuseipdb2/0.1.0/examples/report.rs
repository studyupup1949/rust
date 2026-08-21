use std::{net::IpAddr, str::FromStr, vec};

use abuseipdb2::{types::Category, Client};

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("ABUSEIPDB_KEY").unwrap());

    let report = client
        .report(
            IpAddr::from_str("127.0.0.1").unwrap(),
            vec![Category::OpenProxy],
            None,
            None,
        )
        .await
        .unwrap();

    println!("{:?}", report);
}
