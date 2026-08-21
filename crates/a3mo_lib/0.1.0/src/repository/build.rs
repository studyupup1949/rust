use crate::sql::sqlite;
use std::path::{Path};
use walkdir::WalkDir;
use indextree::{Arena, NodeId};

extern crate rusqlite;

extern crate custom_error;
use custom_error::custom_error;

use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json;
use serde_indextree::Node;

custom_error! {pub BuildRepoError
    FolderNotFound = "Folder not found!",
    SQLError{source: rusqlite::Error} = "SQL Error",
    WalkDirError{source: walkdir::Error} = "Walkdir Error",
    CryptoError{source: crate::crypto::xx_hasher::CryptoError} = "Crypto Error",
    SerdeError{source: serde_json::Error} = "Serde Error"
}

#[derive(Debug, Serialize)]
pub struct FileSystemEntity {
    pub name: String,
    pub is_folder: bool,
    pub hash: u64,
}
impl FileSystemEntity {
    pub fn new(name: &str) -> Result<FileSystemEntity, BuildRepoError> {
        let is_directory = Path::is_dir(name.as_ref());
        let mut xhash: u64 = 0;
        if !is_directory {
            xhash = crate::crypto::xx_hasher::hash_path(name.as_ref())?
        }
        Ok(FileSystemEntity {
            name: String::from(name),
            is_folder: is_directory,
            hash: xhash,
        })
    }
}

fn build_tree(name: &str, arena: &mut Arena<FileSystemEntity>) -> Result<NodeId, BuildRepoError> {
    let repo = sqlite::get_repository(name)?;

    if Path::exists(repo.path.as_ref()) == false {
        return Err(BuildRepoError::FolderNotFound);
    };

    //let arena = &mut Arena::new();

    let mut node_map: HashMap<String, NodeId> = HashMap::new();

    let mut root_node: Option<NodeId> = None;

    //Insert nodes
    for entry in WalkDir::new(repo.path) {
        let f = entry?;

        let fname = f.path().to_str().unwrap();

        let fse = FileSystemEntity::new(fname)?;

        let fse_s = String::from(&fse.name);

        let node_id = arena.new_node(fse);

        if root_node.is_none(){
            root_node = Some(node_id);
        }

        node_map.insert(fse_s, node_id);
    }

    for n in &node_map {
        let node_name = n.0;
        let node_index = n.1;

        //TODO Fix for linux
        let xf = node_name.rfind("\\").unwrap();
        let parent_name = node_name.split_at(xf).0;
/*
        println!("{:?}", node_name);
        println!("{:?}", parent_name);
*/
        let parent = node_map.get(parent_name);
        match parent {
            Some(v) => {
                v.append(*node_index, arena);
            }
            None => {
                continue
            }
        }

    }

    Ok(root_node.unwrap())
}


/// (Re)build a repository
/// [name] : Repository name (Has to be created using new command)
pub fn build(name: &str) -> Result<(), BuildRepoError> {
    let arena = &mut Arena::new();
    let root_node = build_tree(name, arena)?;
    let sn = Node::new(root_node, arena);

    let json = serde_json::to_string(&sn)?;

    //Save json at repo

    Ok(())
}
