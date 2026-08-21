use std::process::{Command, Output};
use std::io::{self, Result};

pub fn ping(ip: &String) -> Result<Output> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "ping", "-n", "2", ip])
            .output()?
    } else {
        Command::new("ping")
            .args(&["-c", "2", ip])
            .output()?
    };

    let output_str = String::from_utf8_lossy(&output.stdout);

    if output.status.success() && output_str.contains("TTL=") {
        Ok(output)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Dispostivo não está pingando!"))
    }
}