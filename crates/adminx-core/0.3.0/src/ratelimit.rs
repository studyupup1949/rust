// adminx-core/src/ratelimit.rs
//
// Fixed-window throttling for the credential endpoints, keyed by account.
//
// Without this, `/login` accepts unbounded password guesses and `/mfa/verify`
// unbounded TOTP guesses — and a 6-digit code is only a million combinations,
// which is minutes of work at HTTP speed. A throttle is what makes the second
// factor worth having.
//
// Two deliberate limits on what this can do:
//
// 1. **Per-process.** The counters live in this process's memory, so N replicas
//    behind a load balancer allow N times the attempts. That still bounds the
//    attack (and matches how `AuthConfig` is set up per-process today), but a
//    multi-replica deployment wanting a hard global bound needs a shared store.
//
// 2. **Keyed by account, not by client.** `ReqCtx` carries no client address —
//    and deriving one from `X-Forwarded-For` without knowing the proxy in front
//    would be worse than nothing, since the header is attacker-settable. So this
//    stops a *targeted* brute force against one account, but not credential
//    stuffing spread thinly across many accounts.
//
// The flip side of per-account keying is that an attacker can deliberately
// burn an admin's attempts to keep them out. That's why this throttles on a
// short fixed window that a successful login clears, rather than latching the
// account until someone intervenes.

use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::RwLock;

/// How many failures are tolerated in a window before the key is throttled.
#[derive(Clone, Debug)]
pub struct Limit {
    pub max_attempts: u32,
    pub window_secs: i64,
}

impl Limit {
    pub const fn new(max_attempts: u32, window_secs: i64) -> Self {
        Self {
            max_attempts,
            window_secs,
        }
    }
}

/// Password guessing: generous enough that a fat-fingered admin won't notice.
pub const DEFAULT_LOGIN_LIMIT: Limit = Limit::new(10, 900);
/// TOTP guessing: tighter, because the search space is only 10^6. At 5 per 15
/// minutes, exhausting it would take roughly 5,000 years.
pub const DEFAULT_MFA_LIMIT: Limit = Limit::new(5, 900);

/// Cap on tracked keys, so a flood of distinct emails can't grow the map without
/// bound. Expired entries are pruned first; see `record_failure_at`.
const MAX_TRACKED_KEYS: usize = 10_000;

/// Which throttles are active. Unlike `AuthConfig` this is on by default: an
/// operator who never calls `configure` still gets protected endpoints.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Failed password attempts per account. `None` allows unlimited guessing.
    pub login: Option<Limit>,
    /// Failed second-factor attempts per account. `None` makes a 6-digit code
    /// brute-forceable.
    pub mfa: Option<Limit>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            login: Some(DEFAULT_LOGIN_LIMIT),
            mfa: Some(DEFAULT_MFA_LIMIT),
        }
    }
}

static CONFIG: OnceCell<RateLimitConfig> = OnceCell::new();

/// Override the default throttles. Call before serving: the first login or MFA
/// attempt locks the defaults in, and a later call is ignored with a warning.
pub fn configure(config: RateLimitConfig) {
    if CONFIG.set(config).is_err() {
        tracing::warn!("adminx rate limits already configured; ignoring reconfigure");
    }
}

fn config() -> &'static RateLimitConfig {
    CONFIG.get_or_init(RateLimitConfig::default)
}

/// The active password-attempt throttle, if any.
pub fn login_limit() -> Option<&'static Limit> {
    config().login.as_ref()
}

/// The active second-factor throttle, if any.
pub fn mfa_limit() -> Option<&'static Limit> {
    config().mfa.as_ref()
}

#[derive(Clone, Debug)]
struct Window {
    count: u32,
    /// Wall-clock second at which this window lapses and the count resets.
    expires_at: i64,
}

