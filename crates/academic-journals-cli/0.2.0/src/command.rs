use clap::{Parser, ValueEnum};

/// Command-line interface for a journal lookup application.
///
/// This struct defines the command-line arguments accepted by the application.
/// It allows users to either find the abbreviation of a journal from its full
/// name or find the full name from an abbreviation.
///
/// # Usage
/// - To find an abbreviation: `academic-journals --abbreviation "Journal of
///   Rust Studies"`
/// - To find a full name: `academic-journals --full-name "JRS"`
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Specifies the type of input provided: a direct string or a path to a
    /// file.
    #[arg(value_enum, required = true)]
    pub input_type: InputType,

    /// The input string or file path, depending on the input type.
    #[arg(required = true)]
    pub input: String,

    /// If set, the application will find the abbreviation for the journal's
    /// full name.
    #[arg(short, long)]
    pub abbreviation: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum InputType {
    /// A literal journal name or abbreviation string.
    String,
    /// A path to a file containing one journal name or abbreviation per line.
    File,
}
