mod game;
mod menu;
mod widgets;

use bevy::prelude::*;

use crate::states::{ClientState, GameState};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        // Menu
        app.add_systems(Update, menu::show.run_if(in_state(ClientState::Menu)));

        // Wait
        app.add_systems(Update, game::wait::update.run_if(in_state(GameState::Wait)));

        // Draw
        app.add_systems(Update, game::draw::update.run_if(in_state(GameState::Draw)));
        app.add_systems(OnEnter(GameState::Draw), game::draw::setup);
        app.add_systems(OnExit(GameState::Draw), game::draw::teardown);

        // Prompt
        app.add_systems(
            Update,
            game::prompt::update.run_if(in_state(GameState::Prompt)),
        );
        app.add_systems(OnEnter(GameState::Prompt), game::prompt::setup);
        app.add_systems(OnExit(GameState::Prompt), game::prompt::teardown);

        // Combine
        app.add_systems(
            Update,
            game::combine::update.run_if(in_state(GameState::Combine)),
        );
        app.add_systems(OnExit(GameState::Combine), game::combine::teardown);

        // Vote
        app.add_systems(Update, game::vote::update.run_if(in_state(GameState::Vote)));
        app.add_systems(OnExit(GameState::Vote), game::vote::teardown);
    }
}
