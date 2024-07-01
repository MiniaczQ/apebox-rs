use bevy::prelude::*;
use bevy_quinnet::server::{ConnectionEvent, ConnectionLostEvent, QuinnetServer};
use common::game::Indexer;

use crate::{game::GameConfig, networking, Users};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ServerState {
    #[default]
    Offline,
    Running,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(ServerState = ServerState::Running)]
pub enum RoomState {
    #[default]
    Waiting,
    Running,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(RoomState = RoomState::Running)]
pub enum GameState {
    #[default]
    Init,
    Draw,
    Prompt,
    Combine,
    Vote,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(GameState = GameState::Vote)]
pub enum VoteState {
    #[default]
    Voting,
    Winner,
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

pub fn setup_room_running(mut commands: Commands, mut users: ResMut<Users>) {
    users.set_playing();
    commands.insert_resource(GameConfig::short());
    commands.init_resource::<Indexer>();
}

pub fn teardown_room_running(mut commands: Commands) {
    commands.remove_resource::<GameConfig>();
    commands.remove_resource::<Indexer>();
}
