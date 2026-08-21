use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, Context, Result};
use bincode::serialize_into;
use csv::{ReaderBuilder, Trim};
use serde::{Deserialize, Serialize};

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
    let out_dir = env::var("OUT_DIR").context("OUT_DIR environment variable not found")?;
    let mut journals = Vec::new();

    if cfg!(feature = "online") {
        let repo_dir = clone_repo()?;
        let import_order = determine_order();
        journals = process_csv_files(&repo_dir, import_order)?;

        let dest_path = Path::new(&out_dir).join("generated_journals.bin");
        write_journals_to_bincode(&journals, &dest_path)?;
    }

    if cfg!(feature = "csv") {
        if journals.is_empty() {
            // Load journals from the pre-generated binary file located in the resources directory
            let bin_path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
                .join("resources")
                .join("generated_journals.bin");
            journals = load_journals_from_bincode(&bin_path)?;
        }

        let output_filename = construct_output_filename(&out_dir, determine_order());
        write_journals_to_csv(&journals, &output_filename)?;
    }

    Ok(())
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
        .collect::<Result<Vec<_>, _>>()
        .map(|v| v.into_iter().flatten().collect())
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

fn write_journals_to_csv(journals: &[Record], output_csv_filename: &Path) -> Result<()> {
    let mut wtr = csv::Writer::from_path(output_csv_filename)?;
    for journal in journals {
        wtr.serialize(journal)?;
    }
    wtr.flush()?;
    Ok(())
}

fn write_journals_to_bincode(journals: &[Record], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    serialize_into(file, journals)?;
    Ok(())
}

fn determine_order() -> Order {
    if cfg!(feature = "dot") {
        Order::Dots
    } else {
        Order::Dotless
    }
}

fn construct_output_filename(out_dir: &str, import_order: Order) -> PathBuf {
    let filename = match import_order {
        Order::Dots => "journalList_dots.csv",
        Order::Dotless => "journalList_dotless.csv",
    };
    Path::new(out_dir).join(filename)
}
