use clap::{Parser, Subcommand};

mod utility;
use utility::{
    CliError, PairService, PortMapping, adb_connect_device, adb_ensure_running, adb_reverse_port,
};

#[derive(Parser)]
#[command(
    name = "adb-wireless",
    version = "1.0",
    about = "CLI tool to generate QR code for adb wireless connection"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate QR code for adb wireless connection")]
    Pair,
    #[command(about = "Map TCP ports from device to host")]
    Reverse {
        #[arg(
            help = "Port mappings in the format <device_port>:<host_port>",
            required = true,
            value_name = "PORT:PORT"
        )]
        ports: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = run_cli(cli) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run_cli(cli: Cli) -> Result<(), CliError> {
    adb_ensure_running()?;

    match cli.command {
        Commands::Reverse { ports } => {
            // Handle reverse port mapping
            for port in ports {
                let mapping = PortMapping::new(&port)?;
                adb_reverse_port(&mapping)?;
                println!(
                    "Reversed port {}:{}",
                    mapping.device_port, mapping.host_port
                );
            }
        }
        Commands::Pair => {
            let service = PairService::new()?;
            service.start_discovery()?;

            qr2term::print_qr(service.qrtext())?;
            println!("QR code generated. Scan it with your device to pair.");

            let device = service.wait_for_pairing()?;
            println!(
                "Device found at {}:{}",
                device.address, device.debugging_port
            );
            adb_connect_device(&device, &service.password)?;

            println!("Device connected");
        }
    }

    Ok(())
}
