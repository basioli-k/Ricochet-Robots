//! Sliding-window rate limits, keyed by e.g. client IP.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

use domain::Timestamp;

/// Decides whether an action is allowed now; records it if so.
pub trait RateLimiter: Send + Sync {
    fn allow(&self, key: &str, now: Timestamp) -> bool;
}

/// At most `limit` actions per key within any `window_ms` milliseconds.
#[derive(Debug)]
pub struct SlidingWindow {
    limit: usize,
    window_ms: u64,
    hits: Mutex<HashMap<String, VecDeque<Timestamp>>>,
}

impl SlidingWindow {
    #[must_use]
    pub fn new(limit: usize, window_ms: u64) -> Self {
        Self {
            limit,
            window_ms,
            hits: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for SlidingWindow {
    fn allow(&self, key: &str, now: Timestamp) -> bool {
        let mut hits = self
            .hits
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Forget keys with nothing recent, so the map can't grow without bound.
        hits.retain(|_, times| {
            // A hit leaves the window once it is `window_ms` old (no
            // subtraction, which would saturate near the clock's start).
            while times
                .front()
                .is_some_and(|t| t.0.saturating_add(self.window_ms) <= now.0)
            {
                times.pop_front();
            }
            !times.is_empty()
        });
        let times = hits.entry(key.to_owned()).or_default();
        if times.len() >= self.limit {
            return false;
        }
        times.push_back(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_up_to_the_limit_per_key() {
        let limiter = SlidingWindow::new(2, 1_000);
        assert!(limiter.allow("a", Timestamp(0)));
        assert!(limiter.allow("a", Timestamp(1)));
        assert!(!limiter.allow("a", Timestamp(2)));
        assert!(limiter.allow("b", Timestamp(2)), "keys are independent");
    }

    #[test]
    fn the_window_slides() {
        let limiter = SlidingWindow::new(1, 1_000);
        assert!(limiter.allow("a", Timestamp(100)));
        assert!(!limiter.allow("a", Timestamp(1_099)));
        assert!(
            limiter.allow("a", Timestamp(1_100)),
            "the first hit has left the window"
        );
    }

    #[test]
    fn refused_attempts_do_not_extend_the_wait() {
        let limiter = SlidingWindow::new(1, 1_000);
        assert!(limiter.allow("a", Timestamp(0)));
        assert!(!limiter.allow("a", Timestamp(500)));
        assert!(limiter.allow("a", Timestamp(1_000)));
    }

    #[test]
    fn idle_keys_are_forgotten() {
        let limiter = SlidingWindow::new(1, 1_000);
        limiter.allow("old", Timestamp(0));
        limiter.allow("new", Timestamp(5_000));
        assert_eq!(limiter.hits.lock().unwrap().len(), 1);
    }
}