lazy_static! {
    static ref FAILURES: RwLock<HashMap<String, Window>> = RwLock::new(HashMap::new());
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// True when `key` has already used up its attempts and the window hasn't
/// lapsed. Read-only: checking never costs an attempt.
fn is_limited_at(key: &str, limit: &Limit, now: i64) -> bool {
    match FAILURES.read().unwrap().get(key) {
        Some(w) if w.expires_at > now => w.count >= limit.max_attempts,
        _ => false,
    }
}

/// Count one failure against `key`, returning the running total for the window.
/// The window is anchored at the *first* failure and not extended by later ones,
/// so a throttled caller is always let back in after `window_secs`.
fn record_failure_at(key: &str, limit: &Limit, now: i64) -> u32 {
    let mut map = FAILURES.write().unwrap();

    if map.len() >= MAX_TRACKED_KEYS {
        map.retain(|_, w| w.expires_at > now);
    }

    let entry = map.entry(key.to_string()).or_insert(Window {
        count: 0,
        expires_at: now + limit.window_secs,
    });

    // A lapsed window starts over rather than accumulating forever.
    if entry.expires_at <= now {
        entry.count = 0;
        entry.expires_at = now + limit.window_secs;
    }
    entry.count += 1;
    entry.count
}

/// True when `key` is currently throttled.
pub fn is_limited(key: &str, limit: &Limit) -> bool {
    is_limited_at(key, limit, now_secs())
}

/// Count one failed attempt against `key`.
pub fn record_failure(key: &str, limit: &Limit) {
    record_failure_at(key, limit, now_secs());
}

/// Forget `key`'s failures. Called on a successful authentication so a user who
/// eventually gets it right isn't punished for the fumbles along the way.
pub fn reset(key: &str) {
    FAILURES.write().unwrap().remove(key);
}

/// Drop all counters. Test-only: the map is process-global, so tests that assert
/// on counts need a clean slate.
#[doc(hidden)]
pub fn clear_all() {
    FAILURES.write().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    /// Time is injected via the `_at` seam so window expiry is tested outright
    /// rather than by sleeping.
    const T0: i64 = 1_000_000;

    lazy_static! {
        static ref TEST_LOCK: Mutex<()> = Mutex::new(());
    }

    /// `FAILURES` is process-global, so these tests would otherwise clobber each
    /// other's counters when cargo runs them in parallel. Take the lock, then
    /// start from a clean map.
    fn isolated() -> MutexGuard<'static, ()> {
        // A test that panics mid-assert poisons the lock; the state it guards is
        // reset on entry anyway, so recover rather than cascade the failure.
        let guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_all();
        guard
    }

    #[test]
    fn throttles_only_after_the_limit_is_reached() {
        let _g = isolated();
        let limit = Limit::new(3, 60);
        let key = "throttles-only-after";

        assert!(!is_limited_at(key, &limit, T0), "clean key is not limited");
        assert_eq!(record_failure_at(key, &limit, T0), 1);
        assert_eq!(record_failure_at(key, &limit, T0), 2);
        assert!(!is_limited_at(key, &limit, T0), "under the limit, still allowed");

        assert_eq!(record_failure_at(key, &limit, T0), 3);
        assert!(is_limited_at(key, &limit, T0), "at the limit, throttled");
    }

    #[test]
    fn window_lapses_and_the_count_starts_over() {
        let _g = isolated();
        let limit = Limit::new(2, 60);
        let key = "window-lapses";

        record_failure_at(key, &limit, T0);
        record_failure_at(key, &limit, T0);
        assert!(is_limited_at(key, &limit, T0));

        // Still inside the window.
        assert!(is_limited_at(key, &limit, T0 + 59));
        // Lapsed.
        assert!(!is_limited_at(key, &limit, T0 + 61));
        assert_eq!(
            record_failure_at(key, &limit, T0 + 61),
            1,
            "a lapsed window restarts at one rather than accumulating"
        );
    }

    #[test]
    fn window_is_anchored_at_the_first_failure() {
        let _g = isolated();
        let limit = Limit::new(2, 60);
        let key = "anchored";

        record_failure_at(key, &limit, T0);
        // A later failure inside the window must not push the expiry out, or a
        // steady drip of attempts would extend a lockout indefinitely.
        record_failure_at(key, &limit, T0 + 50);
        assert!(is_limited_at(key, &limit, T0 + 50));
        assert!(
            !is_limited_at(key, &limit, T0 + 61),
            "expiry stays anchored to the first failure"
        );
    }

    #[test]
    fn success_clears_the_count() {
        let _g = isolated();
        let limit = Limit::new(2, 60);
        let key = "success-clears";

        record_failure_at(key, &limit, T0);
        record_failure_at(key, &limit, T0);
        assert!(is_limited_at(key, &limit, T0));

        reset(key);
        assert!(!is_limited_at(key, &limit, T0), "reset lifts the throttle");
    }

    #[test]
    fn keys_are_tracked_independently() {
        let _g = isolated();
        let limit = Limit::new(1, 60);
        record_failure_at("alice", &limit, T0);
        assert!(is_limited_at("alice", &limit, T0));
        assert!(
            !is_limited_at("bob", &limit, T0),
            "one account's failures must not throttle another"
        );
    }

    #[test]
    fn expired_entries_are_pruned_under_pressure() {
        let _g = isolated();
        let limit = Limit::new(1, 60);

        // Fill past the cap with entries that all lapse at T0 + 60.
        for i in 0..MAX_TRACKED_KEYS {
            record_failure_at(&format!("key-{i}"), &limit, T0);
        }
        assert_eq!(FAILURES.read().unwrap().len(), MAX_TRACKED_KEYS);

        // A later write past the cap sweeps the lapsed ones out.
        record_failure_at("newcomer", &limit, T0 + 61);
        let len = FAILURES.read().unwrap().len();
        assert!(
            len < MAX_TRACKED_KEYS,
            "expired entries should be pruned, still holding {len}"
        );
        assert!(is_limited_at("newcomer", &limit, T0 + 61));
    }
}
