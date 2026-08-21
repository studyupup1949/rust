use add_custom_device_to_pcsclite::{DEFAULT_NAME, DEFAULT_PID, DEFAULT_VID, run_check, run_add};
use clap;
use clap::{Parser, Subcommand, arg, CommandFactory};
use std::process;

const HELP_TEMPLATE: &'static str = "\
{before-help}
{name} {version} by {author}

{usage-heading} {usage}

{all-args}

{after-help}
";

#[derive(Parser)]
#[command(
    version,
    about = "This tool adds your unofficial VID:PID entry to pcsclite's Info.plist file.",
    author = "Орест Смертний (foresle)",
    help_template = HELP_TEMPLATE,
)]
struct Cli {
    /// Vendor ID.
    #[arg(long, default_value_t = String::from(DEFAULT_VID))]
    vid: String,
    /// Product ID.
    #[arg(long, default_value_t = String::from(DEFAULT_PID))]
    pid: String,
    /// Product name.
    #[arg(long, default_value_t = String::from(DEFAULT_NAME))]
    name: String,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if VID:PID entry already exist in file.
    Check,
    /// Add VID:PID entry to the file.
    Add {
        /// Check is VID:PID values already exist. If that exists, don't do anything.
        #[arg(long, default_value_t = false)]
        check_existing: bool,
    }
}

fn main() {
    let cli: Cli = Cli::parse();

    match &cli.command {
        Some(Commands::Check) => {
            if let Err(error) = run_check(&cli.vid, &cli.pid) {
                eprintln!("Critical error: \"{error}\".");
                process::exit(1);
            }
        }
        Some(Commands::Add { check_existing }) => {
            if let Err(error) = run_add(&cli.vid, &cli.pid, &cli.name, check_existing) {
                eprintln!("Critical error: \"{error}\".");
                process::exit(1);
            };
        }
        None => {
            Cli::command().print_help().unwrap();
            process::exit(0);
        }
    }

    println!("All operations done successfully. Exiting!");
    process::exit(0);
}
