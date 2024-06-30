mod game;
mod modes;
mod networking;
mod states;
mod users;

use bevy::prelude::*;
use bevy::{dev_tools::states::log_transitions, log::LogPlugin, state::app::StatesPlugin};
use bevy_quinnet::server::QuinnetServerPlugin;
use common::{
    app::AppExt,
    protocol::{ClientMsgComm, ClientMsgRoot, NetMsg},
    transitions::IdentityTransitionsPlugin,
};
use game::{GameConfig, StateData};
use modes::ModesPlugin;
use states::{GameState, RoomState, ServerState, VoteState};
use users::Users;

fn main() {
    let mut app = App::new();
    app.init_state::<ServerState>();
    app.add_sub_state::<RoomState>();
    app.add_sub_state::<GameState>();
    app.add_sub_state::<VoteState>();
    app.add_plugins((
        MinimalPlugins,
        LogPlugin::default(),
        StatesPlugin,
        QuinnetServerPlugin::default(),
        IdentityTransitionsPlugin::<GameState>::default(),
        IdentityTransitionsPlugin::<VoteState>::default(),
    ));
    app.init_resource::<Users>();
    app.configure_sets(
        Update,
        (
            GameSystemOdering::Networking,
            GameSystemOdering::StateLogic,
            GameSystemOdering::ChangeState,
        )
            .chain(),
    );

    // Debug
    app.add_systems(
        Update,
        (
            log_transitions::<ServerState>,
            log_transitions::<RoomState>,
            log_transitions::<GameState>,
        )
            .chain(),
    );

    // ServerState::Offline
    app.add_systems(OnEnter(ServerState::Offline), states::setup_server_offline);

    // ServerState::Running
    app.add_systems(OnEnter(ServerState::Running), states::setup_server_online);
    app.add_systems(OnExit(ServerState::Running), states::teardown_server_online);
    app.add_event::<NetMsg<ClientMsgRoot>>();
    app.add_event::<NetMsg<ClientMsgComm>>();
    app.add_event::<ProgressGame>();
    app.add_systems(
        PreUpdate,
        (
            networking::handle_server_events,
            networking::receive_messages,
            networking::handle_root,
            networking::handle_comm,
        )
            .chain()
            .in_set(GameSystemOdering::Networking)
            .run_if(in_state(ServerState::Running)),
    );
    app.enable_state_scoped_entities::<RoomState>();

    // RoomState::Waiting
    app.add_systems(
        Update,
        start_lobby
            .in_set(GameSystemOdering::ChangeState)
            .run_if(in_state(RoomState::Waiting)),
    );

    // RoomState::Running
    app.add_event::<ProgressGame>();
    app.add_statebound(
        RoomState::Running,
        states::setup_room_running,
        states::teardown_room_running,
        (progress_game, stop_lobby).in_set(GameSystemOdering::ChangeState),
    );

    app.add_plugins(ModesPlugin);

    app.run();
}

#[derive(SystemSet, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum GameSystemOdering {
    Networking,
    StateLogic,
    ChangeState,
}

#[derive(Event)]
pub struct ProgressGame;

fn progress_game(
    mut progress: ResMut<Events<ProgressGame>>,
    mut commands: Commands,
    mut room_next: ResMut<NextState<RoomState>>,
    mut game_next: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameConfig>,
) {
    if progress.drain().last().is_none() {
        return;
    }

    let Some(next_state) = game_data.next_state() else {
        room_next.set(RoomState::Waiting);
        return;
    };

    match next_state {
        StateData::Draw(config) => {
            game_next.set(GameState::Draw);
            commands.insert_resource(config);
        }
        StateData::Prompt(config) => {
            game_next.set(GameState::Prompt);
            commands.insert_resource(config);
        }
        StateData::Combine(config) => {
            game_next.set(GameState::Combine);
            commands.insert_resource(config);
        }
        StateData::Vote(config) => {
            game_next.set(GameState::Vote);
            commands.insert_resource(config);
        }
    };
}

fn start_lobby(
    users: Res<Users>,
    mut room_next: ResMut<NextState<RoomState>>,
    mut progress: EventWriter<ProgressGame>,
) {
    // TODO: Wait for host start instead
    if users.registered.len() >= 2 {
        room_next.set(RoomState::Running);
        progress.send(ProgressGame);
    }
}

fn stop_lobby(users: Res<Users>, mut room_next: ResMut<NextState<RoomState>>) {
    if users.registered.len() < 1 {
        room_next.set(RoomState::Waiting);
    }
}
