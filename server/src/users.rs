use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_quinnet::shared::ClientId;

/// Users and information about them.
#[derive(Resource, Debug, Clone, Default)]
pub struct Users {
    /// Clients that connected but didn't register a username.
    pub pending: HashMap<ClientId, Duration>,
    /// Clients that registered a username.
    pub registered: HashMap<ClientId, UserData>,
}

impl Users {
    /// Register a new pending user.
    pub fn add_pending(&mut self, id: ClientId, now: Duration) {
        self.pending.insert(id, now);
    }

    /// Turn a pending user into an active user by giving them a name.
    pub fn register(&mut self, id: ClientId, name: String) {
        self.pending.remove(&id).unwrap();
        self.registered.insert(
            id,
            UserData {
                name,
                playing: false,
            },
        );
    }

    /// Set all registered players to playing.
    pub fn set_playing(&mut self) {
        for user in self.registered.values_mut() {
            user.playing = true;
        }
    }

    /// Remove all trace of a user.
    pub fn remove(&mut self, id: &ClientId) -> Option<UserData> {
        self.pending.remove(id);
        self.registered.remove(id)
    }

    pub fn iter_active(&self) -> impl Iterator<Item = (&u64, &UserData)> {
        self.registered.iter().filter(|(_, u)| u.playing)
    }

    /// Take all pending users who didn't register a name for too long.
    pub fn drain_pending_too_long(
        &mut self,
        max_pending: Duration,
        now: Duration,
    ) -> Vec<ClientId> {
        let mut pending_too_long = vec![];
        for (id, joined) in self.pending.iter() {
            if *joined + max_pending < now {
                pending_too_long.push(*id);
            }
        }
        for id in pending_too_long.iter() {
            self.pending.remove(id);
        }
        pending_too_long
    }
}

/// Data of a single registered user.
#[derive(Resource, Debug, Clone)]
pub struct UserData {
    /// Username.
    pub name: String,
    /// Whether the user is waiting for a new game or already playing.
    pub playing: bool,
}
