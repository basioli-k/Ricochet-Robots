//! The current time, injected so tests control it.

use std::sync::atomic::{AtomicU64, Ordering};

use domain::Timestamp;

pub trait Clock: Send + Sync {
    /// Milliseconds since the Unix epoch, never going backwards.
    fn now(&self) -> Timestamp;
}

/// The system clock.
#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Timestamp(u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
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
