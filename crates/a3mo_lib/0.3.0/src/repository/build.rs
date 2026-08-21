use crate::sql::sqlite;
use indextree::{Arena, NodeId};
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

extern crate rayon;
use rayon::prelude::*;

use delta_patch;

extern crate rusqlite;

extern crate custom_error;
use custom_error::custom_error;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json;

use std::fs::File;

use delta_patch::mksum::SignatureOptions;

extern crate failure;

custom_error! {pub BuildRepoError
    FolderNotFound = "Folder not found!",
    NodeAppendError = "Node could not be appended",
    SQLError{source: rusqlite::Error} = "SQL Error",
    WalkDirError{source: walkdir::Error} = "Walkdir Error",
    CryptoError{source: easy_xxhash64::file_hash::CryptoError} = "Crypto Error",
    SerdeError{source: serde_json::Error} = "Serde Error",
    SystemTimeErr{source: std::time::SystemTimeError} = "System Time Error",
    IOError{source: std::io::Error} = "IO Error"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystemEntity {
    pub name: String,
    pub is_folder: bool,
    pub hash: u64,
}
impl FileSystemEntity {
    pub fn new(name: &str, repo_path: &str) -> Result<FileSystemEntity, BuildRepoError> {
        let is_directory = Path::is_dir(name.as_ref());
        let mut xhash: u64 = 0;
        if !is_directory {
            xhash = easy_xxhash64::file_hash::hash_path(name)?;
            let signame: String = String::from(name) + ".a3mo_delta";
            let mut base = File::open(&name)?;
            let mut sig = File::create(&signame)?;
            delta_patch::mksum::generate_signature(
                &mut base,
                &SignatureOptions::default(),
                &mut sig,
            )?;
        }
        println!("{:?}:\t{:?}", &name, &xhash);
        Ok(FileSystemEntity {
            name: String::from(name.replace(repo_path, "").trim_start_matches('\\')),
            is_folder: is_directory,
            hash: xhash,
        })
    }
}

fn build_tree(
    _name: &str,
    arena: &mut Arena<FileSystemEntity>,
    repo_path: String,
    rayon: bool,
) -> Result<NodeId, BuildRepoError> {
    let mut node_map: HashMap<String, NodeId> = HashMap::new();

    let mut root_node: Option<NodeId> = None;

    //Insert nodes

    if rayon {
        let entries: Vec<std::result::Result<walkdir::DirEntry, walkdir::Error>> =
            WalkDir::new(&repo_path).into_iter().collect();
        let fsxe_s: Vec<FileSystemEntity> = entries
            .par_iter()
            .map(|p| {
                let fa: &std::result::Result<walkdir::DirEntry, walkdir::Error> = p;
                let f = fa.as_ref().unwrap();

                let fname = f.path().to_str().unwrap();
                FileSystemEntity::new(fname, &repo_path.as_str()).unwrap()
            })
            .collect();

        for fse in fsxe_s {
            let fse_s = String::from(&fse.name);

            let node_id = arena.new_node(fse);

            if root_node.is_none() {
                root_node = Some(node_id);
            }
            node_map.insert(fse_s, node_id);
        }
    } else {
        for entry in WalkDir::new(&repo_path) {
            let f = entry?;

            let fname = f.path().to_str().unwrap();

            let fse = FileSystemEntity::new(fname, &repo_path.as_str())?;

            let fse_s = String::from(&fse.name);

            let node_id = arena.new_node(fse);

            if root_node.is_none() {
                root_node = Some(node_id);
            }

            node_map.insert(fse_s, node_id);
        }
    }

    for n in &node_map {
        let node_name = n.0;
        let node_index = n.1;

        //TODO Fix for linux
        let xf = match node_name.rfind('\\') {
            Some(v) => v,
            None => 0,
        };
        let parent_name = node_name.split_at(xf).0;

        let parent = node_map.get(parent_name);
        match parent {
            Some(v) => {
                //Prevent self attaching on head
                if v == node_index {
                    continue;
                }
                v.append(*node_index, arena);
            }
            None => continue,
        }
    }

    Ok(root_node.unwrap())
}

fn remove_old_delta(path: &str) -> Result<(), BuildRepoError> {
    let root_path = path.trim_end_matches(".a3mo");
    for f in WalkDir::new(root_path) {
        let fx = f?;
        let path = fx.path();

        if path.to_str().unwrap().contains(".a3mo_delta") {
            println!("{:?}", fx);
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}

/// (Re)build a repository
/// * `name` : Repository name (Has to be created using new command)
/// * `fmt_json` : Output formatted json
/// * `rayon` : Parallelize building using rayon (requires multiple cores/threads)
pub fn build(name: &str, fmt_json: bool, rayon: bool) -> Result<(), BuildRepoError> {
    let mut conn = sqlite::get_conn()?;
    let repo = sqlite::get_repository(name, &mut conn)?;

    if !Path::exists(repo.path.as_ref()) {
        return Err(BuildRepoError::FolderNotFound);
    };

    println!("Building repository {:?}", &name);
    let start = SystemTime::now();

    let sync_folder_path: String = String::clone(&repo.path) + "\\.a3mo";

    //Delete sync folder
    #[allow(unused_must_use)]
    {
        std::fs::remove_dir_all(&sync_folder_path);
    }

    remove_old_delta(&sync_folder_path)?;

    let arena = &mut Arena::new();

    let _root_node = build_tree(name, arena, String::clone(&repo.path), rayon)?;

    let json = if !fmt_json {
        serde_json::to_string(&arena)?
    } else {
        serde_json::to_string_pretty(&arena)?
    };

    //Create sync folder
    std::fs::create_dir(&sync_folder_path)?;

    //Save json at repo
    std::fs::write(sync_folder_path + "\\sync.json", json)?;

    println!("Finished building. {:?} sec", start.elapsed()?);

    Ok(())
}
