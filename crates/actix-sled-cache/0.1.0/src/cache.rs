use actix::{Actor, AsyncContext, Context, Handler, Recipient};
use chrono::{DateTime, Utc};
use futures::Future;
use log::{error, info};
use sled_extensions::{
    expiring::{Tree, TreeBuilder},
    Db, DbExt, Encoding, IVec,
};
use std::{collections::HashSet, sync::Arc, time::Duration};

use crate::{error::Error, Deleted, Subscribe};

/// A cache backed by Sled and Actix, storing data via the Encoding trait from sled_extensions
pub struct Cache<V, E, F> {
    cache: Tree<V, E, F>,
    frequency: Duration,
    perform_delete: bool,
    subscribers: Vec<Recipient<Deleted>>,
}

impl<V, E, F> Cache<V, E, F>
where
    V: Clone + Send + 'static,
    E: Clone + Encoding<HashSet<IVec>> + Encoding<DateTime<Utc>> + Send + 'static,
    F: Clone + Encoding<V> + Send + 'static,
{
    /// Access the tree used for storage
    pub fn tree(&self) -> Tree<V, E, F> {
        self.cache.clone()
    }

    /// Start building a new cache
    pub fn builder(db: Db, name: &str) -> CacheBuilder<V, E, F> {
        CacheBuilder::new(db, name)
    }
}

impl<V, E, F> Actor for Cache<V, E, F>
where
    V: Clone + Send + 'static,
    E: Clone + Encoding<HashSet<IVec>> + Encoding<DateTime<Utc>> + Send + 'static,
    F: Clone + Encoding<V> + Send + 'static,
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(self.frequency, move |actor, _| {
            let cache = actor.cache.clone();
            let subscribers = actor.subscribers.clone();
            let perform_delete = actor.perform_delete;

            actix::spawn(
                actix_threadpool::run(move || {
                    let expired = cache.clone().expired().count();
                    info!(
                        "Notifying for {} expired records of {} total records",
                        expired,
                        cache.len()
                    );

                    for key in cache.clone().expired() {
                        if perform_delete {
                            cache.remove(key.clone())?;
                        }

                        let key: Arc<[u8]> = key.into();
                        for subscribers in subscribers.clone() {
                            subscribers.do_send(Deleted(key.clone()))?;
                        }
                    }
                    Ok(()) as Result<(), Error>
                })
                .from_err()
                .map(|_| ())
                .map_err(|e: Error| error!("Error in delete, {}", e)),
            );
        });
    }
}

impl<V, E, F> Handler<Subscribe> for Cache<V, E, F>
where
    V: Clone + Send + 'static,
    E: Clone + Encoding<HashSet<IVec>> + Encoding<DateTime<Utc>> + Send + 'static,
    F: Clone + Encoding<V> + Send + 'static,
{
    type Result = ();

    fn handle(&mut self, Subscribe(recipient): Subscribe, _: &mut Self::Context) {
        self.subscribers.push(recipient);
    }
}

/// A builder for caches
pub struct CacheBuilder<V, E, F> {
    tree_builder: TreeBuilder<V, E, F>,
    frequency: Duration,
    perform_delete: bool,
    subscribers: Vec<Recipient<Deleted>>,
}

impl<V, E, F> CacheBuilder<V, E, F>
where
    E: Encoding<HashSet<IVec>> + Encoding<DateTime<Utc>> + 'static,
    F: Encoding<V> + 'static,
{
    /// Create a new builder
    pub fn new(db: Db, name: &str) -> Self {
        CacheBuilder {
            tree_builder: db.open_expiring_tree(name),
            frequency: Duration::from_secs(60 * 5),
            perform_delete: true,
            subscribers: Vec::new(),
        }
    }

    /// Change the frequency at which the cache is checked for expired records
    pub fn frequency(&mut self, duration: Duration) -> &mut Self {
        self.frequency = duration;
        self
    }

    /// Add an actor as a recipient of expiry events
    pub fn subscribe(&mut self, recipient: Recipient<Deleted>) -> &mut Self {
        self.subscribers.push(recipient);
        self
    }

    /// Tell the cache not to delete records on it's own
    ///
    /// This is likely useful if a subscribed actor needs access to the data before it is
    /// removed.
    pub fn no_delete(&mut self) -> &mut Self {
        self.perform_delete = false;
        self
    }

    /// Create the cache from the builder
    pub fn build(&self) -> Result<Cache<V, E, F>, Error> {
        Ok(Cache {
            cache: self.tree_builder.build()?,
            frequency: self.frequency,
            perform_delete: self.perform_delete,
            subscribers: self.subscribers.clone(),
        })
    }
}

impl<V, E, F> AsMut<TreeBuilder<V, E, F>> for CacheBuilder<V, E, F> {
    fn as_mut(&mut self) -> &mut TreeBuilder<V, E, F> {
        &mut self.tree_builder
    }
}
