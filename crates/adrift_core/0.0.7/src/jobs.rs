use dotenv::dotenv;
use job_queue::{Client, Job};
use std::env;
use std::sync::OnceLock;

static QUEUE: OnceLock<Client> = OnceLock::new();

pub struct Queue;

impl Queue {
    pub async fn dispatch(task: &impl Job) -> anyhow::Result<()> {
        let queue = get_queue().await?;

        queue.dispatch(task).await?;

        Ok(())
    }
}

async fn get_queue() -> anyhow::Result<Client> {
    match QUEUE.get() {
        Some(client) => Ok(client.clone()),
        None => {
            dotenv().ok();

            let database_url = env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("Failed to get DATABASE_URL"))?;

            let queue = Client::builder()
                .connect(&database_url)
                .await
                .map_err(|_| anyhow::anyhow!("Failed to connect to database"))?;

            let _ = QUEUE.set(queue);

            match QUEUE.get() {
                Some(client) => Ok(client.clone()),
                None => Err(anyhow::anyhow!("Failed to get queue")),
            }
        }
    }
}