use crate::states::GameState;

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
use common::{
    game::{Author, Combination, Vote},
    protocol::NetMsg,
};
use common::{
    game::{Drawing, Prompt},
    protocol::{ClientMsgComm, ClientMsgRoot, ServerMsgRoot},
};
use std::time::Duration;

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

pub fn receive_messages(
    mut server: ResMut<QuinnetServer>,
    mut sink: EventWriter<NetMsg<ClientMsgRoot>>,
) {
    let endpoint = server.endpoint_mut();
    for client in endpoint.clients() {
        while let Some((channel, data)) = endpoint.try_receive_message_from::<ClientMsgRoot>(client)
        {
            let net_msg = NetMsg::new(client, channel, data);
            sink.send(net_msg);
        }
    }
}

pub fn handle_root(
    mut source: ResMut<Events<NetMsg<ClientMsgRoot>>>,
    mut sink: EventWriter<NetMsg<ClientMsgComm>>,
    mut server: ResMut<QuinnetServer>,
    mut users: ResMut<Users>,
) {
    let endpoint = server.endpoint_mut();
    for NetMsg {
        client,
        channel,
        data,
    } in source.drain()
    {
        let user = users.registered.get(&client);
        match data {
            ClientMsgRoot::Connect { name } => {
                let name_exists = users.registered.values().any(|u| u.name == *name);
                if user.is_some() || name_exists {
                    handle_disconnect(
                        &mut users,
                        client,
                        Some(endpoint),
                        "Duplicate client id or name",
                    );
                } else {
                    info!(client, name, "Client active.");
                    users.register(client, name);
                    endpoint.send_message(client, ServerMsgRoot::Wait).unwrap();
                }
            }
            ClientMsgRoot::Comm(comm) => {
                if user.is_none() {
                    handle_disconnect(
                        &mut users,
                        client,
                        Some(endpoint),
                        "Non-registered user attempted communication",
                    );
                    continue;
                }
                sink.send(NetMsg::new(client, channel, comm));
            }
            ClientMsgRoot::Disconnect => {
                info!(client, "Client disconnected.");
                handle_disconnect(&mut users, client, Some(endpoint), "Disconnected");
            }
        }
    }
}

#[derive(Event, Debug)]
pub struct Submission<T> {
    pub author: Author,
    pub data: T,
}

impl<T> Submission<T> {
    pub fn new(author: Author, data: T) -> Self {
        Self { author, data }
    }
}

pub fn handle_comm(
    mut source: ResMut<Events<NetMsg<ClientMsgComm>>>,
    mut sub_draw: EventWriter<Submission<Drawing>>,
    mut sub_prompt: EventWriter<Submission<Prompt>>,
    mut sub_combination: EventWriter<Submission<Combination>>,
    mut sub_vote: EventWriter<Submission<Vote>>,
    state: Option<Res<State<GameState>>>,
    users: Res<Users>,
) {
    let Some(state) = state else {
        source.clear();
        return;
    };
    for NetMsg { client, data, .. } in source.drain() {
        let state = state.get();
        let user = users.registered.get(&client).unwrap();
        let author = Author {
            id: client,
            name: user.name.clone(),
        };
        match data {
            ClientMsgComm::SubmitDrawing(data) => {
                if *state != GameState::Draw {
                    continue;
                }
                sub_draw.send(Submission::new(author, data));
            }
            ClientMsgComm::SubmitPrompt(data) => {
                if *state != GameState::Prompt {
                    continue;
                }
                sub_prompt.send(Submission::new(author, data));
            }
            ClientMsgComm::SubmitCombination(data) => {
                if *state != GameState::Combine {
                    continue;
                }
                sub_combination.send(Submission::new(author, data));
            }
            ClientMsgComm::SubmitVote(data) => {
                if *state != GameState::Vote {
                    continue;
                }
                sub_vote.send(Submission::new(author, data));
            }
        }
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
