use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use parking_lot::RwLock;
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::account::KeyedAccount;
use crate::config::{CacheConfig, RefreshPolicy};
use crate::error::{AccacheError, Result};
use crate::format::{Entry, read_file, write_file};
use crate::source::{AccacheBuilder, Source};

#[cfg(feature = "rpc")]
use crate::rpc::blocking::RpcFetcher;

#[derive(Debug, Default)]
pub(crate) struct AccacheInner {
    pub entries: IndexMap<Pubkey, (Account, Option<u64>)>, // (account, fetched_at_unix_ms)
}

/// Sync (blocking) cache. Cheap to clone via `Arc<RwLock<...>>`.
#[derive(Clone)]
pub struct Accache {
    inner: Arc<RwLock<AccacheInner>>,
    #[cfg(feature = "rpc")]
    rpc: Option<Arc<RpcFetcher>>,
    config: Arc<CacheConfig>,
    source: Source,
}

impl Accache {
    pub fn builder() -> AccacheBuilder {
        AccacheBuilder::new()
    }

    pub(crate) fn from_builder(builder: AccacheBuilder) -> Result<Self> {
        let source = builder.source();
        let (rpc_url, files, config) = builder.into_parts();

        let mut inner = AccacheInner::default();
        for path in &files {
            let (_, entries) = read_file(path)?;
            for entry in entries {
                let pk = entry.pubkey();
                let ts = entry.fetched_at_unix_ms;
                inner.entries.insert(pk, (entry.into_keyed().account, ts));
            }
        }

        #[cfg(feature = "rpc")]
        let rpc = rpc_url
            .as_ref()
            .map(|url| Arc::new(RpcFetcher::new(url.clone(), config.commitment)));
        #[cfg(not(feature = "rpc"))]
        let _ = rpc_url;

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
            #[cfg(feature = "rpc")]
            rpc,
            config: Arc::new(config),
            source,
        })
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    pub fn len(&self) -> usize {
        self.inner.read().entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.read().entries.is_empty()
    }

    pub fn keys(&self) -> Vec<Pubkey> {
        self.inner.read().entries.keys().copied().collect()
    }

    pub fn iter(&self) -> std::vec::IntoIter<KeyedAccount> {
        let g = self.inner.read();
        let v: Vec<KeyedAccount> = g
            .entries
            .iter()
            .map(|(k, (a, _))| KeyedAccount::new(*k, a.clone()))
            .collect();
        v.into_iter()
    }

    /// Cache-only peek. Returns None if the key isn't cached, regardless of policy.
    pub fn try_get(&self, key: &Pubkey) -> Option<KeyedAccount> {
        self.inner
            .read()
            .entries
            .get(key)
            .map(|(a, _)| KeyedAccount::new(*key, a.clone()))
    }

    pub fn insert(&self, key: Pubkey, account: Account) {
        self.insert_with_ts(key, account, Some(now_unix_ms()));
        self.maybe_persist();
    }

    fn insert_with_ts(&self, key: Pubkey, account: Account, ts: Option<u64>) {
        self.inner.write().entries.insert(key, (account, ts));
    }

    pub fn remove(&self, key: &Pubkey) -> Option<KeyedAccount> {
        let res = self
            .inner
            .write()
            .entries
            .shift_remove(key)
            .map(|(a, _)| KeyedAccount::new(*key, a));
        if res.is_some() {
            self.maybe_persist();
        }
        res
    }

    /// Get one account, honoring [`RefreshPolicy`].
    pub fn get(&self, key: &Pubkey) -> Result<KeyedAccount> {
        if let Some(entry) = self.cache_lookup_if_fresh(key) {
            return Ok(entry);
        }

        match self.config.refresh {
            RefreshPolicy::Offline => Err(AccacheError::Offline(*key)),
            _ => self.fetch_and_cache(key),
        }
    }

    /// Get many accounts. RPC calls are batched.
    pub fn get_multiple(&self, keys: &[Pubkey]) -> Result<Vec<KeyedAccount>> {
        let policy = self.config.refresh;
        let mut out: Vec<Option<KeyedAccount>> = vec![None; keys.len()];
        let mut to_fetch: Vec<(usize, Pubkey)> = Vec::new();

        for (i, key) in keys.iter().enumerate() {
            if let Some(entry) = self.cache_lookup_if_fresh(key) {
                out[i] = Some(entry);
            } else {
                match policy {
                    RefreshPolicy::Offline => return Err(AccacheError::Offline(*key)),
                    _ => to_fetch.push((i, *key)),
                }
            }
        }

        if !to_fetch.is_empty() {
            let pks: Vec<Pubkey> = to_fetch.iter().map(|(_, k)| *k).collect();
            let fetched = self.rpc_fetch_multiple(&pks)?;
            for ((i, key), maybe) in to_fetch.into_iter().zip(fetched.into_iter()) {
                let account = maybe.ok_or(AccacheError::NotFound(key))?;
                self.insert_with_ts(key, account.clone(), Some(now_unix_ms()));
                out[i] = Some(KeyedAccount::new(key, account));
            }
            self.maybe_persist();
        }

        Ok(out
            .into_iter()
            .map(|o| o.expect("all slots filled"))
            .collect())
    }

    /// Force a refresh of `key` from RPC, regardless of cache state.
    pub fn refresh(&self, key: &Pubkey) -> Result<KeyedAccount> {
        let account = self
            .rpc_fetch_one(key)?
            .ok_or(AccacheError::NotFound(*key))?;
        self.insert_with_ts(*key, account.clone(), Some(now_unix_ms()));
        self.maybe_persist();
        Ok(KeyedAccount::new(*key, account))
    }

    /// Force a refresh of every cached key from RPC.
    pub fn refresh_all(&self) -> Result<()> {
        let keys = self.keys();
        if keys.is_empty() {
            return Ok(());
        }
        let fetched = self.rpc_fetch_multiple(&keys)?;
        let now = now_unix_ms();
        let mut g = self.inner.write();
        for (key, maybe) in keys.into_iter().zip(fetched.into_iter()) {
            if let Some(account) = maybe {
                g.entries.insert(key, (account, Some(now)));
            } else {
                g.entries.shift_remove(&key);
            }
        }
        drop(g);
        self.maybe_persist();
        Ok(())
    }

    /// Persist to the configured outfile. No-op if no outfile is set.
    pub fn flush(&self) -> Result<()> {
        match &self.config.outfile {
            Some(path) => self.write_to(path),
            None => Ok(()),
        }
    }

    /// Persist to an explicit path.
    pub fn write_to(&self, path: &Path) -> Result<()> {
        let entries = self.snapshot_entries();
        write_file(path, &entries, self.config.compression)
    }

    fn snapshot_entries(&self) -> Vec<Entry> {
        let g = self.inner.read();
        g.entries
            .iter()
            .map(|(k, (a, ts))| Entry::from_keyed(&KeyedAccount::new(*k, a.clone()), *ts))
            .collect()
    }

    fn cache_lookup_if_fresh(&self, key: &Pubkey) -> Option<KeyedAccount> {
        let g = self.inner.read();
        let (account, ts) = g.entries.get(key)?;
        let fresh = match self.config.refresh {
            RefreshPolicy::Always => false,
            RefreshPolicy::Offline | RefreshPolicy::OnMiss => true,
            RefreshPolicy::Ttl(d) => match ts {
                None => true, // hand-built / file-loaded; treat as fresh
                Some(t) => is_within_ttl(*t, d),
            },
        };
        if fresh {
            Some(KeyedAccount::new(*key, account.clone()))
        } else {
            None
        }
    }

    fn maybe_persist(&self) {
        if self.config.auto_persist {
            if let Err(e) = self.flush() {
                log::warn!("auto-persist failed: {e}");
            }
        }
    }

    #[cfg(feature = "rpc")]
    fn rpc_fetch_one(&self, key: &Pubkey) -> Result<Option<Account>> {
        let rpc = self.rpc.as_ref().ok_or(AccacheError::NoRpcConfigured)?;
        rpc.fetch(key)
    }

    #[cfg(feature = "rpc")]
    fn rpc_fetch_multiple(&self, keys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        let rpc = self.rpc.as_ref().ok_or(AccacheError::NoRpcConfigured)?;
        rpc.fetch_multiple(keys)
    }

    #[cfg(not(feature = "rpc"))]
    fn rpc_fetch_one(&self, _key: &Pubkey) -> Result<Option<Account>> {
        Err(AccacheError::NoRpcConfigured)
    }

    #[cfg(not(feature = "rpc"))]
    fn rpc_fetch_multiple(&self, _keys: &[Pubkey]) -> Result<Vec<Option<Account>>> {
        Err(AccacheError::NoRpcConfigured)
    }

    fn fetch_and_cache(&self, key: &Pubkey) -> Result<KeyedAccount> {
        let account = self
            .rpc_fetch_one(key)?
            .ok_or(AccacheError::NotFound(*key))?;
        self.insert_with_ts(*key, account.clone(), Some(now_unix_ms()));
        self.maybe_persist();
        Ok(KeyedAccount::new(*key, account))
    }
}

impl Drop for Accache {
    fn drop(&mut self) {
        // Only flush from the *last* surviving handle. Other handles may still hold the data.
        if Arc::strong_count(&self.inner) == 1
            && self.config.auto_persist
            && self.config.outfile.is_some()
        {
            if let Err(e) = self.flush() {
                log::warn!("drop-time flush failed: {e}");
            }
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn is_within_ttl(fetched_at_ms: u64, ttl: Duration) -> bool {
    let now = now_unix_ms();
    let age_ms = now.saturating_sub(fetched_at_ms);
    age_ms < ttl.as_millis() as u64
}
