mod game;
mod networking;
mod state;

use std::time::Duration;

use bevy::{
    dev_tools::states::log_transitions, ecs::prelude::Resource, log::LogPlugin,
    state::app::StatesPlugin,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_quinnet::{
    server::{QuinnetServer, QuinnetServerPlugin},
    shared::ClientId,
};
use common::protocol::ServerMessage;
use game::{GameData, GameStage};
use state::{GameState, ServerState};

fn main() {
    let mut app = App::new();
    app.init_state::<ServerState>();
    app.add_sub_state::<GameState>();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin::default(),
        StatesPlugin,
        QuinnetServerPlugin::default(),
    ));
    app.init_resource::<Users>();

    // Debug
    app.add_systems(
        Update,
        (log_transitions::<ServerState>, log_transitions::<GameState>).chain(),
    );

    // ServerState::Offline
    app.add_systems(OnEnter(ServerState::Offline), state::setup_server_offline);

    // ServerState::Running
    app.add_systems(OnEnter(ServerState::Running), state::setup_server_online);
    app.add_systems(OnExit(ServerState::Running), state::teardown_server_online);
    app.add_systems(
        PreUpdate,
        (
            networking::handle_server_events,
            networking::handle_client_messages,
        )
            .chain()
            .run_if(in_state(ServerState::Running)),
    );

    // GameState::Lobby
    app.add_systems(Update, start_lobby.run_if(in_state(GameState::Lobby)));

    // GameState::Playing
    app.add_systems(OnEnter(GameState::Playing), state::setup_game_playing);
    app.add_systems(OnExit(GameState::Playing), state::teardown_game_playing);
    app.add_systems(
        Update,
        (progress_game, everyone_left).run_if(in_state(GameState::Playing)),
    );

    app.run();
}

/// Users and information about them.
#[derive(Resource, Debug, Clone, Default)]
struct Users {
    /// Clients that connected but didn't register a username.
    pending: HashMap<ClientId, Duration>,
    /// Clients that registered a username.
    registered: HashMap<ClientId, UserData>,
}

impl Users {
    /// Register a new pending user.
    fn add_pending(&mut self, id: ClientId, now: Duration) {
        self.pending.insert(id, now);
    }

    /// Turn a pending user into an active user by giving them a name.
    fn register(&mut self, id: ClientId, name: String) {
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
    fn set_playing(&mut self) {
        for user in self.registered.values_mut() {
            user.playing = true;
        }
    }

    /// Remove all trace of a user.
    fn remove(&mut self, id: &ClientId) -> Option<UserData> {
        self.pending.remove(id);
        self.registered.remove(id)
    }

    /// Take all pending users who didn't register a name for too long.
    fn drain_pending_too_long(&mut self, max_pending: Duration, now: Duration) -> Vec<ClientId> {
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
struct UserData {
    /// Username.
    name: String,
    /// Whether the user is waiting for a new game or already playing.
    playing: bool,
}

fn progress_game(
    users: ResMut<Users>,
    mut gamedata: ResMut<GameData>,
    time: Res<Time>,
    mut server: ResMut<QuinnetServer>,
    mut next: ResMut<NextState<GameState>>,
) {
    let now = time.elapsed();
    if !gamedata.try_progress(now) {
        return;
    }

    if gamedata.has_ended() {
        next.set(GameState::Lobby);
        return;
    }

    let stage = gamedata.stage.as_ref().unwrap();

    let message = match stage {
        GameStage::Draw { duration } => ServerMessage::Draw {
            duration: *duration,
        },
        GameStage::Prompt { duration, .. } => ServerMessage::Prompt {
            duration: *duration,
        },
        GameStage::Combine { duration } => ServerMessage::Prompt {
            duration: *duration,
        },
        GameStage::Vote { duration } => ServerMessage::Prompt {
            duration: *duration,
        },
    };
    let endpoint = server.endpoint_mut();
    for (id, _) in users.registered.iter().filter(|(_, u)| u.playing) {
        endpoint.send_message(*id, &message).unwrap();
    }
}

fn start_lobby(users: Res<Users>, mut next: ResMut<NextState<GameState>>) {
    // TODO: Wait for host start instead
    if users.registered.len() >= 1 {
        next.set(GameState::Playing);
    }
}

fn everyone_left(users: Res<Users>, mut next: ResMut<NextState<GameState>>) {
    if users.registered.len() < 1 {
        next.set(GameState::Lobby);
    }
}
