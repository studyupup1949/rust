use std::{collections::HashMap, hash::Hash};

use nanoserde::DeJson;
use time::OffsetDateTime;

mod tests;

/// A trait bound that limits what types can be used in an ActionIDMap.
pub trait Mappable: DeJson + Eq + Hash {}
impl<T> Mappable for T where T: DeJson + Eq + Hash {}

/// The main struct that contains the map, the timestamp that the map was last updated,
/// the maximum amount of time that the map can be cached, and the URL to the map.
pub struct ActionIDMap<X: Mappable, Y: Mappable> {
    last_updated: i64,
    max_cache_duration: i64,
    url: String,
    map: HashMap<X, Y>,
}

impl<X: Mappable, Y: Mappable> ActionIDMap<X, Y> {
    /// Construct a new ActionIDMap with the given URL and max cache duration
    pub fn new(url: String, max_cache_duration: i64) -> Option<Self> {
        let req = ureq::get(&url).call().ok()?.into_string().ok()?;
        let hashmap = nanoserde::DeJson::deserialize_json(&req).ok()?;
        Some(Self {
            // last_updated should be the current UNIX timestamp
            last_updated: OffsetDateTime::now_utc().unix_timestamp(),
            max_cache_duration,
            url,
            map: hashmap,
        })
    }
    /// Retrieve a value from the map
    pub fn get(&self, action: X) -> Option<&Y> {
        self.map.get(&action)
    }
    /// Force a refresh of the map
    pub fn refresh(&mut self) -> Option<()> {
        let req = ureq::get(&self.url).call().ok()?.into_string().ok()?;
        self.map = nanoserde::DeJson::deserialize_json(&req).ok()?;
        self.last_updated = OffsetDateTime::now_utc().unix_timestamp();
        Some(())
    }
    /// Check if the map needs to be refreshed
    pub fn needs_refresh(&self) -> bool {
        self.last_updated + self.max_cache_duration < OffsetDateTime::now_utc().unix_timestamp()
    }
}
