use adbc_core::{Driver, Optionable, error::Result, options::OptionValue};

use super::database::TaosDatabase;

#[derive(Default)]
pub struct TaosDriver {}

impl Driver for TaosDriver {
    type DatabaseType = TaosDatabase;

    fn new_database(&mut self) -> Result<Self::DatabaseType> {
        Ok(Self::DatabaseType::default())
    }

    fn new_database_with_opts(
        &mut self,
        opts: impl IntoIterator<Item = (<Self::DatabaseType as Optionable>::Option, OptionValue)>,
    ) -> Result<Self::DatabaseType> {
        let mut database = Self::DatabaseType::default();
        for (key, value) in opts {
            database.set_option(key, value)?;
        }
        Ok(database)
    }
}