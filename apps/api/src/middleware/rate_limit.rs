use dashmap::DashMap;
use governor::{
    Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::direct::NotKeyed,
};
use std::num::NonZeroU32;
use std::sync::Arc;

/// A per-user rate limiter that uses a Token Bucket algorithm.
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    limiters: Arc<DashMap<String, Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
    quota: Quota,
}

impl UserRateLimiter {
    /// Create a new limiter with a specific quota (e.g. 5 requests per minute with a burst of 10).
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
        let limiter = self
            .limiters
            .entry(user_id.to_owned())
            .or_insert_with(|| Arc::new(RateLimiter::direct(self.quota)))
            .value()
            .clone();

        limiter.check().is_ok()
    }
}
