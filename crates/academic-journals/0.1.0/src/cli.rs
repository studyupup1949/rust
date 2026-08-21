use clap::Parser;

/// Finds abbreviations and full names of journals
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The full name or abbreviation of the journal
    pub input: String,

    #[arg(short, long)]
    /// Look up the abbreviation for the given full name
    pub abbreviation: bool,
}
