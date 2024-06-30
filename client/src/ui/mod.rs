pub mod fonts;
pub mod menu;
pub mod modes;
pub mod util;
pub mod widgets;

use bevy::prelude::*;
use fonts::FontsPlugin;
use modes::ModesPlugin;

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
            modes::wait::update
                .in_set(GameSystemOdering::StateLogic)
                .run_if(in_state(GameState::Wait)),
        );

        app.add_plugins((FontsPlugin, ModesPlugin));
    }
}
