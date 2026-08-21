use clap::{ArgGroup, Parser};
use base64::{engine::general_purpose, Engine};
use std::fs;
use std::io::{self, Read};

/// A simple Base64 decoder
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(group(
    ArgGroup::new("input_source")
        .required(false)
        .args(&["input", "file"]),
))]
struct Args {
    /// Base64 string to decode
    #[arg()]
    input: Option<String>,

    #[arg(short, long)]
    file: Option<String>,

    #[arg(short = 'u', long = "base64url")]
    base64url: bool,

    #[arg(short = 'p', long = "no-pad")]
    no_pad: bool,
}

fn main() {
    let args = Args::parse();

    let input = if let Some(path) = args.file {
        match fs::read_to_string(&path) {
            Ok(content) => content.trim().to_string(),
            Err(e) => {
                eprintln!("Failed to read file '{path}': {e}");
                std::process::exit(1);
            }
        }
    } else if let Some(args_input) = args.input {
        args_input
    } else {
        let mut buffer = String::new();
        if io::stdin().read_to_string(&mut buffer).is_err() {
            eprintln!("Failed to read from stdin");
            std::process::exit(1);
        }
        buffer.trim().to_string()
    };

    let engine = if args.base64url {
        if args.no_pad {
            &general_purpose::URL_SAFE_NO_PAD
        } else {
            &general_purpose::URL_SAFE
        }
    } else {
        if args.no_pad {
            &general_purpose::STANDARD_NO_PAD
        } else {
            &general_purpose::STANDARD
        }
    };

    match engine.decode(&input) {
        Ok(decoded) => match String::from_utf8(decoded) {
            Ok(s) => println!("{}", s),
            Err(_) => {
                eprintln!("Decoded data is not valid UTF-8");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to decode Base64: {e}");
            std::process::exit(1);
        }
    }
}
