extern crate rusqlite;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

fn create() -> Result<()> {
    let conn = Connection::open("a3mm.sqlite3")?;
    conn.execute(
        "create table if not exists repositories (\
         id integer primary key,\
         name text not null unique,\
         path text not null,\
         url text not null,\
         delta_patch integer);",
        NO_PARAMS,
    )?;
    Ok(())
}

pub fn insert_repository(name: &str, path: &str, url: &str, delta_patch: bool) -> Result<()> {
    create()?;

    let conn = Connection::open("a3mm.sqlite3")?;
    conn.execute(
        "INSERT INTO repositories \
         (id, name, path, url, delta_patch) \
         VALUES (NULL, ?1, ?2, ?3, ?4)",
        &[name, path, url, delta_patch.to_string().as_str()],
    )?;

    Ok(())
}

#[derive(Debug)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub url: String,
    pub delta_patch: bool,
}

pub fn get_repository(name: &str) -> Result<Repository> {
    let conn = Connection::open("a3mm.sqlite3")?;
    let mut stmt = conn.prepare(
        "SELECT id, name, path, url, delta_patch FROM repositories WHERE name = ?1 LIMIT 1",
    )?;

    let repo = stmt.query_map(&[name], |row| {
        let d: String = row.get(4)?;

        let df = d == "True".to_string();

        Ok(Repository {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            url: row.get(3)?,
            delta_patch: df,
        })
    })?;

    match repo.last() {
        Some(x) => x,
        None => Err(rusqlite::Error::InvalidQuery),
    }
}
