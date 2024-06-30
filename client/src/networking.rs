use bevy::prelude::*;
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode,
        connection::{
            ClientEndpointConfiguration, ConnectionEvent, ConnectionFailedEvent,
            ConnectionLostEvent,
        },
        QuinnetClient,
    },
    shared::channels::ChannelsConfiguration,
};
use common::protocol::ServerMsgRoot;

use crate::{
    states::{ClientState, GameState, MenuState},
    ui::modes::{combine, draw, prompt, vote},
    ConnectionData,
};

pub fn handle_server_messages(
    mut commands: Commands,
    mut client: ResMut<QuinnetClient>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(connection) = client.get_connection_mut() else {
        return;
    };
    while let Some((_, message)) = connection.try_receive_message::<ServerMsgRoot>() {
        match message {
            ServerMsgRoot::Draw { duration } => {
                next.set(GameState::Draw);
                commands.insert_resource(draw::Data { duration });
            }
            ServerMsgRoot::Prompt { duration } => {
                next.set(GameState::Prompt);
                commands.insert_resource(prompt::Data { duration });
            }
            ServerMsgRoot::Combine {
                duration,
                drawings,
                prompts,
            } => {
                next.set(GameState::Combine);
                commands.insert_resource(combine::Data {
                    duration,
                    drawings,
                    prompts,
                });
            }
            ServerMsgRoot::Vote {
                duration,
                combination1,
                combination2,
            } => {
                next.set(GameState::Vote);
                commands.insert_resource(vote::Data {
                    duration,
                    combination1,
                    combination2,
                });
            }
            ServerMsgRoot::Wait => next.set(GameState::Wait),
        }
    }
}

pub fn start_connection(mut client: ResMut<QuinnetClient>, data: Res<ConnectionData>) {
    client
        .open_connection(
            ClientEndpointConfiguration::from_strings(&data.address, "0.0.0.0:0").unwrap(),
            CertificateVerificationMode::SkipVerification,
            ChannelsConfiguration::default(),
        )
        .unwrap();
    info!("Connecting");
}

pub fn handle_client_connecting_events(
    mut connection: EventReader<ConnectionEvent>,
    mut connection_failed: EventReader<ConnectionFailedEvent>,
    mut client_next: ResMut<NextState<ClientState>>,
    mut menu_next: ResMut<NextState<MenuState>>,
    mut client: ResMut<QuinnetClient>,
) {
    if !connection.is_empty() {
        connection.clear();
        client_next.set(ClientState::Game);
        info!("Connected");
    }
    if !connection_failed.is_empty() {
        connection_failed.clear();
        menu_next.set(MenuState::Configuring);
        handle_disconnect(&mut client, "Connection failed");
    }
}

pub fn handle_client_disconnected_events(
    mut connection_lost: EventReader<ConnectionLostEvent>,
    mut client_next: ResMut<NextState<ClientState>>,
    mut client: ResMut<QuinnetClient>,
) {
    if !connection_lost.is_empty() {
        connection_lost.clear();
        client_next.set(ClientState::Menu);
        handle_disconnect(&mut client, "Connection lost");
    }
}

pub fn handle_disconnect(client: &mut QuinnetClient, cause: &'static str) {
    info!(cause, "Disconnected");
    client.close_all_connections().ok();
}
