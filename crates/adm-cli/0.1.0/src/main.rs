use structopt::StructOpt;

use adm::config::CONFIG;
mod error;
use self::error::TurnError;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    #[structopt(raw(setting = "structopt::clap::AppSettings::DisableVersion"))]
    /// Manage device power states from the command line.
    Turn {
        /// The device to modify.
        device: String,
        /// The desired state of the device (on or off).
        state: String,
        /// Send requests quickly, without waiting for confirmation.
        #[structopt(short, long)]
        fast: bool,
    },
}

fn parse_state<S: ToString>(s: &S) -> Result<bool, TurnError> {
    let lower = s.to_string().to_ascii_lowercase();
    match lower.as_str() {
        "on" | "1" => Ok(true),
        "off" | "0" => Ok(false),
        _ => Err(TurnError::UnrecognizedState(lower)),
    }
}

fn main() -> Result<(), TurnError> {
    let Command::Turn {
        device,
        state,
        fast,
    } = Command::from_args();
    if let Some(device) = CONFIG.find(&device) {
        let target = parse_state(&state)?;
        device.power(target, fast)?;
        Ok(())
    } else {
        Err(TurnError::DeviceNotFound(device))
    }
}
