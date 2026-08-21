use walkdir::WalkDir;
use clap::Parser;

//beggining section is from the textbook example on making grep
// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    // The pattern to look for
    pattern: String,
    // The path to the file to start from
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let target_dir = &args.path;
    let mut found = false;

    //for loop is from WalkDir tutorial. Recursively moves through each directory
    for entry in WalkDir::new(target_dir).into_iter().filter_map(|e| e.ok()) {
        let mut dir_path = entry.path().display().to_string(); //This gets my path as a string
        if dir_path.contains(&args.pattern) && !found{ //check to see if it has the pattern
            if entry.file_type().is_file() { //new to jmp, because we only want a directory, checks for files
                if let Some(parent) = entry.path().parent(){ //changes the file to its parent directory
                    dir_path = parent.display().to_string();
                }
            }
            println!("{}", dir_path);
            found = true; //sets found so we only get the first (top level) folder and not all the subdirectories and files
        }
    }
    if !found{
        println!("Nothing found with the name {} under the given directory {}", &args.pattern, &args.path.display())
    }
}
