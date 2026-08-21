use act2pal::*;
use clap::Parser;
use std::{fs, path::PathBuf, process::exit};

#[derive(Debug, Parser)]
struct Command {
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(short = 'l', long, value_name = "NUMBER")]
    assert_len: Option<usize>,
}

fn main() {
    let Command {
        input,
        output,
        assert_len,
    } = Command::parse();
    let bytes = match fs::read(&input) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("{error} reading {input:?}.");
            exit(1);
        }
    };
    let colors = match Colors::from_act(&bytes) {
        Ok(colors) => colors,
        Err(error) => {
            eprintln!("Failed to process ACT palette: {error}.");
            exit(1);
        }
    };
    if let Some(assert_len) = assert_len {
        if assert_len != colors.len() {
            eprintln!("Got {} colors, expected {assert_len}.", colors.len());
            exit(1);
        }
    }
    let pal = match colors.to_pal_string() {
        Ok(pal) => pal,
        Err(error) => {
            eprintln!("{error} generating PAL string.");
            exit(1);
        }
    };
    match output {
        Some(output) => {
            if let Err(error) = fs::write(&output, pal) {
                eprintln!("{error} writing PAL file.");
                exit(1);
            }
        }
        None => {
            print!("{pal}")
        }
    }
}
