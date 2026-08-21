#![forbid(unsafe_code)]

mod repl;

use ablescript::{interpret::ExecEnv, parser::parse};
use clap::Parser;
use std::{path::PathBuf, process::exit};

fn main() {
    // variables::test(); // NOTE(Able): Add this as a test case
    let args = Args::parse();
    match args.file {
        Some(file_path) => {
            // Read file
            let source = match std::fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(e) => {
                    println!("Failed to read file \"{:?}\": {}", file_path, e);
                    exit(1)
                }
            };

            // Parse & evaluate
            if let Err(e) = parse(&source).and_then(|ast| {
                if args.debug {
                    eprintln!("{:#?}", ast);
                }
                ExecEnv::<ablescript::host_interface::Standard>::default().eval_stmts(&ast)
            }) {
                println!(
                    "Error `{:?}` occurred at span: {:?} = `{:?}`",
                    e.kind,
                    e.span.clone(),
                    &source[e.span]
                );
            }
        }
        None => {
            println!("Hi [AbleScript {}]", env!("CARGO_PKG_VERSION"));
            repl::repl(args.debug);
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "AbleScript", version, about)]
struct Args {
    /// File to execute
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Dump AST to console
    #[arg(short, long)]
    debug: bool,
}
