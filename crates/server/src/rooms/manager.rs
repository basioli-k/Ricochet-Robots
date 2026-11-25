//! All open rooms: creating them within limits, finding them, and closing
//! rooms that have stayed empty.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use domain::{Board, Robots, Timestamp};

use super::actor::{DeckShuffler, RoomHandle};
use crate::clock::Clock;
use crate::identity::random_token;
use crate::rate_limit::RateLimiter;

/// An empty room is closed after this long.
pub const EMPTY_CLOSE_MS: u64 = 10 * 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomLimits {
    /// Open rooms in total.
    pub global: usize,
    /// Open rooms per creating player, and separately per client address.
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
    RateLimited,
}

/// Who opens a room: the player, and their client's rate-limit key (see
/// [`crate::rate_limit::client_key`]). The room counts against both.
#[derive(Debug, Clone, Copy)]
pub struct Creator<'a> {
    pub player: &'a str,
    pub client: &'a str,
}

/// Lobby view of a room.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RoomSummary {
    pub id: String,
    pub members: usize,
}

struct Entry {
    handle: RoomHandle,
    owners: Vec<String>,
    empty_since: Option<Timestamp>,
}

/// Creates rooms on a fixed board setup; the actor does the rest.
pub struct RoomManager {
    limits: RoomLimits,
    setup: Box<dyn Fn() -> (Board, Robots) + Send + Sync>,
    clock: Arc<dyn Clock>,
    shuffler: Arc<dyn DeckShuffler>,
    /// Room creations per client over time, on top of the open-room caps.
    create_limiter: Arc<dyn RateLimiter>,
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
        create_limiter: Arc<dyn RateLimiter>,
    ) -> Self {
        Self {
            limits,
            setup: Box::new(setup),
            clock,
            shuffler,
            create_limiter,
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

    /// Opens a room for `by`; must run inside a Tokio runtime. Only a
    /// creation that passes the caps counts against the rate limit.
    ///
    /// # Errors
    /// `TooManyRooms` at the global cap, `OwnerLimit` if the player or the
    /// client is at their cap, `RateLimited` if the client creates too often.
    pub fn create(&self, by: Creator<'_>) -> Result<(String, RoomHandle), CreateError> {
        // Separate namespaces, so a player ID can't collide with a client key.
        let owners = [
            format!("player:{}", by.player),
            format!("client:{}", by.client),
        ];
        let mut rooms = self.lock();
        if rooms.len() >= self.limits.global {
            return Err(CreateError::TooManyRooms);
        }
        for owner in &owners {
            let open = rooms.values().filter(|e| e.owners.contains(owner)).count();
            if open >= self.limits.per_owner {
                return Err(CreateError::OwnerLimit);
            }
        }
        if !self.create_limiter.allow(by.client, self.clock.now()) {
            return Err(CreateError::RateLimited);
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
                owners: owners.into(),
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

    /// Closes rooms that have been empty for [`EMPTY_CLOSE_MS`]: their
    /// tasks stop, so handles still held elsewhere get `Closed`, and they no
    /// longer count against the caps. Call it periodically. Returns how
    /// many were closed.
    pub async fn close_idle(&self) -> usize {
        let now = self.clock.now();
        let handles: Vec<(String, RoomHandle)> = self
            .lock()
            .iter()
            .map(|(id, e)| (id.clone(), e.handle.clone()))
            .collect();
        let mut closed = 0;
        for (id, handle) in handles {
            // A room whose task already stopped counts as empty.
            let empty = handle.members().await.map_or(true, |m| m.is_empty());
            let due = {
                let mut rooms = self.lock();
                let Some(entry) = rooms.get_mut(&id) else {
                    continue;
                };
                if !empty {
                    entry.empty_since = None;
                    continue;
                }
                let since = *entry.empty_since.get_or_insert(now);
                now.0.saturating_sub(since.0) >= EMPTY_CLOSE_MS
            };
            if !due {
                continue;
            }
            // The task decides, so a member who joined since the check
            // above keeps the room open.
            let stopped = handle.close_if_empty().await.unwrap_or(true);
            let mut rooms = self.lock();
            if stopped {
                closed += usize::from(rooms.remove(&id).is_some());
            } else if let Some(entry) = rooms.get_mut(&id) {
                entry.empty_since = None;
            }
        }
        closed
    }
}
