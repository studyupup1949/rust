use std::future::Future;
use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// An operation can only fail if the director has been closed
#[derive(Debug)]
pub struct DirectorError(());

impl DirectorError {
    fn closed() -> DirectorError {
        DirectorError(())
    }
}

/// Adds ability to spawn tasks with a restricted number of tasks.
///
/// Example:
/// ```rust
/// use adirector::{Director, DirectorError};
/// async fn main() -> Result<(), DirectorError> {
///     let mut director = Director::new(10);
///     for _ in 0..100 {
///         director.spawn(async move { println!("go wild!") }).await?;
///     }
///     director.join_all().await;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Director {
    sem: Arc<Semaphore>,
    set: JoinSet<()>,
}

impl Director {
    /// Creates an instance of Director with the provided number of maximum tasks
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adirector::Director;
    ///
    /// fn main() {
    ///     // (an immutable director is useless)
    ///     let director = Director::new(10);
    /// }
    /// ```
    pub fn new(size: usize) -> Director {
        let sem = Arc::new(Semaphore::new(size));
        let set = JoinSet::new();
        Director { sem, set }
    }

    /// Suspends until the task is spawned
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adirector::{Director, DirectorError};
    ///
    /// async fn main() -> Result<(), DirectorError>  {
    ///     let mut director = Director::new(10);
    ///     director.spawn(async move { println!("go wild!") }).await?;
    ///     director.join_all().await;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This method panics if called outside a Tokio runtime.
    pub async fn spawn<F>(&mut self, task: F) -> Result<(), DirectorError>
    where
        F: Future<Output = ()>,
        F: Send + 'static,
    {
        let permit = self
            .sem
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| DirectorError::closed())?;

        self.set.spawn(async move {
            task.await;
            drop(permit);
        });

        Ok(())
    }

    /// Closes the director and wait for all tasks to be completed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::convert::Infallible;
    ///
    /// use adirector::Director;
    ///
    /// async fn main() -> Result<(), Infallible> {
    ///     let mut director = Director::new(10);
    ///     // ...
    ///     director.join_all().await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn join_all(mut self) {
        self.sem.close();

        while let Some(handle) = self.set.join_next().await {
            handle.unwrap();
        }
    }
}
