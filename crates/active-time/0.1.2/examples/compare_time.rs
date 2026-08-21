use std::{time::{Duration, Instant}, thread::sleep};

use active_time::active_time;

fn main() -> std::io::Result<()> {
    println!("Start");

    let before_active = active_time()?;
    let before = Instant::now();
    sleep(Duration::from_secs(100)); // put your pc to sleep mode here
    let after_active = active_time()?;
    println!("Active time: {:?}", after_active - before_active);
    println!("Actual time: {:?}", before.elapsed());
    Ok(())
}
