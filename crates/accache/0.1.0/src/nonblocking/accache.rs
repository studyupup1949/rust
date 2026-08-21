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
use crate::rpc::nonblocking::RpcFetcher;
use crate::source::{AccacheBuilder, Source};

#[derive(Debug, Default)]
struct AccacheInner {
    entries: IndexMap<Pubkey, (Account, Option<u64>)>,
}

/// Async variant of [`crate::Accache`]. Same surface, `async fn`s.
#[derive(Clone)]
pub struct Accache {
    inner: Arc<RwLock<AccacheInner>>,
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

        let rpc = rpc_url
            .as_ref()
            .map(|url| Arc::new(RpcFetcher::new(url.clone(), config.commitment)));

        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
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

    pub fn try_get(&self, key: &Pubkey) -> Option<KeyedAccount> {
        self.inner
            .read()
            .entries
            .get(key)
            .map(|(a, _)| KeyedAccount::new(*key, a.clone()))
    }

    pub fn insert(&self, key: Pubkey, account: Account) {
        self.inner
            .write()
            .entries
            .insert(key, (account, Some(now_unix_ms())));
        if self.config.auto_persist {
            if let Err(e) = self.flush() {
                log::warn!("auto-persist failed: {e}");
            }
        }
    }

    pub fn remove(&self, key: &Pubkey) -> Option<KeyedAccount> {
        let res = self
            .inner
            .write()
            .entries
            .shift_remove(key)
            .map(|(a, _)| KeyedAccount::new(*key, a));
        if res.is_some() && self.config.auto_persist {
            let _ = self.flush();
        }
        res
    }

    pub async fn get(&self, key: &Pubkey) -> Result<KeyedAccount> {
        if let Some(entry) = self.cache_lookup_if_fresh(key) {
            return Ok(entry);
        }
        match self.config.refresh {
            RefreshPolicy::Offline => Err(AccacheError::Offline(*key)),
            _ => {
                let account = self
                    .rpc
                    .as_ref()
                    .ok_or(AccacheError::NoRpcConfigured)?
                    .fetch(key)
                    .await?
                    .ok_or(AccacheError::NotFound(*key))?;
                self.inner
                    .write()
                    .entries
                    .insert(*key, (account.clone(), Some(now_unix_ms())));
                if self.config.auto_persist {
                    let _ = self.flush();
                }
                Ok(KeyedAccount::new(*key, account))
            }
        }
    }

    pub async fn get_multiple(&self, keys: &[Pubkey]) -> Result<Vec<KeyedAccount>> {
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
            let fetched = self
                .rpc
                .as_ref()
                .ok_or(AccacheError::NoRpcConfigured)?
                .fetch_multiple(&pks)
                .await?;
            let now = now_unix_ms();
            let mut g = self.inner.write();
            for ((i, key), maybe) in to_fetch.into_iter().zip(fetched.into_iter()) {
                let account = maybe.ok_or(AccacheError::NotFound(key))?;
                g.entries.insert(key, (account.clone(), Some(now)));
                out[i] = Some(KeyedAccount::new(key, account));
            }
            drop(g);
            if self.config.auto_persist {
                let _ = self.flush();
            }
        }

        Ok(out
            .into_iter()
            .map(|o| o.expect("all slots filled"))
            .collect())
    }

    pub async fn refresh(&self, key: &Pubkey) -> Result<KeyedAccount> {
        let account = self
            .rpc
            .as_ref()
            .ok_or(AccacheError::NoRpcConfigured)?
            .fetch(key)
            .await?
            .ok_or(AccacheError::NotFound(*key))?;
        self.inner
            .write()
            .entries
            .insert(*key, (account.clone(), Some(now_unix_ms())));
        if self.config.auto_persist {
            let _ = self.flush();
        }
        Ok(KeyedAccount::new(*key, account))
    }

    pub async fn refresh_all(&self) -> Result<()> {
        let keys = self.keys();
        if keys.is_empty() {
            return Ok(());
        }
        let fetched = self
            .rpc
            .as_ref()
            .ok_or(AccacheError::NoRpcConfigured)?
            .fetch_multiple(&keys)
            .await?;
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
        if self.config.auto_persist {
            let _ = self.flush();
        }
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        match &self.config.outfile {
            Some(path) => self.write_to(path),
            None => Ok(()),
        }
    }

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
                None => true,
                Some(t) => is_within_ttl(*t, d),
            },
        };
        if fresh {
            Some(KeyedAccount::new(*key, account.clone()))
        } else {
            None
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
