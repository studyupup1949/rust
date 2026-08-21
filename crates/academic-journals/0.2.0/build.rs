//! Build script for the `academic-journals` crate.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[cfg(feature = "online")]
use crate::online::process_online_and_encode;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR")
        .map(PathBuf::from)
        .context("OUT_DIR environment variable not found")?;

    let dest_path = out_dir.join("generated_journals.bin");
    gather_and_write(&out_dir, &dest_path)
}

#[cfg(feature = "online")]
fn gather_and_write(out_dir: &Path, dest_path: &Path) -> Result<()> {
    process_online_and_encode(out_dir, dest_path)
}

#[cfg(not(feature = "online"))]
fn gather_and_write(_out_dir: &Path, dest_path: &Path) -> Result<()> {
    let bin_path = Path::new(&env::var("CARGO_MANIFEST_DIR")?)
        .join("resources")
        .join("generated_journals.bin");

    println!("cargo:rerun-if-changed=resources/generated_journals.bin");

    std::fs::copy(&bin_path, dest_path).with_context(|| {
        format!(
            "Failed to copy offline binary from {}. Run with `--features online` to download and \
             generate it.",
            bin_path.display()
        )
    })?;
    Ok(())
}

#[cfg(feature = "online")]
mod online {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use anyhow::{Context, Result};
    use csv::{ReaderBuilder, Trim};
    use rkyv::{Archive, Serialize};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Archive, Serialize, Clone)]
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
        const DOTLESS_SUFFIXES: [&'static str; 2] = ["entrez", "medicus"];
        const DOTS_SUFFIXES: [&'static str; 10] = [
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
        ];

        const fn file_suffixes(self) -> &'static [&'static str] {
            match self {
                Self::Dots => &Self::DOTS_SUFFIXES,
                Self::Dotless => &Self::DOTLESS_SUFFIXES,
            }
        }
    }

    pub fn process_online_and_encode(out_dir: &Path, dest_path: &Path) -> Result<()> {
        let order = determine_order();
        let repo_dir = clone_repo(out_dir)?;
        let records = process_csv_files(&repo_dir, order)?;
        encode_journals(records, dest_path)
    }

    // rkyv::to_bytes requires Sized; [Record] (unsized) does not implement Serialize.
    #[allow(clippy::needless_pass_by_value)]
    fn encode_journals(journals: Vec<Record>, output_path: &Path) -> Result<()> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&journals)
            .context("Failed to serialize journals")?;
        std::fs::write(output_path, &*bytes)?;
        Ok(())
    }

    fn read_csv(file_path: &Path) -> Result<Vec<Record>> {
        ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .trim(Trim::All)
            .from_path(file_path)
            .with_context(|| format!("Failed to open CSV file {}", file_path.display()))?
            .deserialize()
            .collect::<Result<Vec<Record>, csv::Error>>()
            .context("Failed to read and deserialize CSV records")
    }

    fn clone_repo(out_dir: &Path) -> Result<PathBuf> {
        let repo_dir = out_dir.join("abbrv.jabref.org");
        if !repo_dir.exists() {
            let status = Command::new("git")
                .args([
                    "clone",
                    "https://github.com/JabRef/abbrv.jabref.org.git",
                    &repo_dir.to_string_lossy(),
                ])
                .status()
                .context("Failed to launch git")?;
            if !status.success() {
                return Err(anyhow::anyhow!(
                    "git clone failed with exit status: {status}"
                ));
            }
        }
        Ok(repo_dir)
    }

    fn process_csv_files(repo_dir: &Path, import_order: Order) -> Result<Vec<Record>> {
        let journals_path = repo_dir.join("journals");
        import_order
            .file_suffixes()
            .iter()
            .map(|suffix| {
                let file_path = journals_path.join(format!("journal_abbreviations_{suffix}.csv"));
                read_csv(&file_path)
            })
            .collect::<Result<Vec<Vec<Record>>>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    const fn determine_order() -> Order {
        if cfg!(feature = "dot") {
            Order::Dots
        } else {
            Order::Dotless
        }
    }
}
