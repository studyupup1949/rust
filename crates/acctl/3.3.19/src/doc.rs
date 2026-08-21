//! Documentation subcommands: `acctl doc build|serve|generate-vars|clean`.
//!
//! Wraps `mdbook` and `cargo doc`. On first use `mdbook` is auto-installed
//! via `cargo install mdbook --locked` if not found on PATH.

use anyhow::{anyhow, Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::DocCommand;

pub async fn cmd_doc(cmd: &DocCommand) -> Result<()> {
    match cmd {
        DocCommand::Build => cmd_build(),
        DocCommand::Serve { port } => cmd_serve(*port),
        DocCommand::GenerateVars => cmd_generate_vars(),
        DocCommand::Clean => cmd_clean(),
    }
}

fn project_root() -> Result<PathBuf> {
    let pj = PathBuf::from("project.json");
    if !pj.exists() {
        return Err(anyhow!(
            "project.json not found in current directory — run from an AutoCore project root"
        ));
    }
    Ok(PathBuf::from("."))
}

fn doc_dir(root: &Path) -> Result<PathBuf> {
    let d = root.join("doc");
    if !d.join("book.toml").exists() {
        return Err(anyhow!(
            "doc/book.toml not found — this project was created before `acctl doc` support.\n\
             Copy doc/ from a freshly-generated project (`acctl new <tmp>`) to add it."
        ));
    }
    Ok(d)
}

fn cmd_build() -> Result<()> {
    let root = project_root()?;
    let doc = doc_dir(&root)?;

    generate_vars_file(&root, &doc)?;
    run_cargo_doc(&root, &doc)?;
    ensure_mdbook()?;

    println!("{}", "Building book...".bold());
    let status = Command::new("mdbook")
        .arg("build")
        .arg(&doc)
        .status()
        .context("failed to execute mdbook")?;
    if !status.success() {
        return Err(anyhow!("mdbook build failed"));
    }

    let out = doc.join("book").join("index.html");
    println!();
    println!("{}", "Documentation built.".green());
    println!("  Open: {}", out.display());
    println!("  Or zip doc/book/ to distribute as a standalone HTML site.");
    Ok(())
}

fn cmd_serve(port: u16) -> Result<()> {
    let root = project_root()?;
    let doc = doc_dir(&root)?;

    generate_vars_file(&root, &doc)?;
    run_cargo_doc(&root, &doc)?;
    ensure_mdbook()?;

    println!(
        "{} http://localhost:{}",
        "Serving documentation at".bold(),
        port
    );
    println!("  (Ctrl+C to stop)");
    let status = Command::new("mdbook")
        .arg("serve")
        .arg(&doc)
        .arg("-p")
        .arg(port.to_string())
        .status()
        .context("failed to execute mdbook")?;
    if !status.success() {
        return Err(anyhow!("mdbook serve exited with error"));
    }
    Ok(())
}

fn cmd_generate_vars() -> Result<()> {
    let root = project_root()?;
    let doc = doc_dir(&root)?;
    generate_vars_file(&root, &doc)?;
    Ok(())
}

fn cmd_clean() -> Result<()> {
    let root = project_root()?;
    let doc = doc_dir(&root)?;

    let book = doc.join("book");
    if book.exists() {
        fs::remove_dir_all(&book).with_context(|| format!("removing {}", book.display()))?;
        println!("  Removed {}", book.display());
    }
    let rd = doc.join("src").join("rustdoc");
    if rd.exists() {
        fs::remove_dir_all(&rd).with_context(|| format!("removing {}", rd.display()))?;
        println!("  Removed {}", rd.display());
    }
    println!("{}", "Clean complete.".green());
    Ok(())
}

// ---------------------------------------------------------------------------
// mdbook auto-install
// ---------------------------------------------------------------------------

fn ensure_mdbook() -> Result<()> {
    if Command::new("mdbook")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }

    println!(
        "{}",
        "mdbook not found on PATH — installing (one-time, ~60s)...".yellow()
    );
    let status = Command::new("cargo")
        .args(["install", "mdbook", "--locked"])
        .status()
        .context("failed to spawn cargo — is the Rust toolchain installed?")?;
    if !status.success() {
        return Err(anyhow!(
            "`cargo install mdbook --locked` failed. Install it manually and retry."
        ));
    }

    if Command::new("mdbook")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        println!("{}", "mdbook installed.".green());
        Ok(())
    } else {
        Err(anyhow!(
            "mdbook installed but not found on PATH. \
             Ensure ~/.cargo/bin is on your PATH and retry."
        ))
    }
}

// ---------------------------------------------------------------------------
// cargo doc → doc/src/rustdoc/
// ---------------------------------------------------------------------------

