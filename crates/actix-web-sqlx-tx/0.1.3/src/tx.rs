use scoped_futures::ScopedBoxFuture;
use sqlx::{Database, Pool, Transaction};

/// Run a callback with a transaction.
/// If the callback returns an error, the transaction is rolled back.
/// If the callback returns Ok, the transaction is committed.
pub async fn with_tx<'a, F, R, E, DB>(pool: &Pool<DB>, callback: F) -> Result<R, E>
where
    F: for<'r> FnOnce(&'r mut Transaction<DB>) -> ScopedBoxFuture<'a, 'r, Result<R, E>> + Send + 'a,
    E: From<sqlx::Error> + Send + 'a,
    R: Send + 'a,
    DB: Database,
{
    let mut tx = pool.begin().await?;
    let res = callback(&mut tx).await;
    match res {
        Ok(response) => {
            tx.commit().await?;
            Ok(response)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Tests module for the tx module
pub mod tests {
    use scoped_futures::ScopedBoxFuture;
    use sqlx::{Database, Pool, Transaction};

    /// Run a callback with a transaction. The transaction is rolled back at the end.
    pub async fn with_tx<'a, F, DB>(pool: &Pool<DB>, f: F)
    where
        F: for<'r> FnOnce(&'r mut Transaction<DB>) -> ScopedBoxFuture<'a, 'r, ()>,
        DB: Database,
    {
        let mut tx = pool.begin().await.expect("Failed to begin transaction");
        f(&mut tx).await;
        tx.rollback().await.expect("Failed to rollback transaction");
    }
}
