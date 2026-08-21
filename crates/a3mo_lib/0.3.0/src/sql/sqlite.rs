extern crate rusqlite;
use crate::repository::build::FileSystemEntity;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

pub fn get_conn() -> Result<Connection> {
    let conn = Connection::open("a3mm.sqlite3")?;
    //conn.execute("PRAGMA journal_mode = WAL", NO_PARAMS)?;
    conn.execute("PRAGMA synchronous = OFF", NO_PARAMS)?;

    Ok(conn)
}

fn create(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;
    tx.execute(
        "create table if not exists repositories (\
         id integer primary key,\
         name text not null unique,\
         path text not null,\
         url text not null);",
        NO_PARAMS,
    )?;

    tx.execute(
        "CREATE TABLE if not exists folder (\
         id	INTEGER PRIMARY KEY AUTOINCREMENT,\
         name	INTEGER,\
         is_root	INTEGER DEFAULT 0,\
         repository_id	INTEGER,\
         parent_id	INTEGER,\
         FOREIGN KEY(repository_id) REFERENCES repositories(id)\
         )",
        NO_PARAMS,
    )?;

    //SQLITE has a weird habit of converting long INTEGERS to *10^X representations, which kill the contained value
    tx.execute(
        "CREATE TABLE if not exists file (\
         id	INTEGER PRIMARY KEY AUTOINCREMENT,\
         name	INTEGER,\
         xxHash64	TEXT,\
         repository_id	INTEGER,\
         parent_id	INTEGER,\
         FOREIGN KEY(parent_id) REFERENCES folder(id),\
         FOREIGN KEY(repository_id) REFERENCES repositories(id)\
         )",
        NO_PARAMS,
    )?;

    tx.commit()?;

    Ok(())
}

pub fn get_file_parent_id(
    parent_node: &FileSystemEntity,
    repo_id: i64,
    conn: &Connection,
) -> Result<i64> {
    println!(
        "SELECT id FROM folder WHERE name = {:?} AND repository_id = {:?} LIMIT 1",
        &parent_node.name, &repo_id
    );

    let mut stmt =
        conn.prepare("SELECT id FROM folder WHERE name = ?1 AND repository_id = ?2 LIMIT 1")?;

    let repo = stmt.query_map(
        &[String::from(&parent_node.name), repo_id.to_string()],
        |row| Ok(row.get(0)?),
    )?;

    match repo.last() {
        Some(x) => x,
        None => Err(rusqlite::Error::InvalidQuery),
    }
}

pub fn get_folder_parent_id(
    parent_node: &FileSystemEntity,
    repo_id: i64,
    conn: &Connection,
) -> Result<i64> {
    println!(
        "SELECT id FROM folder WHERE name = {:?} AND repository_id = {:?} LIMIT 1",
        &parent_node.name, &repo_id
    );

    let mut stmt =
        conn.prepare("SELECT id FROM folder WHERE name = ?1 AND repository_id = ?2 LIMIT 1")?;

    let repo = stmt.query_map(
        &[String::from(&parent_node.name), repo_id.to_string()],
        |row| Ok(row.get(0)?),
    )?;

    match repo.last() {
        Some(x) => x,
        None => Err(rusqlite::Error::InvalidQuery),
    }
}

pub fn insert_file(
    name: &str,
    xx_hash: u64,
    repo_id: i64,
    parent_id: i64,
    conn: &Connection,
) -> Result<()> {
    let xx = xx_hash.to_string();
    println!("{:?}", &xx.as_str());
    conn.execute(
        "INSERT INTO file \
         (id, name, xxHash64, repository_id, parent_id) \
         VALUES (NULL, ?1, ?2, ?3, ?4)",
        &[
            name,
            &xx.as_str(),
            repo_id.to_string().as_str(),
            parent_id.to_string().as_str(),
        ],
    )?;

    Ok(())
}

pub fn insert_folder(
    name: &str,
    repo_id: i64,
    parent_id: Option<i64>,
    conn: &Connection,
) -> Result<()> {
    let is_root: bool = parent_id.is_none();
    let parent_id: i64 = parent_id.unwrap_or(0);
    conn.execute(
        "INSERT INTO folder \
         (id, name, is_root, repository_id, parent_id) \
         VALUES (NULL, ?1, ?2, ?3, ?4)",
        &[
            name,
            is_root.to_string().as_str(),
            repo_id.to_string().as_str(),
            parent_id.to_string().as_str(),
        ],
    )?;

    Ok(())
}

pub fn insert_repository(name: &str, path: &str, url: &str, conn: &mut Connection) -> Result<()> {
    create(conn)?;

    conn.execute(
        "INSERT INTO repositories \
         (id, name, path, url) \
         VALUES (NULL, ?1, ?2, ?3)",
        &[name, path, url],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub url: String,
}

pub fn get_repository(name: &str, mut conn: &mut Connection) -> Result<Repository> {
    create(&mut conn)?;

    let mut stmt =
        conn.prepare("SELECT id, name, path, url FROM repositories WHERE name = ?1 LIMIT 1")?;

    let repo = stmt.query_map(&[name], |row| {
        Ok(Repository {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            url: row.get(3)?,
        })
    })?;

    match repo.last() {
        Some(x) => x,
        None => Err(rusqlite::Error::InvalidQuery),
    }
}

#[derive(Debug)]
pub struct RFolder {
    pub id: i64,
    pub name: String,
    pub is_root: bool,
    pub parent_id: i64,
}

pub fn get_repo_folders(repo_id: i64, conn: &mut Connection) -> Result<Vec<RFolder>> {
    let mut stmt =
        conn.prepare("SELECT id, name, is_root, parent_id FROM folder WHERE repository_id = ?1")?;

    let folders = stmt.query_map(&[repo_id], |row| {
        let isroot: String = row.get(2)?;

        Ok(RFolder {
            id: row.get(0)?,
            name: row.get(1)?,
            is_root: isroot.ends_with("true"),
            parent_id: row.get(3)?,
        })
    })?;

    let _repo_folders: Vec<RFolder> = folders.map(|f| f.unwrap()).collect();

    Ok(_repo_folders)
}

#[derive(Debug)]
pub struct RFile {
    pub id: i64,
    pub name: String,
    pub xx_hash64: String,
    pub parent_id: i64,
}

pub fn get_repo_files(repo_id: i64, conn: &mut Connection) -> Result<Vec<RFile>> {
    let mut stmt =
        conn.prepare("SELECT id, name, xxHash64, parent_id FROM file WHERE repository_id = ?1")?;

    let files = stmt.query_map(&[repo_id], |row| {
        Ok(RFile {
            id: row.get(0)?,
            name: row.get(1)?,
            xx_hash64: row.get(2)?,
            parent_id: row.get(3)?,
        })
    })?;

    let _repo_files: Vec<RFile> = files.map(|f| f.unwrap()).collect();

    Ok(_repo_files)
}
