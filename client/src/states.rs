use bevy::prelude::*;

use crate::loader::ResourceBarrierMarker;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum ClientState {
    /// Loading resources.
    #[default]
    Loading,
    /// In menu.
    Menu,
    /// In a lobby.
    Game,
}

pub struct InitialResources;

impl ResourceBarrierMarker for InitialResources {
    type State = ClientState;

    fn loading_state() -> Self::State {
        ClientState::Loading
    }

    fn next_state() -> Self::State {
        ClientState::Menu
    }
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(ClientState = ClientState::Menu)]
pub enum MenuState {
    /// Configuring server parameters.
    #[default]
    Configuring,
    /// Attempting to connect to a server.
    Connecting,
}

#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(ClientState = ClientState::Game)]
pub enum GameState {
    #[default]
    Wait,
    Draw,
    Prompt,
    Combine,
    Vote,
    Winner,
}
