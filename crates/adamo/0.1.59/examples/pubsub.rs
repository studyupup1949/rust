use std::time::Duration;

use adamo::{Protocol, PublishOptions, Session};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let session = Session::open(&api_key, Protocol::Quic)?;
    println!("connected to org: {}", session.org()?);

    let sub = session.subscribe("demo/rs")?;
    session.put("demo/rs", b"hello from rust", PublishOptions::default())?;

    let sample = sub.recv(Some(Duration::from_secs(5)))?;
    println!("got {} bytes on {}", sample.payload.len(), sample.key);
    Ok(())
}
