use sea_orm::{ColumnTrait, FromJsonQueryResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, FromJsonQueryResult)]
pub struct VecString(pub Vec<String>);

pub trait DefaultColumnTrait {
    fn get_created_at() -> impl ColumnTrait;
    fn get_updated_at() -> impl ColumnTrait;
}
