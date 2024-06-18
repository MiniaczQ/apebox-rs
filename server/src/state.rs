use bevy::prelude::*;
use bevy_quinnet::server::{ConnectionEvent, ConnectionLostEvent, QuinnetServer};

use crate::{
    game::{stages_short, GameData},
    networking, Users,
};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ServerState {
    /// No listener and active connections.
    #[default]
    Offline,
    /// Listening for connections.
    Running,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(ServerState = ServerState::Running)]
pub enum GameState {
    /// Waiting for users.
    #[default]
    Lobby,
    /// Playing a single round.
    Playing,
}

pub fn setup_server_offline(mut next: ResMut<NextState<ServerState>>) {
    next.set(ServerState::Running);
}

pub fn setup_server_online(
    mut commands: Commands,
    mut connection: ResMut<Events<ConnectionEvent>>,
    mut connection_lost: ResMut<Events<ConnectionLostEvent>>,
    mut server: ResMut<QuinnetServer>,
) {
    commands.init_resource::<Users>();
    connection.clear();
    connection_lost.clear();
    networking::start_server(&mut server);
    info!("Server online!");
}

pub fn teardown_server_online(mut commands: Commands, mut server: ResMut<QuinnetServer>) {
    networking::stop_server(&mut server);
    commands.remove_resource::<Users>();
    info!("Server offline!");
}

pub fn setup_game_playing(mut commands: Commands, mut users: ResMut<Users>) {
    users.set_playing();
    commands.insert_resource(GameData::from_stages(stages_short()));
}

pub fn teardown_game_playing(mut commands: Commands) {
    commands.remove_resource::<GameData>();
}
