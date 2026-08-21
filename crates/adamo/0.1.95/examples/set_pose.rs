use std::time::Duration;

use adamo::{Protocol, Session};

fn main() -> adamo::Result<()> {
    let api_key = std::env::var("ADAMO_API_KEY").expect("set ADAMO_API_KEY");
    let session = Session::open(&api_key, Protocol::Quic)?;
    println!("connected to org: {}", session.org()?);

    // Feed set_pose from your localization loop. Calls beyond 5 Hz are
    // dropped by the SDK, so publishing every odometry tick is fine.
    let mut x = 0.0;
    loop {
        session.set_pose("demo-bot", x, 2.0, 0.0, "map", Some(1.57))?;
        x += 0.05;
        std::thread::sleep(Duration::from_millis(50));
    }
}
