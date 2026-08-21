use clap::Parser;

#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let data = std::fs::read_to_string(&args.path).expect("Unable to read.");

    for l in data.lines(){
        if l.contains(&args.pattern){
            println!("{}",l);
        }
    }
}
