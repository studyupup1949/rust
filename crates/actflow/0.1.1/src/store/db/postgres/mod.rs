use sqlx::{Error as DbError, postgres::PgRow};

mod collection;
mod database;
mod synclient;

pub use database::PostgresStore;

pub trait DbRow {
    fn id(&self) -> &str;
    fn from_row(row: &PgRow) -> std::result::Result<Self, DbError>
    where
        Self: Sized;
}

pub trait DbInit {
    fn init(&self);
}
