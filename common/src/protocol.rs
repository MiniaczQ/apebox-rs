use std::time::Duration;

use bevy::prelude::Event;
use bevy_quinnet::shared::{channels::ChannelId, ClientId};
use serde::{Deserialize, Serialize};

use crate::game::{Combination, Drawing, Index, Prompt, Vote};

#[derive(Event)]
pub struct NetMsg<T> {
    pub client: ClientId,
    pub channel: ChannelId,
    pub data: T,
}

impl<T> NetMsg<T> {
    pub fn new(client: ClientId, channel: ChannelId, data: T) -> Self {
        Self {
            client,
            channel,
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsgRoot {
    Connect { name: String },
    Comm(ClientMsgComm),
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMsgComm {
    SubmitDrawing(Drawing),
    SubmitPrompt(Prompt),
    SubmitCombination(Combination),
    SubmitVote(Vote),
}

impl ClientMsgComm {
    pub fn root(self) -> ClientMsgRoot {
        ClientMsgRoot::Comm(self)
    }
}

// Messages from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMsgRoot {
    Draw {
        duration: Duration,
    },
    Prompt {
        duration: Duration,
    },
    Combine {
        duration: Duration,
        drawings: Vec<(Index, Drawing)>,
        prompts: Vec<(Index, Prompt)>,
    },
    Vote {
        duration: Duration,
        combination1: (Index, Drawing, Prompt),
        combination2: (Index, Drawing, Prompt),
    },
    Winner {
        duration: Duration,
        drawing: Drawing,
        prompt: Prompt,
    },
    Wait,
}
