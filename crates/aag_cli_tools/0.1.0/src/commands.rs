use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};

pub fn grep(pattern: &str, filename: &str) -> io::Result<()> {
    if let Ok(file) = File::open(filename) {
        for line in io::BufReader::new(file).lines() {
            let line = line?;
            if line.contains(pattern) {
                println!("{}", line);
            }
        }
    } else {
        eprintln!("Failed to open file: {}", filename);
    }
    Ok(())
}

pub fn cat(filenames: &[String], output_filename: Option<&str>) -> io::Result<()> {
    let mut output: Box<dyn Write> = match output_filename {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    for filename in filenames {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        output.write_all(content.as_bytes())?;
    }
    Ok(())
}

pub fn ls(path: &str) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        println!("{}", entry.file_name().to_string_lossy());
    }
    Ok(())
}

pub fn diff(file1: &str, file2: &str) -> io::Result<()> {
    let file1 = File::open(file1)?;
    let file2 = File::open(file2)?;
    
    let lines1: Vec<_> = io::BufReader::new(file1).lines().collect::<Result<_, _>>()?;
    let lines2: Vec<_> = io::BufReader::new(file2).lines().collect::<Result<_, _>>()?;

    for (i, (line1, line2)) in lines1.iter().zip(&lines2).enumerate() {
        if line1 != line2 {
            println!("Line {}:\n- {}\n+ {}", i + 1, line1, line2);
        }
    }
    Ok(())
}

pub fn time() -> String {
    let now = chrono::Local::now();
    now.format("%H:%M:%S").to_string()
}

pub fn date() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d").to_string()
}
