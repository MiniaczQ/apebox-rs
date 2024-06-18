mod networking;
mod states;
mod ui;

use std::{thread::sleep, time::Duration};

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_egui::EguiPlugin;
use bevy_quinnet::client::{QuinnetClient, QuinnetClientPlugin};
use common::protocol::ClientMessage;
use states::{ClientState, GameState, MenuState};
use ui::ClientUiPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pee K.O.".into(),
                ..default()
            }),
            ..default()
        }),
        QuinnetClientPlugin::default(),
        EguiPlugin,
        ClientUiPlugin,
    ));
    app.init_state::<ClientState>();
    app.add_sub_state::<MenuState>();
    app.add_sub_state::<GameState>();
    app.init_resource::<ConnectionData>();

    // Debug
    app.add_systems(
        Update,
        (
            log_transitions::<ClientState>,
            log_transitions::<MenuState>,
            log_transitions::<GameState>,
        )
            .chain(),
    );

    // ClientState::Lobby
    app.add_systems(OnEnter(ClientState::Game), setup_client_lobby);
    app.add_systems(
        Update,
        (
            networking::handle_client_disconnected_events,
            networking::handle_server_messages,
        )
            .chain()
            .run_if(in_state(ClientState::Game)),
    );

    // MenuState::Connecting
    app.add_systems(OnEnter(MenuState::Connecting), networking::start_connection);
    app.add_systems(
        Update,
        networking::handle_client_connecting_events.run_if(in_state(MenuState::Connecting)),
    );

    app.add_systems(PostUpdate, on_app_exit);
    app.run();
}

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Option<Res<QuinnetClient>>) {
    if let Some(client) = client {
        if !app_exit_events.is_empty() {
            client
                .connection()
                .send_message(ClientMessage::Disconnect)
                .unwrap();
            sleep(Duration::from_secs_f32(0.1));
        }
    }
}

#[derive(Resource)]
pub struct ConnectionData {
    address: String,
    name: String,
}

impl Default for ConnectionData {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:6000".to_owned(),
            name: "test-user".to_owned(),
        }
    }
}

pub fn setup_client_lobby(mut client: ResMut<QuinnetClient>, data: Res<ConnectionData>) {
    client
        .connection_mut()
        .send_message(ClientMessage::Join {
            name: data.name.clone(),
        })
        .ok();
}
