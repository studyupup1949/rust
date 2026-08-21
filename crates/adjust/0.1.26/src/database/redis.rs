use diesel::r2d2;

use super::{Pool, PoolBuilder};

pub type Redis = redis::Client;

impl PoolBuilder<Redis> for Redis {
  fn create_pool() -> anyhow::Result<Pool<Redis>> {
    let redis_url = std::env::var("REDIS_URL")?;
    let manager = redis::Client::open(redis_url)?;

    Ok(r2d2::Pool::builder().build(manager)?.into())
  }
}