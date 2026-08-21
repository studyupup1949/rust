use crate::constants::{BASE_CONFIG, DEFAULT_CHECKER};
use anyhow::anyhow;
use colored::Colorize;
use std::path::Path;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    process::Command,
};
use zip_extensions::zip_writer::zip_create_from_directory;

pub async fn handle_prepare_problem_cmd(path: &Path) -> anyhow::Result<()> {
    fs::create_dir(path).await.map_err(|e| {
        eprintln!(
            "{} {e}",
            "Failed to create problem's directory:".red().bold()
        );
        e
    })?;
    fs::create_dir(path.join("tests")).await.map_err(|e| {
        eprintln!("{} {e}", "Failed to create tests' directory:".red().bold());
        e
    })?;
    let mut checker = File::create(path.join("checker.rs")).await.map_err(|e| {
        eprintln!(
            "{} {e}",
            "Failed to create checker source file:".red().bold()
        );
        e
    })?;
    checker
        .write_all(DEFAULT_CHECKER.as_bytes())
        .await
        .map_err(|e| {
            eprintln!(
                "{} {e}",
                "Failed to write default checker's code to checker source file:"
                    .red()
                    .bold()
            );
            e
        })?;
    let mut problem_config_file = File::create(path.join("config.toml")).await.map_err(|e| {
        eprintln!("{} {e}", "Failed to create config file:".red().bold());
        e
    })?;
    problem_config_file
        .write_all(BASE_CONFIG.as_bytes())
        .await
        .map_err(|e| {
            eprintln!(
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
            eprintln!("{} {e}", "Failed to compile checker:".red().bold());
            e
        })?;
    println!("{}", "Problem has prepared successfully".green().italic());

    Ok(())
}

pub async fn handle_insert_tests_to_problem_cmd(
    path: &Path,
    from: &i32,
    to: &i32,
) -> anyhow::Result<()> {
    for i in *from..=*to {
        let test_str = i.to_string();
        fs::create_dir(path.join("tests").join(&test_str))
            .await
            .map_err(|e| {
                eprintln!(
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
                eprintln!(
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
                eprintln!(
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

pub async fn handle_run_cmd(
    path: &Path,
    solution_path: &Path,
    from: &i32,
    to: &i32,
) -> anyhow::Result<()> {
    for i in *from..=*to {
        let test_str = i.to_string();
        let stdin = File::open(path.join("tests").join(test_str.clone()).join("in"))
            .await
            .map_err(|e| {
                eprintln!(
                    "{}{}{} {e}",
                    "Failed to test solution on test #".red().bold(),
                    test_str.red().bold(),
                    ":".red().bold()
                );
                e
            })?;
        let stdout = File::create(path.join("tests").join(test_str.clone()).join("out"))
            .await
            .map_err(|e| {
                eprintln!(
                    "{}{}{} {e}",
                    "Failed to test solution on test #".red().bold(),
                    test_str.red().bold(),
                    ":".red().bold()
                );
                e
            })?;

        let res = Command::new(solution_path)
            .stdin(stdin.into_std().await)
            .stdout(stdout.into_std().await)
            .status()
            .await
            .map_err(|e| {
                eprintln!(
                    "{}{}{} {e}",
                    "Failed to test solution on test #".red().bold(),
                    test_str.red().bold(),
                    ":".red().bold()
                );
                e
            })?;

        if !res.success() {
            eprintln!(
                "{}{}",
                "Failed to test solution on test #".red().bold(),
                test_str.red().bold(),
            );
            return Err(anyhow!("failed to test solution on test #{test_str}"));
        }
    }

    println!("{}", "Solution has run successfully".green().italic());

    Ok(())
}

pub async fn handle_archive_problem_cmd(path: &Path) -> anyhow::Result<()> {
    let mut dst_path = path.to_owned();
    dst_path.pop();
    dst_path.push(format!(
        "{}.zip",
        path.file_name()
            .ok_or_else(|| {
                eprintln!(
                    "{} file name is empty",
                    "Failed to archive problem:".red().bold()
                );
                anyhow!("file name is empty")
            })?
            .to_string_lossy()
    ));
    zip_create_from_directory(&dst_path, &path.to_path_buf()).map_err(|e| {
        eprintln!("{} {e}", "Failed to archive problem:".red().bold());
        e
    })?;

    println!("{}", "Problem has archived successfully".green().italic());

    Ok(())
}
