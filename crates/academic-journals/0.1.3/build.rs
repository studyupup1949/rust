use anyhow::{anyhow, Context, Result};
use bincode::serialize_into;
use csv::{ReaderBuilder, Trim};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Record {
    full_name: String,
    #[serde(default)]
    abbreviation_1: Option<String>,
    #[serde(default)]
    abbreviation_2: Option<String>,
    #[serde(default)]
    abbreviation_3: Option<String>,
}

#[derive(Clone, Copy)]
enum Order {
    Dots,
    Dotless,
}

impl Order {
    fn file_suffixes(&self) -> &[&str] {
        match self {
            Order::Dots => &[
                "acs",
                "ams",
                "general",
                "geology_physics",
                "ieee",
                "lifescience",
                "mathematics",
                "mechanical",
                "meteorology",
                "sociology",
                "webofscience-dots",
            ],
            Order::Dotless => &["entrez", "medicus", "webofscience-dotless"],
        }
    }
}

fn main() -> Result<()> {
    let out_dir = out_dir()?;
    let import_order = determine_order();

    let journals = if cfg!(feature = "online") {
        let repo_dir = clone_repo()?;
        process_csv_files(&repo_dir, import_order)?
    } else {
        load_journals_from_local()?
    };

    let dest_path = Path::new(&out_dir).join("generated_journals.bin");
    write_journals_to_bincode(&journals, &dest_path)?;

    Ok(())
}

fn out_dir() -> Result<PathBuf> {
    env::var("OUT_DIR")
        .context("OUT_DIR environment variable not found")
        .map(PathBuf::from)
}

fn determine_order() -> Order {
    if cfg!(feature = "dot") {
        Order::Dots
    } else {
        Order::Dotless
    }
}

fn load_journals_from_bincode(bin_path: &Path) -> Result<Vec<Record>> {
    let file = File::open(bin_path).context("Failed to open the binary file")?;
    bincode::deserialize_from(file)
        .context("Failed to deserialize the journals from the binary file")
}

fn read_csv(file_path: &Path) -> Result<Vec<Record>> {
    ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(Trim::All)
        .from_path(file_path)
        .with_context(|| format!("Failed to open CSV file {:?}", file_path))?
        .deserialize()
        .collect::<Result<Vec<Record>, _>>()
        .context("Failed to read and deserialize CSV records")
}

fn load_journals_from_local() -> Result<Vec<Record>> {
    let bin_path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("resources")
        .join("generated_journals.bin");
    if bin_path.exists() {
        load_journals_from_bincode(&bin_path)
    } else {
        Err(anyhow!("Local journal data not found"))
    }
}

fn clone_repo() -> Result<PathBuf> {
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable not found")?;
    let repo_dir = Path::new(&out_dir).join("abbrv.jabref.org");
    if !repo_dir.exists() {
        Command::new("git")
            .args([
                "clone",
                "https://github.com/JabRef/abbrv.jabref.org.git",
                repo_dir
                    .to_str()
                    .ok_or_else(|| anyhow!("Failed to convert repo path to string"))?,
            ])
            .status()
            .context("Failed to clone the repository")?;
    }
    Ok(repo_dir)
}

fn process_csv_files(repo_dir: &Path, import_order: Order) -> Result<Vec<Record>> {
    let journals_path = repo_dir.join("journals");
    import_order
        .file_suffixes()
        .iter()
        .map(|suffix| {
            let file_path = journals_path.join(format!("journal_abbreviations_{}.csv", suffix));
            read_csv(&file_path)
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map(|v| v.into_iter().flatten().collect())
}

fn write_journals_to_bincode(journals: &[Record], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    serialize_into(file, journals)?;
    Ok(())
}
