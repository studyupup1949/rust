# adirector

asynchronous tokio task spawner with a limited size.

Full example:

```rust
use adirector::{Director, DirectorError};

#[tokio: main]
async fn main() -> Result<(), DirectorError> {
    // create executor that allow 10 tasks concurrently.
    let mut director = Director::new(10);

    // read line by line stdin
    let mut lines = BufReader::new(stdin()).lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        director.spawn(async move {
            println!("{}", line);
            sleep(Duration::from_millis(50)).await;
        }).await?; // Suspends until the task is spawned
    }

    // Wait for remaining tasks to complete
    director.join_all().await
}
```

## Implementation

A [Semaphore](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html) is used to limit the count of tasks, and
a [JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html) to join all tasks at once.

## Dependencies

* [tokio](https://docs.rs/tokio) with `rt` `sync` features
