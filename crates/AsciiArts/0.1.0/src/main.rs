use std;
use clap::Parser;

#[derive(Parser)]
struct Arguments {
    pattern: String,
    path: std::path::PathBuf,
}

fn main(){
    let args = Arguments::parse();

    let file_content = std::fs::read_to_string(&args.path);

    match file_content {
        Ok(content) => {
            println!("Searching for {} in {:?}", args.pattern, args.path);

            let mut instances = 0;
            let mut important_lines: Vec<String> = Vec::new();

            for line in content.lines() {
                if line.contains(&args.pattern) {
                    instances += 1;
                    important_lines.push(line.to_string());
                }
            }

            println!("Found {} instances of {:?} in the following lines", instances, args.pattern);

            for (line_number, line) in content.lines().enumerate() {
                // `enumerate()` gives zero-based index, so we add 1 for 1-based line numbers
                print!("{}: {}   ", line_number + 1, line);
            }
        }
        Err(e) => {
            println!("Error: could not read the file. Error: {}", e);
        }
    }
}
