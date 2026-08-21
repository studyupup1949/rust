use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

fn copy_dir_recursive(from: &Path, to: &Path) -> io::Result<()> {
    if !to.exists() {
        fs::create_dir_all(to)?;
    }

    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = to.join(entry.file_name());
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if metadata.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps"])
        .status()?;

    if !status.success() {
        return Err("cargo doc failed".into());
    }

    let source = Path::new("target/doc");
    let destination = Path::new("../public");

    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }
    fs::create_dir_all(destination)?;
    copy_dir_recursive(source, destination)?;

    println!("Docs staged for publish in ../public");
    Ok(())
}