pub mod fonts;
pub mod game;
pub mod menu;
pub mod util;
pub mod widgets;

use bevy::prelude::*;
use fonts::FontsPlugin;
use game::GamePlugin;

use crate::{
    states::{ClientState, GameState},
    GameSystemOdering,
};

pub struct ClientUiPlugin;

impl Plugin for ClientUiPlugin {
    fn build(&self, app: &mut App) {
        // Menu
        app.add_systems(Update, menu::show.run_if(in_state(ClientState::Menu)));

        // Wait
        app.add_systems(
            Update,
            game::wait::update
                .in_set(GameSystemOdering::StateLogic)
                .run_if(in_state(GameState::Wait)),
        );

        app.add_plugins((FontsPlugin, GamePlugin));
    }
}
