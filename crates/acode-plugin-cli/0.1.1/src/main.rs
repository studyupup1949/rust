use clap::Parser;
use console::{Emoji, style};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::get;
use serde_json::json;
use std::time::Duration;
use std::{
    fs::{self, File},
    io::{Cursor, Read, Seek},
    path::Path,
};
use zip::ZipArchive;

static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");

/// Simple CLI tool to create Acode plugins easily
#[derive(Parser)]
#[command(
    name = "acode-plugin-cli",
    about = "Create an Acode plugin from official templates",
    long_about = "A user-friendly CLI tool to bootstrap Acode plugins using official templates.\nSupports both JavaScript and TypeScript templates with automatic setup."
)]
struct Cli {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _cli = Cli::parse();

    println!();
    println!("{}",style("╔════════════════════════════╗").cyan());
    println!("{}",style("║    ACODE PLUGIN CREATOR    ║").cyan());
    println!("{}",style("╚════════════════════════════╝").cyan());
    println!();

    println!(
        "{}{}",
        ROCKET,
        style("Welcome to the Acode Plugin Creator!")
            .bold()
            .blue()
            .bright()
    );
    println!("This tool will help you create a new Acode plugin from official templates.");
    println!();

    let theme = ColorfulTheme::default();

    println!(
        "{}{}",
        SPARKLES,
        style("Let's gather some information about your plugin:")
            .bold()
            .yellow()
    );
    println!();

