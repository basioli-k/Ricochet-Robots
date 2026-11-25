//! Sliding-window rate limits, keyed by e.g. client IP.

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::Mutex;

use domain::Timestamp;

/// Decides whether an action is allowed now; records it if so.
pub trait RateLimiter: Send + Sync {
    fn allow(&self, key: &str, now: Timestamp) -> bool;
}

/// The rate-limit key for a client address. An IPv6 client usually holds
/// a whole /64, so it is keyed by that prefix; otherwise it could use a
/// fresh address per request. IPv4-mapped IPv6 addresses count as IPv4.
#[must_use]
pub fn client_key(ip: IpAddr) -> String {
    match ip.to_canonical() {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => {
            let s = v6.segments();
            format!("{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3])
        }
    }
}

/// At most `limit` actions per key within any `window_ms` milliseconds.
#[derive(Debug)]
pub struct SlidingWindow {
    limit: usize,
    window_ms: u64,
    state: Mutex<Windows>,
}

#[derive(Debug, Default)]
struct Windows {
    hits: HashMap<String, VecDeque<Timestamp>>,
    /// When idle keys were last swept; sweeping at most once per window
    /// keeps each call O(1) amortized however many keys there are.
    swept_at: u64,
}

impl SlidingWindow {
    #[must_use]
    pub fn new(limit: usize, window_ms: u64) -> Self {
        Self {
            limit,
            window_ms,
            state: Mutex::new(Windows::default()),
        }
    }

    /// Drops hits that have left the window: a hit leaves once it is
    /// `window_ms` old (no subtraction, which would saturate near time 0).
    fn expire(&self, times: &mut VecDeque<Timestamp>, now: Timestamp) {
        while times
            .front()
            .is_some_and(|t| t.0.saturating_add(self.window_ms) <= now.0)
        {
            times.pop_front();
        }
    }
}

impl RateLimiter for SlidingWindow {
    fn allow(&self, key: &str, now: Timestamp) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Forget keys with nothing recent, so the map holds at most the
        // keys seen in about two windows.
        if state.swept_at.saturating_add(self.window_ms) <= now.0 {
            state.hits.retain(|_, times| {
                self.expire(times, now);
                !times.is_empty()
            });
            state.swept_at = now.0;
        }
        let times = state.hits.entry(key.to_owned()).or_default();
        self.expire(times, now);
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

    fn keys(limiter: &SlidingWindow) -> usize {
        limiter.state.lock().unwrap().hits.len()
    }

    #[test]
    fn idle_keys_are_swept_once_per_window() {
        let limiter = SlidingWindow::new(1, 1_000);
        limiter.allow("a", Timestamp(1_000)); // sweeps
        limiter.allow("b", Timestamp(1_900));
        limiter.allow("c", Timestamp(2_500)); // sweeps: "a" is idle
        assert_eq!(keys(&limiter), 2);
        limiter.allow("d", Timestamp(3_000));
        assert_eq!(keys(&limiter), 3, "idle \"b\" waits for the next sweep");
        limiter.allow("e", Timestamp(3_500)); // sweeps: "b" and "c" are idle
        assert_eq!(keys(&limiter), 2);
    }

    #[test]
    fn ipv4_is_keyed_by_address_and_ipv6_by_its_64_prefix() {
        let key = |s: &str| client_key(s.parse().unwrap());
        assert_eq!(key("203.0.113.7"), "203.0.113.7");
        assert_eq!(key("::ffff:203.0.113.7"), "203.0.113.7");
        assert_eq!(key("2001:db8:1:2:aaaa::1"), "2001:db8:1:2::/64");
        assert_eq!(key("2001:db8:1:2:bbbb::9"), key("2001:db8:1:2::5"));
        assert_ne!(key("2001:db8:1:2::1"), key("2001:db8:1:3::1"));
    }
}
