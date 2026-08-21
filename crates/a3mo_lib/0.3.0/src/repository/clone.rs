use url::Url;
extern crate custom_error;
use crate::repository::build::FileSystemEntity;
use crate::sql;
use crate::sql::sqlite;
use custom_error::custom_error;
use indextree::Arena;
use rayon::prelude::*;
use reqwest;
use std::fs::File;
use std::io::Seek;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

custom_error! {pub CloneError
    FolderNotFound = "Folder not found!",
    NodeAppendError = "Node could not be appended",
    FileParentError = "File without parent found !",
    SQLError{source: rusqlite::Error} = "SQL Error",
    WalkDirError{source: walkdir::Error} = "Walkdir Error",
    CryptoError{source: easy_xxhash64::file_hash::CryptoError} = "Crypto Error",
    SerdeError{source: serde_json::Error} = "Serde Error",
    SystemTimeErr{source: std::time::SystemTimeError} = "System Time Error",
    IOError{source: std::io::Error} = "IO Error",
    ParseError{source: url::ParseError} = "Parse Error",
    RequestError{source: reqwest::Error} = "Request Error"
}

/// Clone an remote repository.
/// * `path` : Path where the data will be stored (can be relative or absolute)
/// * `url` : URL to the a3mo folder
/// * `name` : Repo name
pub fn clone(path: &str, url: &str, name: &str) -> Result<(), CloneError> {
    Url::parse(url)?;

    println!("Cloning repository {:?}", &name);
    let start = SystemTime::now();

    let sync_url = url.to_owned() + "/sync.json";
    let mut sync_json_resp = reqwest::get(&sync_url)?;
    let jstring = sync_json_resp.text()?;

    let arena: Arena<FileSystemEntity> = serde_json::from_str(jstring.as_str())?;

    let mut conn = sqlite::get_conn()?;

    sql::sqlite::insert_repository(name, path, url, &mut conn)?;

    let repository = sql::sqlite::get_repository(name, &mut conn)?;

    // Insert new repo into db
    for fse_node in arena.iter() {
        let fse = fse_node.get();
        let parent_node = match fse_node.parent() {
            Some(v) => arena.get(v),
            None => None,
        };

        if fse.is_folder {
            match parent_node {
                Some(v) => {
                    let parent_id =
                        sql::sqlite::get_folder_parent_id(v.get(), repository.id, &conn)?;
                    //println!("SOME FOLDER INSERT {:?} -> {:?}[{:?}]", &fse, &v.get(), &parent_id);
                    sql::sqlite::insert_folder(
                        fse.name.as_str(),
                        repository.id,
                        Some(parent_id),
                        &conn,
                    )?;
                }
                None => {
                    //println!("NONE FOLDER INSERT {:?}", &fse);
                    sql::sqlite::insert_folder(fse.name.as_str(), repository.id, None, &conn)?;
                }
            }
        } else {
            match parent_node {
                Some(v) => {
                    let parent_id = sql::sqlite::get_file_parent_id(v.get(), repository.id, &conn)?;

                    println!(
                        "SOME FILE INSERT {:?} -> {:?}[{:?}]",
                        &fse,
                        &v.get(),
                        &parent_id
                    );
                    sql::sqlite::insert_file(
                        fse.name.as_str(),
                        fse.hash,
                        repository.id,
                        parent_id,
                        &conn,
                    )?;
                }
                None => {
                    //This should never happen, every file has atleast "" as root
                    return Err(CloneError::FileParentError);
                }
            }
        }
    }

    //Download missing files

    if !Path::new(&path).exists() {
        fs::create_dir_all(&path)?;
    }

    let mut to_download: Vec<(String, String)> = Vec::new();

    for fse_node in arena.iter() {
        let fse = fse_node.get();
        if !fse.is_folder {
            let fullpath = path.to_owned() + "\\" + &fse.hash.to_string();

            if !Path::new(&fullpath).exists() {
                let url_p = Url::parse(url)?;

                let xpath = "../".to_owned() + &fse.name;
                let uri = url_p.join(&xpath)?;

                println!("PUSH {:?} -> {:?} || {:?}", &fse, &uri, &fullpath);

                to_download.push((uri.to_string(), fullpath));
            } else {
                println!("Skip {:?}", &fse);
            }
        }
    }

    let _x: Vec<u64> = to_download
        .par_iter()
        .map(|c| {
            println!("Downloading {:?}", &c);

            let url: &String = &c.0;
            let filepath: &String = &c.1;
            let resp = reqwest::get(url.as_str());
            let out = File::create(filepath);

            if resp.is_err() {
                println!("Could not download {:?} Err: {:?}", url, resp.err());
                return 0;
            }

            if out.is_err() {
                println!("Could create file {:?} Err: {:?}", filepath, out.err());
                return 0;
            }

            let mut uw_resp = resp.unwrap();
            let mut uw_out = out.unwrap();

            let iocp = io::copy(&mut uw_resp, &mut uw_out);
            if iocp.is_err() {
                println!("Could copy {:?} Err: {:?}", filepath, iocp.err());
                return 0;
            }

            let stpos = uw_out.stream_position();

            if stpos.is_err() {
                println!("Could get stpos {:?} Err: {:?}", filepath, stpos.err());
                return 0;
            }

            stpos.unwrap()
        })
        .collect();

    let elapsed = start.elapsed()?;
    let size: u64 = _x.iter().sum();
    println!(
        "Finished cloning. {:?} byte in {:?} sec ({:?} MB/s)",
        &size,
        &elapsed,
        (size / elapsed.as_secs()) / 1_000_000
    );

    Ok(())
}
