//! Print adamo fabric time vs the local wall clock so you can see the
//! sync converge and the offset between this node and the network.
//!
//!   ADAMO_API_KEY=<key> cargo run --example fabric_time

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use adamo::{Protocol, Session};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let _session = Session::open(&api_key, Protocol::Quic)?;

    println!("waiting for time-sync handshake…");
    for _ in 0..20 {
        if adamo::fabric_synced() {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    if !adamo::fabric_synced() {
        eprintln!("WARN: never synced; values below are local-only");
    }

    println!("{:>12}  {:>12}  {:>10}", "fabric (ms)", "local (ms)", "offset (ms)");
    for _ in 0..10 {
        let local = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i128;
        let fabric = adamo::fabric_now_us() as i128 / 1000;
        println!(
            "{:>12}  {:>12}  {:>+10}",
            fabric,
            local,
            fabric - local
        );
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}
