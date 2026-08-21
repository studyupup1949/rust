use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Write};
use std::{env, io};

fn main() -> Result<(), anyhow::Error> {
    let usage = "Usage: anew <FILE>\n\nFlags:\n-h/--help: Print help";

    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return Err(anyhow::anyhow!("invalid arguments.\n\n{}", usage));
    }

    let input_file_name = &args[1];

    if input_file_name.eq_ignore_ascii_case("-h") || input_file_name.eq_ignore_ascii_case("--help") {
        println!("{}", usage);
        return Ok(());
    }

    let mut file_handle = File::options()
        .append(true)
        .create(true)
        .read(true)
        .open(input_file_name)?;

    let mut file_lines = parse_file_lines(&file_handle);

    let mut in_line = String::new();

    match io::stdin().read_line(&mut in_line) {
        Ok(_) => {
            in_line = in_line.trim().to_string();

            if file_lines.insert(in_line.clone()) {
                _ = file_handle.write_all(in_line.as_bytes())?;
                println!("{}", in_line);
            }
        }
        Err(error) => println!("failed to read line: {}", error),
    }

    Ok(())
}

fn parse_file_lines(mut f: &File) -> BTreeSet<String> {
    let mut file_content = String::new();
    _ = f.read_to_string(&mut file_content);

    BTreeSet::from_iter(
        file_content
            .lines()
            .map(String::from)
            .map(|l| l.trim().to_string()),
    )
}
