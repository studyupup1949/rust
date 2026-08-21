use absurd_future::absurd_future;
use anyhow::{bail, Result};
use std::{convert::Infallible, time::Duration};

use tokio::task::JoinSet;

#[tokio::main]
async fn main() -> Result<()> {
    let _result = main_inner().await?;
    Ok(())
}

async fn task_one() -> Infallible {
    loop {
        println!("Hello from task 1");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn task_two() -> Result<Infallible> {
    let mut counter = 1;
    loop {
        println!("Hello from task 2");
        counter += 1;
        tokio::time::sleep(Duration::from_secs(1)).await;
        if counter >= 3 {
            bail!("Counter is >= 3")
        }
    }
}

async fn main_inner() -> Result<Infallible> {
    let mut join_set = JoinSet::new();

    join_set.spawn(absurd_future(task_one()));
    join_set.spawn(task_two());

    match join_set.join_next().await {
        Some(result) => match result {
            Ok(res) => match res {
                Ok(_res) => bail!("Impossible: Infallible witnessed!"),
                Err(e) => {
                    join_set.abort_all();
                    bail!("Task exited with {e}")
                }
            },
            Err(e) => {
                join_set.abort_all();
                bail!("Task exited with {e}")
            }
        },
        None => {
            join_set.abort_all();
            bail!("No tasks found in task set")
        }
    }
}
