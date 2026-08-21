#![deny(clippy::all, clippy::pedantic)]

#[cfg(windows)]
mod windows;

#[cfg(windows)]
fn main() -> windows::error::Result<()> {
    use crate::windows::{cli, parse_attributes};
    use clap::Parser;

    let cli = cli::Cli::parse();

    let value = if cli.decimal {
        cli.value.as_str().parse::<u32>()?
    } else {
        u32::from_str_radix(&cli.value, 16)?
    };

    parse_attributes(value, cli.r#type);

    Ok(())
}

#[cfg(not(windows))]
fn main() {
    compile_error!("This crate is only supported on Windows");
}
