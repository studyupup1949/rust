use std::net::SocketAddr;

use super::{error::CliError, port::PortMapping};

pub fn adb_ensure_running() -> Result<(), CliError> {
    // Check if ADB exists
    match std::process::Command::new("adb").arg("version").output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbNotFound);
            }
        }
        Err(_) => return Err(CliError::AdbNotFound),
    }

    // Start ADB server
    match std::process::Command::new("adb")
        .arg("start-server")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to start ADB server. {}",
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}

pub fn adb_reverse_port(mapping: &PortMapping) -> Result<(), CliError> {
    match std::process::Command::new("adb")
        .arg("reverse")
        .arg(format!("tcp:{}", mapping.device_port))
        .arg(format!("tcp:{}", mapping.host_port))
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to reverse port {}:{}. {}",
                        mapping.device_port,
                        mapping.host_port,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}

pub fn adb_connect_device(address: &SocketAddr, password: &str) -> Result<(), CliError> {
    match std::process::Command::new("adb")
        .arg("pair")
        .arg(format!("{}:{}", address.ip(), address.port()))
        .arg(password)
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(CliError::AdbServerError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Failed to connect to device at {}. {}",
                        address,
                        String::from_utf8_lossy(&output.stderr)
                    ),
                )));
            }
        }
        Err(err) => return Err(CliError::AdbServerError(err)),
    }

    Ok(())
}
