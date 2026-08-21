use anyhow::{Context, Result};
use clap::Parser;

// Next line is a macro :O (how cool)
#[derive(Parser)]
struct Cli {
    pattern: String,
    path: std::path::PathBuf,
}

fn find_matches(
    content: &str,
    pattern: &str,
    mut writer: impl std::io::Write,
) -> Result<(), std::io::Error> {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line)?;
        }
    }
    Ok(())
}

#[test]
fn test_find_match() -> Result<(), std::io::Error> {
    let mut result = Vec::new();
    find_matches(
        "d\njdfi\ndif\ndi\nmoose\ndif\nsd\noose\nmoose\nmoo\n",
        "moose",
        &mut result,
    )?;
    assert_eq!(result, b"moose\nmoose\n");
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let content = std::fs::read_to_string(&args.path)
        .with_context(|| format!("Could not read file `{}`", args.path.display()))?;

    find_matches(&content, &args.pattern, &mut std::io::stdout())?;
    Ok(())
}
