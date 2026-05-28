use dashmap::DashMap;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;

/// Per-user token-bucket limiter (governor's direct in-memory variant).
type UserLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// How long an entry can sit idle before the cleanup task drops it. Picked to
/// comfortably exceed the longest quota window (per-minute today).
const IDLE_EVICTION_SECS: u64 = 3_600; // 1 hour

/// How often the cleanup task sweeps the map.
const CLEANUP_INTERVAL_SECS: u64 = 300; // 5 minutes

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Map entry pairing the limiter with a last-seen timestamp used for eviction.
struct Entry {
    limiter: Arc<UserLimiter>,
    last_seen: AtomicU64,
}

/// A per-user rate limiter that uses a Token Bucket algorithm.
///
/// Idle entries are dropped by a background cleanup task started via
/// [`UserRateLimiter::spawn_cleanup_task`] so the map size tracks active users
/// rather than growing monotonically with everyone who ever made a request.
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    limiters: Arc<DashMap<String, Entry>>,
    quota: Quota,
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry")
            .field("last_seen", &self.last_seen.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl UserRateLimiter {
    /// Create a new limiter with a specific quota (e.g. 5 requests per minute with a burst of 10).
    #[must_use]
    pub fn new(requests_per_minute: u32, burst: u32) -> Self {
        let rpm =
            NonZeroU32::new(requests_per_minute).unwrap_or_else(|| NonZeroU32::new(60).unwrap());
        let b = NonZeroU32::new(burst).unwrap_or_else(|| NonZeroU32::new(10).unwrap());
        Self {
            limiters: Arc::new(DashMap::new()),
            quota: Quota::per_minute(rpm).allow_burst(b),
        }
    }

    /// Check if a user is allowed to perform an action.
    /// Returns true if allowed, false if rate limited.
    pub fn check(&self, user_id: &str) -> bool {
        let now = now_secs();
        // Get or create the entry, refresh last_seen, and release the shard
        // lock before performing the (cheap, sync) rate-limit check so we
        // don't serialise high-frequency callers on the DashMap shard.
        let limiter = {
            let entry = self
                .limiters
                .entry(user_id.to_owned())
                .or_insert_with(|| Entry {
                    limiter: Arc::new(RateLimiter::direct(self.quota)),
                    last_seen: AtomicU64::new(now),
                });
            entry.last_seen.store(now, Ordering::Relaxed);
            entry.limiter.clone()
        };

        limiter.check().is_ok()
    }

    /// Spawn a background task that periodically evicts limiters idle for more
    /// than [`IDLE_EVICTION_SECS`]. Shuts down when `shutdown` is cancelled.
    /// Safe to call at most once per instance; calling twice would just create
    /// two cleanup tasks doing the same work.
    pub fn spawn_cleanup_task(&self, shutdown: CancellationToken) {
        let limiters = Arc::clone(&self.limiters);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let cutoff = now_secs().saturating_sub(IDLE_EVICTION_SECS);
                        let before = limiters.len();
                        limiters.retain(|_, e| e.last_seen.load(Ordering::Relaxed) > cutoff);
                        let evicted = before.saturating_sub(limiters.len());
                        if evicted > 0 {
                            tracing::info!("🧹 Evicted {evicted} idle rate limiters (now tracking {})", limiters.len());
                        }
                    }
                    _ = shutdown.cancelled() => {
                        tracing::info!("🛑 Rate limiter cleanup task shutting down");
                        break;
                    }
                }
            }
        });
    }

    /// Current number of tracked users. Primarily for tests/metrics.
    #[must_use]
    pub fn tracked_count(&self) -> usize {
        self.limiters.len()
    }
}