    let name: String = Input::with_theme(&theme)
        .with_prompt("Plugin name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Plugin name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let name_lower = name
        .replace('-', "")
        .replace('_', "")
        .replace(' ', "")
        .to_lowercase();

    let plugin_id: String = Input::with_theme(&theme)
        .with_prompt("Plugin ID")
        .default(format!("com.{}.{}", name_lower, name_lower))
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Plugin ID cannot be empty")
            } else if !input.contains('.') {
                Err("Plugin ID should follow reverse domain notation (e.g., com.example.plugin)")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let version: String = Input::with_theme(&theme)
        .with_prompt("Version")
        .default("1.0.0".into())
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Version cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let description: String = Input::with_theme(&theme)
        .with_prompt("Description")
        .allow_empty(true)
        .interact_text()?;

    let repository: String = Input::with_theme(&theme)
        .with_prompt("Repository")
        .allow_empty(true)
        .interact_text()?;

    println!();
    println!("{}", style("Author Information:").bold().cyan().bright());

    let author_name: String = Input::with_theme(&theme)
        .with_prompt("Author name")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().is_empty() {
                Err("Author name cannot be empty")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let author_email: String = Input::with_theme(&theme)
        .with_prompt("Author email")
        .allow_empty(true)
        .validate_with(|input: &String| -> Result<(), &str> {
            if !input.trim().is_empty() && !input.contains('@') {
                Err("Please enter a valid email address")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let author_url: String = Input::with_theme(&theme)
        .with_prompt("Author website")
        .allow_empty(true)
        .interact_text()?;

    let author_github: String = Input::with_theme(&theme)
        .with_prompt("GitHub username")
        .allow_empty(true)
        .interact_text()?;

    println!("{}", style("Plugin Configuration:").bold().cyan().bright());

    let license_options = vec![
        "MIT",
        "Apache-2.0",
        "GPL-3.0",
        "BSD-3-Clause",
        "ISC",
        "Custom",
    ];

    let license_choice = Select::with_theme(&theme)
        .with_prompt("License")
        .items(&license_options)
        .default(0)
        .interact()?;

    let license = license_options[license_choice].to_string();

    let price: u32 = Input::with_theme(&theme)
        .with_prompt("Price (0 for free)")
        .default(0)
        .interact()?;

    let min_version_code: u32 = Input::with_theme(&theme)
        .with_prompt("Minimum Acode version code")
        .default(292)
        .interact()?;

    let keywords_input: String = Input::with_theme(&theme)
        .with_prompt("Keywords (comma-separated)")
        .allow_empty(true)
        .interact_text()?;

    let keywords: Vec<String> = if keywords_input.trim().is_empty() {
        vec![]
    } else {
        keywords_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let add_files = Confirm::with_theme(&theme)
        .with_prompt("Add additional files to include?")
        .default(false)
        .interact()?;

    let additional_files: Vec<String> = if add_files {
        let files_input: String = Input::with_theme(&theme)
            .with_prompt("Additional files (comma-separated)")
            .allow_empty(true)
            .interact_text()?;

        if files_input.trim().is_empty() {
            vec![]
        } else {
            files_input
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
    } else {
        vec![]
    };

    println!("{}", style("Choose your preferred template:").bold().yellow());

    let template_options = vec![
        "JavaScript",
        "TypeScript",
    ];

    let template_choice = Select::with_theme(&theme)
        .with_prompt("Programming Language")
        .items(&template_options)
        .default(0)
        .interact()?;

    let (repo_zip_url, plugin_dir_name, lang_name) = if template_choice == 0 {
        (
            "https://github.com/Acode-Foundation/acode-plugin/archive/refs/heads/main.zip",
            "acode-plugin-main",
            "JavaScript",
        )
    } else {
        (
            "https://github.com/Acode-Foundation/AcodeTSTemplate/archive/refs/heads/main.zip",
            "AcodeTSTemplate-main",
            "TypeScript",
        )
    };

    println!();
    println!("{}", style("Downloading template...").bold().magenta().bright());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Fetching {} template from GitHub...", lang_name));
    pb.enable_steady_tick(Duration::from_millis(100));

    let resp = get(repo_zip_url)?.error_for_status()?.bytes()?;
    pb.set_message("Template downloaded successfully!");
    pb.finish_and_clear();

    println!("{}", style("Setting up your plugin...").bold().magenta().bright());

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Extracting files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    let cursor = Cursor::new(resp);
    let output_dir = format!("{}", name);
    unzip(cursor, &output_dir)?;

    let nested_dir = format!("{}/{}", &output_dir, plugin_dir_name);
    for entry in fs::read_dir(&nested_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        fs::rename(
            &path,
            format!("{}/{}", &output_dir, file_name.to_string_lossy()),
        )?;
    }
    fs::remove_dir(&nested_dir)?;

    pb.set_message("Configuring plugin.json...");
    pb.finish_and_clear();

    let plugin_json_path = format!("{}/plugin.json", &output_dir);

    let author_obj = json!({
        "name": author_name,
        "email": author_email,
        "url": author_url,
        "github": author_github
    });

    let plugin_json = json!({
        "id": plugin_id,
        "name": name,
        "main": "dist/main.js",
        "version": version,
        "readme": "readme.md",
        "icon": "icon.png",
        "minVersionCode": min_version_code,
        "price": price,
        "repository": repository,
        "license": license,
        "author": author_obj,
        "description": description,
        "keywords": keywords,
        "files": additional_files,
        "changelogs": "changelogs.md"
    });

    let new_json = serde_json::to_string_pretty(&plugin_json)?;
    fs::write(&plugin_json_path, new_json)?;

    println!();
    println!("{}",style("╔═══════════════════╗").bright().green());
    println!("{}",style("║    SUCCESS! 🎉    ║").bright().green());
    println!("{}",style("╚═══════════════════╝").bright().green());
    println!();

    println!(
        "{}",
        style(&format!("Plugin '{}' created successfully!", name))
            .bold()
            .bright()
            .green()
    );
    println!("{}",style("Next steps:").bold());
    println!("  1. {} Navigate to your plugin directory",style("cd").bright().cyan());
    println!("     {}", style(&format!("cd {}", output_dir)).dim());
    println!("  2. {} Install dependencies",style("npm").bright().cyan());
    println!("     {}", style("npm install").dim());
    println!("  3. Start developing your plugin!");
    println!();

    println!("{}",style("Useful resources:").bold());
    println!("  • Acode Plugin Documentation: https://docs.acode.app/docs/getting-started/intro");
    println!("  • Plugin Marketplace: https://acode.app/plugins");
    println!();
    println!("{}", style("Happy coding!").bold().bright().magenta());
    println!();

    Ok(())
}

fn unzip<R: Read + Seek>(reader: R, output_dir: &str) -> std::io::Result<()> {
    let mut archive = ZipArchive::new(reader)?;
    fs::create_dir_all(output_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(output_dir).join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}
