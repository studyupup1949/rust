//  SPDX-FileCopyrightText: 2025 Greg Heartsfield <scsibug@imap.cc>
//  SPDX-License-Identifier: GPL-3.0-or-later

//! Command line arguments.
use clap::Parser;

#[derive(Parser)]
#[command(about = "Convert ALFA to XACML.", author = env!("CARGO_PKG_AUTHORS"), version = env!("CARGO_PKG_VERSION"), arg_required_else_help = true)]
pub struct CLIArgs {
    #[arg(
        short = 'i',
        long = "input",
        help = "Read ALFA from <files> or <directories>",
        required = false
    )]
    pub alfa_dir: Vec<String>,
    #[arg(
        long = "show-builtins",
        help = "Show ALFA built-in definitions",
        required = false
    )]
    pub show_builtins: bool,
    #[arg(
        short = 'd',
        long = "disable-builtins",
        help = "Disable ALFA built-in definitions (enabled by default)",
        default_value_t = false,
        required = false
    )]
    pub disable_builtins: bool,
    #[arg(
        short = 'o',
        long = "output",
        help = "Write XACML files to <directory>",
        required = false
    )]
    pub output_dir: Option<String>,
    #[arg(
        short = 'n',
        long = "namepace",
        help = "URI prefix for policies",
        required = false
    )]
    pub base_namespace: Option<String>,
}
