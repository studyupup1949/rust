pub mod data;
mod db;
pub mod query;
mod store;

use std::error::Error;

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use crate::{ActflowError, Result};

use query::*;

pub use db::{MemStore, PostgresStore};
pub use store::Store;

fn map_db_err(err: impl Error) -> ActflowError {
    ActflowError::Store(err.to_string())
}

#[derive(Debug, Clone, AsRefStr, PartialEq, Hash, Eq, EnumIter)]
pub enum StoreIden {
    #[strum(serialize = "workflows")]
    Workflows,
    #[strum(serialize = "events")]
    Events,
    #[strum(serialize = "procs")]
    Procs,
    #[strum(serialize = "nodes")]
    Nodes,
    #[strum(serialize = "logs")]
    Logs,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PageData<T> {
    pub count: usize,
    pub page_num: usize,
    pub page_count: usize,
    pub page_size: usize,
    pub rows: Vec<T>,
}

pub trait DbCollectionIden {
    fn iden() -> StoreIden;
}

pub trait DbCollection: Send + Sync {
    type Item;
    fn exists(
        &self,
        id: &str,
    ) -> Result<bool>;
    fn find(
        &self,
        id: &str,
    ) -> Result<Self::Item>;
    fn query(
        &self,
        query: &Query,
    ) -> Result<PageData<Self::Item>>;
    fn create(
        &self,
        data: &Self::Item,
    ) -> Result<bool>;
    fn update(
        &self,
        data: &Self::Item,
    ) -> Result<bool>;
    fn delete(
        &self,
        id: &str,
    ) -> Result<bool>;
}

pub trait DbStore {
    fn init(
        &self,
        s: &Store,
    );
}
