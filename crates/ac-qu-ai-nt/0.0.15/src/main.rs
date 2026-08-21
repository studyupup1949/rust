use std::{
    fs::create_dir_all,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use cfg_if::cfg_if;
use clap::{Parser, Subcommand};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Parser)]
struct Args {
    #[arg(
        long,
        env,
        default_value_os_t = dirs_next::data_local_dir().expect("sorry but you're on a platform where dirs_next::data_local_dir() returned None, so please specify a data directory for the application").join("ac-qu-ai-nt")
    )]
    application_data_directory: PathBuf,

    // If there is a GUI or TUI available from this binary,
    // then calling this program without arguments
    // is acceptable: it will launch a user interface
    #[cfg(any(feature = "gui-eframe", feature = "tui-ratatui"))]
    #[command(subcommand)]
    command: Option<Command>,

    #[cfg(not(any(feature = "gui-eframe", feature = "tui-ratatui")))]
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[cfg(feature = "gui-eframe")]
    #[command(alias = "gui")]
    GuiEframe,

    #[cfg(feature = "tui-ratatui")]
    #[command(alias = "tui")]
    TuiRatatui,

    #[cfg(feature = "cli-clap")]
    #[command(alias = "cli")]
    CliClap(ac_qu_ai_nt_cli_clap::Args),
}

// When this program is run without arguments,
// it will launch a user interface
cfg_if!(
    // with the GUI (made with eframe)
    // being considered more appealing (made the default)
    if #[cfg(feature = "gui-eframe")] {
        #[allow(clippy::derivable_impls)]
        impl Default for Command {
            fn default() -> Self {
                Command::GuiEframe
            }
        }
    }
    // than the TUI (made with Ratatui)
    else if #[cfg(feature = "tui-ratatui")] {
        #[allow(clippy::derivable_impls)]
        impl Default for Command {
            fn default() -> Self {
                Command::TuiRatatui
            }
        }
    }
    // with it not being logical to specify
    // the CLI (made with clap)
    // as an option,
    // because if `ac-qu-ai-nt ask "why is the sky blue?"`
    // were accepted and worked when `cli-clap` was the only
    // interface enabled, then when another interface
    // like `gui-eframe` were enabled, it would stop working
    // (only able to work as `ac-qu-ai-nt cli-clap ask "why is the sky blue?"`)
    // so it should be required to do it the way that would work
    // in both cases from the beginning
);

#[derive(Debug, Snafu)]
enum ApplicationError {
    #[snafu(display("failed to create the {path:?} directory"))]
    DirectoryCreationError {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Create the directory and its parents,
/// but don't return an error if it already exists and is a directory
fn create_dir_all_exist_ok(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    match create_dir_all(&path) {
        Ok(()) => Ok(()),
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists && path.as_ref().is_dir() {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

#[snafu::report]
fn main() -> Result<(), ApplicationError> {
    let Args {
        application_data_directory,
        command,
    } = Args::parse();

    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    create_dir_all_exist_ok(&application_data_directory).with_context(|_| {
        DirectoryCreationSnafu {
            path: application_data_directory.clone(),
        }
    })?;

    let tracing_directory = application_data_directory.join("logs");
    create_dir_all_exist_ok(&tracing_directory).with_context(|_| DirectoryCreationSnafu {
        path: tracing_directory.clone(),
    })?;

    #[cfg(any(feature = "gui-eframe", feature = "tui-ratatui"))]
    let command = command.unwrap_or_default();

    match command {
        #[cfg(feature = "cli-clap")]
        Command::CliClap(args) => ac_qu_ai_nt_cli_clap::main(args),
        #[cfg(feature = "gui-eframe")]
        Command::GuiEframe => ac_qu_ai_nt_gui_eframe::main(),
        #[cfg(feature = "tui-ratatui")]
        Command::TuiRatatui => ac_qu_ai_nt_tui_ratatui::main(),
    }

    Ok(())
}
