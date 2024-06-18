use crate::game::{GameData, GameStage};

use super::Users;
use bevy::prelude::*;
use bevy_quinnet::shared::ClientId;
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionEvent, ConnectionLostEvent, Endpoint,
        QuinnetServer, ServerEndpointConfiguration,
    },
    shared::channels::ChannelsConfiguration,
};
use common::game::Signed;
use common::protocol::{ClientMessage, ServerMessage};
use std::time::Duration;

pub fn handle_client_messages(
    mut server: ResMut<QuinnetServer>,
    mut users: ResMut<Users>,
    mut gamestate: Option<ResMut<GameData>>,
) {
    let endpoint = server.endpoint_mut();
    for id in endpoint.clients() {
        while let Some((_, message)) = endpoint.try_receive_message_from::<ClientMessage>(id) {
            handle_client_message(message, &mut users, gamestate.as_deref_mut(), id, endpoint);
        }
    }
}

pub fn handle_client_message(
    message: ClientMessage,
    users: &mut Users,
    gamestate: Option<&mut GameData>,
    id: u64,
    endpoint: &mut Endpoint,
) {
    let user_exists = users.registered.get(&id);
    match message {
        ClientMessage::Join { name } => {
            let name_exists = users.registered.values().any(|u| u.name == name);
            if user_exists.is_some() || name_exists {
                handle_disconnect(users, id, Some(endpoint), "Duplicate client id");
            } else {
                info!(id, name, "Client active.");
                users.register(id, name);
                endpoint
                    .send_message(
                        id,
                        ServerMessage::Wait {
                            message: "Waiting for next game.".to_owned(),
                        },
                    )
                    .unwrap();
            }
        }
        message => {
            let Some(gamestate) = gamestate else {
                handle_disconnect(users, id, Some(endpoint), "Protocol error");
                return;
            };
            let Some(user) = user_exists else {
                handle_disconnect(users, id, Some(endpoint), "Duplicate username");
                return;
            };
            match message {
                ClientMessage::Disconnect {} => {
                    info!(id, "Client disconnected.");
                    handle_disconnect(users, id, Some(endpoint), "Disconnected");
                }
                ClientMessage::SubmitDrawing(drawing) => {
                    let Some(GameStage::Draw { .. }) = gamestate.stage else {
                        return;
                    };
                    // TODO: ensure one drawing per user per stage
                    gamestate.drawings.push(Signed {
                        data: drawing,
                        author: user.name.clone(),
                    })
                }
                ClientMessage::SubmitPrompt(prompt) => {
                    let Some(GameStage::Prompt { .. }) = gamestate.stage else {
                        return;
                    };
                    // TODO: ensure max N prompts from all users per stagte
                    gamestate.prompts.push(Signed {
                        data: prompt,
                        author: user.name.clone(),
                    })
                }
                ClientMessage::SubmitCombination(combination) => {
                    let Some(GameStage::Combine { .. }) = gamestate.stage else {
                        return;
                    };
                    // TODO: ensure one combination per user per stage
                    gamestate.combinations.push(Signed {
                        data: combination,
                        author: user.name.clone(),
                    })
                }
                ClientMessage::SubmitVote(vote) => {
                    let Some(GameStage::Vote { .. }) = gamestate.stage else {
                        return;
                    };
                    // TODO: ensure one vote per user per voting
                    info!("Vote on {}", vote);
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn handle_server_events(
    mut connection: EventReader<ConnectionEvent>,
    mut connection_lost: EventReader<ConnectionLostEvent>,
    mut users: ResMut<Users>,
    mut server: ResMut<QuinnetServer>,
    time: Res<Time>,
) {
    let now = time.elapsed();
    for client in connection.read() {
        users.add_pending(client.id, now);
    }
    for client in connection_lost.read() {
        handle_disconnect(&mut users, client.id, None, "Connection lost");
    }
    let endpoint = server.endpoint_mut();
    for id in users.drain_pending_too_long(Duration::from_secs(3), now) {
        handle_disconnect(&mut users, id, Some(endpoint), "Pending too long");
    }
}

pub fn handle_disconnect(
    users: &mut Users,
    id: ClientId,
    endpoint: Option<&mut Endpoint>,
    cause: &'static str,
) {
    if let Some(user) = users.remove(&id) {
        info!(id, name = user.name, cause, "Client disconnected.");
    } else {
        info!(id, cause, "Client disconnected.");
    }
    if let Some(endpoint) = endpoint {
        endpoint.disconnect_client(id).ok();
    }
}

pub fn start_server(server: &mut QuinnetServer) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_string("0.0.0.0:6000").unwrap(),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: "ApeBox sp. Zloo".to_string(),
            },
            ChannelsConfiguration::default(),
        )
        .unwrap();
}

pub fn stop_server(server: &mut QuinnetServer) {
    server.stop_endpoint().ok();
}
