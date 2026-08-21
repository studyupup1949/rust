use std::{collections::HashMap, env};

use crate::commands::{Arg, Command};

pub struct QueueWork;

#[async_trait::async_trait]
impl Command for QueueWork {
    fn name(&self) -> &'static str {
        "queue:work"
    }

    fn description(&self) -> &'static str {
        "Start processing jobs on the queue as a daemon"
    }

    fn args(&self) -> Vec<Arg> {
        vec![]
    }

    async fn handle(&self, _args: HashMap<String, String>) -> anyhow::Result<()> {
        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("Failed to get DATABASE_URL"))?;

        let worker = job_queue::Worker::builder()
            .max_connections(10)
            .worker_count(4)
            .connect(&database_url)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to connect to database"))?;

        worker.start().await?;

        Ok(())
    }
}
