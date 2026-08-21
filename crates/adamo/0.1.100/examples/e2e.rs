//! End-to-end smoke test for the Rust SDK.
//!
//! Opens a session, declares a subscriber, publishes a payload on the
//! same key, and waits for the round-trip through the Adamo network.
//! Prints RTT and asserts the payload matches.
//!
//! Run with:
//!   ADAMO_API_KEY=<key> cargo run --example e2e

use std::time::{Duration, Instant};

use adamo::{Protocol, PublishOptions, Session};

const KEY: &str = "test/rust-sdk-e2e";

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");

    println!("opening session (quic)…");
    let t0 = Instant::now();
    let session = Session::open(&api_key, Protocol::Quic)?;
    println!(
        "  connected to org `{}` in {:?}",
        session.org()?,
        t0.elapsed()
    );

    println!("declaring subscriber on `{KEY}`…");
    let sub = session.subscribe(KEY)?;

    // Let the subscription propagate through the fabric before publishing.
    std::thread::sleep(Duration::from_millis(250));

    let payload = format!("hello @ {}ms", now_ms()).into_bytes();
    println!("publishing {} bytes…", payload.len());
    let sent = Instant::now();
    session.put(
        KEY,
        &payload,
        PublishOptions {
            priority: 200,
            express: true,
        },
    )?;

    println!("waiting for round-trip (timeout 5s)…");
    let sample = sub.recv(Some(Duration::from_secs(5)))?;
    let rtt = sent.elapsed();

    assert_eq!(sample.payload, payload, "round-trip payload mismatch");
    println!(
        "  received {} bytes on `{}` — rtt {:?}",
        sample.payload.len(),
        sample.key,
        rtt
    );

    println!("OK");
    Ok(())
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
