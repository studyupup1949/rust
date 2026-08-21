use ada_judge_public_models::problems::ProblemConfig;
use colored::Colorize;
use std::path::Path;
use tokio::{
    fs::{self, File, read_to_string},
    io::AsyncWriteExt,
    process::Command,
};

use crate::{
    Cli,
    config::load_config,
    constants::{BASE_CONFIG, DEFAULT_CHECKER},
    db::Database,
};

pub async fn handle_prepare_problem_cmd(path: &Path) -> anyhow::Result<()> {
    fs::create_dir(path).await.map_err(|e| {
        println!(
            "{} {e}",
            "Failed to create problem's directory:".red().bold()
        );
        e
    })?;
    fs::create_dir(path.join("tests")).await.map_err(|e| {
        println!("{} {e}", "Failed to create tests' directory:".red().bold());
        e
    })?;
    let mut checker = File::create(path.join("checker.rs")).await.map_err(|e| {
        println!(
            "{} {e}",
            "Failed to create checker source file:".red().bold()
        );
        e
    })?;
    checker
        .write_all(DEFAULT_CHECKER.as_bytes())
        .await
        .map_err(|e| {
            println!(
                "{} {e}",
                "Failed to write default checker's code to checker source file:"
                    .red()
                    .bold()
            );
            e
        })?;
    let mut problem_config_file = File::create(path.join("config.toml")).await.map_err(|e| {
        println!("{} {e}", "Failed to create config file:".red().bold());
        e
    })?;
    problem_config_file
        .write_all(BASE_CONFIG.as_bytes())
        .await
        .map_err(|e| {
            println!(
                "{} {e}",
                "Failed to write a base config to config file:".red().bold()
            );
            e
        })?;
    Command::new("rustc")
        .arg(path.join("checker.rs"))
        .args(["-O", "-C", "target-cpu=native", "-C", "lto", "-o"])
        .arg(path.join("checker"))
        .spawn()
        .map_err(|e| {
            println!("{} {e}", "Failed to compile checker:".red().bold());
            e
        })?;
    println!("{}", "Problem has prepared successfully".green().italic());

    Ok(())
}

#[allow(clippy::cast_possible_wrap)]
pub async fn handle_create_problem_cmd(cli: &Cli, path: &Path) -> anyhow::Result<()> {
    let config = load_config(cli).await?;
    let problem_config: ProblemConfig = toml::from_str(
        &read_to_string(path.join("config.toml"))
            .await
            .map_err(|e| {
                println!("{} {e}", "Failed to read problem's config:".red().bold());
                e
            })?,
    )
    .map_err(|e| panic!("Failed to create problem: {e}"))?;
    let db = Database::new(&config).await.map_err(|e| {
        println!("{} {e}", "Failed to connect to database:".red().bold());
        e
    })?;
    let problem_id = db.insert_problem(&problem_config).await.map_err(|e| {
        println!(
            "{} {e}",
            "Failed to insert a problem to database:".red().bold()
        );
        e
    })?;
    for (ind, subgroup) in problem_config.subgroups.iter().enumerate() {
        db.insert_problems_subgroup(problem_id, subgroup, ind as i64)
            .await
            .map_err(|e| {
                println!(
                    "{} {e}",
                    "Failed to insert a problem's subgroup to database:"
                        .red()
                        .bold()
                );
                e
            })?;
    }
    println!(
        "{} {problem_id}",
        "Problem has inserted to database successfully. Problem's id:"
            .green()
            .italic()
    );

    Ok(())
}

pub async fn handle_insert_range_of_tests_to_problem_cmd(
    path: &Path,
    from: &i32,
    to: &i32,
) -> anyhow::Result<()> {
    for i in *from..=*to {
        let test_str = i.to_string();
        fs::create_dir(path.join("tests").join(&test_str))
            .await
            .map_err(|e| {
                println!(
                    "{}{}{} {e}",
                    "Failed to create test #".red().bold(),
                    test_str.red().bold(),
                    " directory:".red().bold()
                );
                e
            })?;
        File::create(path.join("tests").join(&test_str).join("in"))
            .await
            .map_err(|e| {
                println!(
                    "{}{}{} {e}",
                    "Failed to create test #".red().bold(),
                    test_str.red().bold(),
                    " input file:".red().bold()
                );
                e
            })?;
        File::create(path.join("tests").join(&test_str).join("out"))
            .await
            .map_err(|e| {
                println!(
                    "{}{}{} {e}",
                    "Failed to create test #".red().bold(),
                    test_str.red().bold(),
                    " output file:".red().bold()
                );
                e
            })?;
    }
    println!(
        "{}",
        "Range of tests has inserted successfully".green().italic()
    );

    Ok(())
}
