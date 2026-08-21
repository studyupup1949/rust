extern crate custom_error;
use crate::sql::sqlite;
use custom_error::custom_error;
use std::process::Command;
use std::time::SystemTime;

extern crate rusqlite;

custom_error! {pub RunError
    FileNotFound = "File not found error",
    SQLError{source: rusqlite::Error} = "SQL Error",
    SystemTimeErr{source: std::time::SystemTimeError} = "System Time Error",
    IOError{source: std::io::Error} = "IO Error"
}

/// Starts ArmA3 with given repository name and args
/// * `name` : Repository name (used for the Build command and displayed on the GUI)
/// * `arma_path` : Path to Arma3 executable
/// * `tmp_folder` : Path to tmp folder
/// * `opt_args` : Optional arguments
pub fn run(
    name: &str,
    arma_path: &str,
    tmp_folder: &str,
    opt_args: Option<Vec<String>>,
) -> Result<(), RunError> {
    let start = SystemTime::now();

    let mut conn = sqlite::get_conn()?;
    let repository = sqlite::get_repository(&name, &mut conn)?;

    let repo_folders = sqlite::get_repo_folders(repository.id, &mut conn)?;
    let repo_files = sqlite::get_repo_files(repository.id, &mut conn)?;

    if tmp_folder.chars().nth(0) != repository.path.chars().nth(0) {
        println!("Both folders have to be on the same drive");
    }

    //Cleanup old folder
    std::fs::remove_dir_all(tmp_folder).unwrap_or_default();

    for repo_folder in &repo_folders {
        let xfolder = tmp_folder.to_owned() + "\\" + &repo_folder.name;
        println!("{:?}", xfolder);
        std::fs::create_dir_all(xfolder)?;
    }

    for repo_file in &repo_files {
        let dfile = tmp_folder.to_owned() + "\\" + &repo_file.name;
        let sfile = repository.path.to_owned() + "\\" + &repo_file.xx_hash64;

        println!("{:?} -> {:?}", dfile, sfile);

        //Same drive only !!!!
        std::fs::hard_link(sfile, dfile)?;
    }

    let mut args: Vec<String> = Vec::new();

    if let Some(mut v) = opt_args {
        args.append(&mut v);
    }

    for repo_folder in &repo_folders {
        if repo_folder.is_root {
            continue;
        }
        let fchar = repo_folder.name.chars().nth(0);
        if fchar.unwrap() == '@' && !repo_folder.name.contains('\\') {
            println!("{:?}", repo_folder.name);
            let f = "-mod=".to_owned()
                + &tmp_folder.to_owned()
                + "\\"
                + String::from(&repo_folder.name).as_str()
                + ";";
            args.push(f);
        }
    }

    println!("{:?}", arma_path);
    println!("{:?}", args);

    let _fx = Command::new(arma_path).args(args).spawn();

    let elapsed = start.elapsed()?;

    println!("Elapsed: {:?}", elapsed);
    Ok(())
}
