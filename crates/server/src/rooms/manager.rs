//! All open rooms: creating them within limits, finding them, and closing
//! rooms that have stayed empty.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use domain::{Board, Robots, Timestamp};

use super::actor::{DeckShuffler, RoomHandle};
use crate::clock::Clock;
use crate::identity::random_token;

/// An empty room is closed after this long.
pub const EMPTY_CLOSE_MS: u64 = 10 * 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomLimits {
    /// Open rooms in total.
    pub global: usize,
    /// Open rooms per owner (a session or client address).
    pub per_owner: usize,
    /// Members per room.
    pub capacity: usize,
}

impl Default for RoomLimits {
    fn default() -> Self {
        Self {
            global: 200,
            per_owner: 2,
            capacity: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateError {
    TooManyRooms,
    OwnerLimit,
}

/// Lobby view of a room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomSummary {
    pub id: String,
    pub members: usize,
}

struct Entry {
    handle: RoomHandle,
    owner: String,
    empty_since: Option<Timestamp>,
}

/// Creates rooms on a fixed board setup; the actor does the rest.
pub struct RoomManager {
    limits: RoomLimits,
    setup: Box<dyn Fn() -> (Board, Robots) + Send + Sync>,
    clock: Arc<dyn Clock>,
    shuffler: Arc<dyn DeckShuffler>,
    rooms: Mutex<HashMap<String, Entry>>,
}

impl std::fmt::Debug for RoomManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomManager")
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl RoomManager {
    /// `setup` gives each new room its board and starting robots.
    pub fn new(
        limits: RoomLimits,
        setup: impl Fn() -> (Board, Robots) + Send + Sync + 'static,
        clock: Arc<dyn Clock>,
        shuffler: Arc<dyn DeckShuffler>,
    ) -> Self {
        Self {
            limits,
            setup: Box::new(setup),
            clock,
            shuffler,
            rooms: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        // Entries are only inserted or removed whole, so a panic elsewhere
        // can't leave the map half-updated.
        self.rooms
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Opens a room for `owner`; must run inside a Tokio runtime.
    ///
    /// # Errors
    /// `TooManyRooms` at the global cap, `OwnerLimit` at the owner's cap.
    pub fn create(&self, owner: &str) -> Result<(String, RoomHandle), CreateError> {
        let mut rooms = self.lock();
        if rooms.len() >= self.limits.global {
            return Err(CreateError::TooManyRooms);
        }
        if rooms.values().filter(|e| e.owner == owner).count() >= self.limits.per_owner {
            return Err(CreateError::OwnerLimit);
        }
        let (board, robots) = (self.setup)();
        let handle = RoomHandle::spawn(
            board,
            robots,
            self.limits.capacity,
            self.clock.clone(),
            self.shuffler.clone(),
        );
        let id = random_token();
        rooms.insert(
            id.clone(),
            Entry {
                handle: handle.clone(),
                owner: owner.to_owned(),
                empty_since: None,
            },
        );
        Ok((id, handle))
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<RoomHandle> {
        self.lock().get(id).map(|e| e.handle.clone())
    }

    /// Open rooms with their member counts, for the lobby.
    pub async fn list(&self) -> Vec<RoomSummary> {
        let handles: Vec<(String, RoomHandle)> = self
            .lock()
            .iter()
            .map(|(id, e)| (id.clone(), e.handle.clone()))
            .collect();
        let mut summaries = Vec::with_capacity(handles.len());
        for (id, handle) in handles {
            if let Ok(members) = handle.members().await {
                summaries.push(RoomSummary {
                    id,
                    members: members.len(),
                });
            }
        }
        summaries.sort_by(|a, b| a.id.cmp(&b.id));
        summaries
    }

    /// Closes rooms that have been empty for [`EMPTY_CLOSE_MS`]; call it
    /// periodically. Returns how many were closed.
    pub async fn close_idle(&self) -> usize {
        let now = self.clock.now();
        let handles: Vec<(String, RoomHandle)> = self
            .lock()
            .iter()
            .map(|(id, e)| (id.clone(), e.handle.clone()))
            .collect();
        let mut closed = 0;
        for (id, handle) in handles {
            let empty = handle.members().await.map_or(true, |m| m.is_empty());
            let mut rooms = self.lock();
            let Some(entry) = rooms.get_mut(&id) else {
                continue;
            };
            if !empty {
                entry.empty_since = None;
                continue;
            }
            let since = *entry.empty_since.get_or_insert(now);
            if now.0.saturating_sub(since.0) >= EMPTY_CLOSE_MS {
                rooms.remove(&id);
                closed += 1;
            }
        }
        closed
    }
}