fn run_cargo_doc(root: &Path, doc: &Path) -> Result<()> {
    let control_manifest = root.join("control").join("Cargo.toml");
    if !control_manifest.exists() {
        println!(
            "  {} control/Cargo.toml not found — skipping Rustdoc generation.",
            "note:".dimmed()
        );
        return Ok(());
    }

    println!("{}", "Generating Rustdoc for control program...".bold());
    let status = Command::new("cargo")
        .args(["doc", "--no-deps", "--manifest-path"])
        .arg(&control_manifest)
        .status()
        .context("failed to spawn cargo doc")?;
    if !status.success() {
        return Err(anyhow!("cargo doc failed"));
    }

    let src = root.join("control").join("target").join("doc");
    let dst = doc.join("src").join("rustdoc");
    if dst.exists() {
        fs::remove_dir_all(&dst).with_context(|| format!("removing {}", dst.display()))?;
    }
    copy_dir_recursive(&src, &dst)
        .with_context(|| format!("copying {} → {}", src.display(), dst.display()))?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// generate-vars: project.json → doc/src/variables.md
// ---------------------------------------------------------------------------

fn generate_vars_file(root: &Path, doc: &Path) -> Result<()> {
    let pj_path = root.join("project.json");
    let content = fs::read_to_string(&pj_path).context("reading project.json")?;
    let project: serde_json::Value =
        serde_json::from_str(&content).context("parsing project.json")?;

    let empty = serde_json::Map::new();
    let vars = project
        .get("variables")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty);

    let mut linked: Vec<(&String, &serde_json::Value)> = Vec::new();
    let mut bit_mapped: Vec<(&String, &serde_json::Value)> = Vec::new();
    let mut plain: Vec<(&String, &serde_json::Value)> = Vec::new();

    for (name, cfg) in vars {
        if cfg.get("link").and_then(|v| v.as_str()).is_some() {
            linked.push((name, cfg));
        } else if cfg.get("source").and_then(|v| v.as_str()).is_some() {
            bit_mapped.push((name, cfg));
        } else {
            plain.push((name, cfg));
        }
    }

    for v in [&mut linked, &mut bit_mapped, &mut plain] {
        v.sort_by(|a, b| a.0.cmp(b.0));
    }

    let mut md = String::new();
    md.push_str("# Global Memory Variables\n\n");
    md.push_str("*Auto-generated by `acctl doc generate-vars` — do not edit by hand.*\n\n");
    md.push_str(&format!(
        "**Total variables:** {} ({} hardware-linked, {} bit-mapped, {} plain)\n\n",
        vars.len(),
        linked.len(),
        bit_mapped.len(),
        plain.len()
    ));

    if !linked.is_empty() {
        md.push_str("## Hardware-Linked Variables\n\n");
        md.push_str("| FQDN | Type | Link | Description |\n");
        md.push_str("|------|------|------|-------------|\n");
        for (name, cfg) in &linked {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} |\n",
                md_escape(name),
                field_str(cfg, "type"),
                field_str(cfg, "link"),
                md_escape(&field_str(cfg, "description"))
            ));
        }
        md.push('\n');
    }

    if !bit_mapped.is_empty() {
        md.push_str("## Bit-Mapped Variables\n\n");
        md.push_str("| FQDN | Type | Source | Bit | Description |\n");
        md.push_str("|------|------|--------|-----|-------------|\n");
        for (name, cfg) in &bit_mapped {
            md.push_str(&format!(
                "| `{}` | `{}` | `{}` | {} | {} |\n",
                md_escape(name),
                field_str(cfg, "type"),
                field_str(cfg, "source"),
                field_str(cfg, "bit"),
                md_escape(&field_str(cfg, "description"))
            ));
        }
        md.push('\n');
    }

    if !plain.is_empty() {
        md.push_str("## Other Variables\n\n");
        md.push_str("| FQDN | Type | Initial | Description |\n");
        md.push_str("|------|------|---------|-------------|\n");
        for (name, cfg) in &plain {
            md.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                md_escape(name),
                field_str(cfg, "type"),
                field_str(cfg, "initial"),
                md_escape(&field_str(cfg, "description"))
            ));
        }
        md.push('\n');
    }

    if vars.is_empty() {
        md.push_str("*No variables defined in project.json.*\n");
    }

    let out = doc.join("src").join("variables.md");
    fs::write(&out, md).with_context(|| format!("writing {}", out.display()))?;
    println!(
        "  Wrote {} ({} variables)",
        out.display(),
        vars.len()
    );
    Ok(())
}

fn field_str(cfg: &serde_json::Value, key: &str) -> String {
    match cfg.get(key) {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Null) | None => "—".to_string(),
        Some(v) => v.to_string(),
    }
}

fn md_escape(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}
