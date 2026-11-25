//! The current time, injected so tests control it.

use std::sync::atomic::{AtomicU64, Ordering};

use domain::Timestamp;

pub trait Clock: Send + Sync {
    /// Milliseconds since the Unix epoch, never going backwards.
    fn now(&self) -> Timestamp;
}

/// The system clock, made monotonic: if the system time is set back, it
/// keeps returning the last value until real time catches up, so bids can
/// never be ordered against the time they were placed.
#[derive(Debug, Default)]
pub struct SystemClock {
    last: AtomicU64,
}

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let wall = u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX);
        let previous = self.last.fetch_max(wall, Ordering::SeqCst);
        Timestamp(previous.max(wall))
    }
}

/// A clock that only moves when told to, for tests.
#[derive(Debug, Default)]
pub struct FakeClock(AtomicU64);

impl FakeClock {
    #[must_use]
    pub fn at(millis: u64) -> Self {
        Self(AtomicU64::new(millis))
    }

    pub fn advance(&self, millis: u64) {
        self.0.fetch_add(millis, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        Timestamp(self.0.load(Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_clock_never_goes_back() {
        let clock = SystemClock::default();
        // Simulate an earlier reading far in the future, as after the
        // system time was set back.
        clock.last.store(u64::MAX - 1, Ordering::SeqCst);
        assert_eq!(clock.now(), Timestamp(u64::MAX - 1));
    }

    #[test]
    fn the_system_clock_tracks_real_time() {
        let clock = SystemClock::default();
        let first = clock.now();
        assert!(first.0 > 1_700_000_000_000, "milliseconds since the epoch");
        assert!(clock.now() >= first);
    }
}
