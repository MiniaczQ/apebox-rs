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
    ui::game::{combine::CombineData, draw::DrawData, prompt::PromptData, vote::VoteData},
    ConnectionData,
};

pub fn handle_server_messages(
    mut client: ResMut<QuinnetClient>,
    mut next: ResMut<NextState<GameState>>,
    mut draw_data: EventWriter<DrawData>,
    mut prompt_data: EventWriter<PromptData>,
    mut combine_data: EventWriter<CombineData>,
    mut vote_data: EventWriter<VoteData>,
) {
    let Some(connection) = client.get_connection_mut() else {
        return;
    };
    while let Some((_, message)) = connection.try_receive_message::<ServerMsgRoot>() {
        match message {
            ServerMsgRoot::Draw { duration } => {
                next.set(GameState::Draw);
                draw_data.send(DrawData { duration });
            }
            ServerMsgRoot::Prompt { duration } => {
                next.set(GameState::Prompt);
                prompt_data.send(PromptData { duration });
            }
            ServerMsgRoot::Combine {
                duration,
                drawings,
                prompts,
            } => {
                next.set(GameState::Combine);
                combine_data.send(CombineData {
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
                vote_data.send(VoteData {
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
