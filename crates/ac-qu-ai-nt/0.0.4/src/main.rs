use std::{fs::create_dir_all, io::ErrorKind, path::PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(
        long,
        env,
        default_value_os_t = dirs_next::data_local_dir().expect("sorry but you're on a platform where dirs_next::data_local_dir() returned None, so please specify a data directory for the application").join("ac-qu-ai-nt")
    )]
    application_data_directory: PathBuf,
}

fn main() {
    let Args {
        application_data_directory,
    } = Args::parse();

    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    match create_dir_all(&application_data_directory) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => {
            panic!("{}", e);
        }
    }

    let tracing_directory = application_data_directory.join("logs");

    match create_dir_all(&tracing_directory) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
        Err(e) => {
            panic!("{}", e);
        }
    }

    #[cfg(feature = "cli-clap")]
    ac_qu_ai_nt_cli_clap::main();
}
