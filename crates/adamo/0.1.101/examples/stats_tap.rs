//! Subscribe to a robot's latency stats and print the raw JSON payload.
//! Used to verify the wire format matches what the frontend expects.
//!
//!   ADAMO_API_KEY=<key> cargo run --example stats_tap

use std::time::Duration;

use adamo::{Protocol, Session};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let robot = std::env::var("ADAMO_ROBOT_NAME").unwrap_or_else(|_| "macbook".into());

    let session = Session::open(&api_key, Protocol::Quic)?;
    let org = session.org()?;
    let key = format!("adamo/{org}/{robot}/stats/latency");
    println!("subscribing to {key}");

    let sub = session.subscribe(&key)?;
    for i in 0..3 {
        let s = sub.recv(Some(Duration::from_secs(5)))?;
        println!(
            "[{i}] key={} is_delete={} payload={}",
            s.key,
            s.is_delete,
            String::from_utf8_lossy(&s.payload)
        );
    }
    Ok(())
}
