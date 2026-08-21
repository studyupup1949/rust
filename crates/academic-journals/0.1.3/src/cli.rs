use clap::Parser;

/// Command-line interface for a journal lookup application.
///
/// This struct defines the command-line arguments accepted by the application.
/// It allows users to either find the abbreviation of a journal from its full name
/// or find the full name from an abbreviation.
///
/// # Usage
/// - To find an abbreviation: `academic-journals --abbreviation "Journal of Rust Studies"`
/// - To find a full name: `academic-journals "JRS"`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Either the full name or abbreviation of a journal.
    pub input: String,

    /// If set, the application will find the abbreviation for the journal's full name.
    #[arg(short, long)]
    pub abbreviation: bool,
}
