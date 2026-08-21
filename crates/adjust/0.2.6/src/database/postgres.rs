use diesel::{r2d2::{self, ConnectionManager}, PgConnection};
use super::{Pool, PoolBuilder};

pub type Postgres = ConnectionManager<PgConnection>;

impl PoolBuilder<Postgres> for Postgres {
  fn create_pool() -> anyhow::Result<Pool<Postgres>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Ok(r2d2::Pool::builder().build(manager)?.into())
  }
}