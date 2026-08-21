use clap::Parser;
use std::path::PathBuf;
use std::io::{self, BufReader, BufRead};
use std::fs::File;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    let f = File::open(args.path)?;

    let mut reader = BufReader::new(f);
    let mut line = String::new();

    loop {
        let lb = reader.read_line(&mut line)?;
        
        if lb == 0{
            break;
        }
        else if line.trim().contains(&args.pattern){
            println!("{}",line.trim());
        }
    line.clear();
    }
    Ok(())

}
