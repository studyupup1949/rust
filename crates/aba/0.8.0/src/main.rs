// SPDX-FileCopyrightText: 2023 Gustavo Coutinho de Souza <dev@onemoresuza.mailer.me>
//
// SPDX-License-Identifier: ISC

use std::error::Error;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod address_book;

use crate::address_book::AddressBook;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "An address book for aerc")]
struct Cli {
    #[arg(short, long, value_name = "PATH")]
    #[arg(help = "Set the address book path")]
    address_book: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = "Add a new address to the address book")]
    Add {
        name: String,
        email: String,
        #[arg(short = 'F', long)]
        #[arg(help = "Do not throw an error when an address already exists")]
        force: bool,
    },
    #[command(about = "Delete an address of the address book")]
    Del {
        #[arg(value_name = "MATCH")]
        match_str: String,
        #[arg(short = 'r', long)]
        #[arg(help = "Treat MATCH as a regular expression")]
        is_regex: bool,
    },
    #[command(about = "List the fuzzy matches")]
    #[command(alias = "ls")]
    List {
        fuzzy: String,
        #[arg(short, long, value_name = "N")]
        #[arg(help = "Show at most N matches")]
        #[arg(default_value = "5")]
        number: usize,
    },
    #[command(about = "Parse a raw email message to get the addresses")]
    Parse {
        filename: Option<PathBuf>,
        #[arg(short = 'F', long)]
        #[arg(help = "Do not throw an error when an address already exists")]
        force: bool,
        #[arg(short = None, long)]
        #[arg(help = "Parse the addresses from the `From' header (default)")]
        from: bool,
        #[arg(short = None, long)]
        #[arg(help = "Parse the addresses from the `Cc' header")]
        cc: bool,
        #[arg(short = None, long)]
        #[arg(help = "Parse the addresses from the `To' header")]
        to: bool,
        #[arg(short = None, long)]
        #[arg(help = "Parse the addresses from the `From', `Cc' and `To' headers")]
        all: bool,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut addbook = AddressBook::new(cli.address_book)?;

    match &cli.cmd {
        Cmd::Add { name, email, force } => addbook.add(name, email, force.to_owned()),
        Cmd::Del {
            match_str,
            is_regex,
        } => addbook.del(match_str, is_regex.to_owned()),
        Cmd::List { fuzzy, number } => addbook.list(fuzzy, number.to_owned()),
        Cmd::Parse {
            filename,
            force,
            from,
            cc,
            to,
            all,
        } => addbook.parse(
            filename,
            force.to_owned(),
            from.to_owned(),
            cc.to_owned(),
            to.to_owned(),
            all.to_owned(),
        ),
    }
}
