//! Caps on open game sockets per player and per client, so one session or
//! address can't tie up thousands of tasks.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Connections {
    per_player: usize,
    per_client: usize,
    open: Mutex<HashMap<String, usize>>,
}

/// One open socket; dropping it frees its place.
#[derive(Debug)]
pub struct Ticket {
    owner: Arc<Connections>,
    keys: [String; 2],
}

impl Connections {
    #[must_use]
    pub fn new(per_player: usize, per_client: usize) -> Self {
        Self {
            per_player,
            per_client,
            open: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, usize>> {
        self.open
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// A ticket for one more socket, unless the player or the client
    /// (see [`crate::rate_limit::client_key`]) already has its share.
    #[must_use]
    pub fn open(self: &Arc<Self>, player: &str, client: &str) -> Option<Ticket> {
        let keys = [format!("player:{player}"), format!("client:{client}")];
        let mut open = self.lock();
        let count = |key: &String| open.get(key).copied().unwrap_or(0);
        if count(&keys[0]) >= self.per_player || count(&keys[1]) >= self.per_client {
            return None;
        }
        for key in &keys {
            *open.entry(key.clone()).or_default() += 1;
        }
        drop(open);
        Some(Ticket {
            owner: self.clone(),
            keys,
        })
    }

    #[cfg(test)]
    fn count(&self, key: &str) -> usize {
        self.lock().get(key).copied().unwrap_or(0)
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let mut open = self.owner.lock();
        for key in &self.keys {
            if let Some(n) = open.get_mut(key) {
                *n -= 1;
                if *n == 0 {
                    open.remove(key);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn players_and_clients_have_separate_caps() {
        let caps = Arc::new(Connections::new(2, 3));
        let a = caps.open("p1", "ip");
        let b = caps.open("p1", "ip");
        assert!(a.is_some() && b.is_some());
        assert!(caps.open("p1", "other").is_none(), "player at its cap");
        let c = caps.open("p2", "ip");
        assert!(c.is_some());
        assert!(caps.open("p3", "ip").is_none(), "client at its cap");
        assert!(caps.open("p3", "other").is_some());
    }

    #[test]
    fn closing_frees_a_place_and_forgets_idle_keys() {
        let caps = Arc::new(Connections::new(1, 1));
        let ticket = caps.open("p1", "ip").unwrap();
        assert!(caps.open("p1", "ip").is_none());
        drop(ticket);
        assert_eq!(caps.count("player:p1"), 0);
        assert!(caps.lock().is_empty());
        assert!(caps.open("p1", "ip").is_some());
    }
}
