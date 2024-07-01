pub mod combine;
pub mod draw;
pub mod prompt;
pub mod vote;
pub mod wait;
pub mod winner;

use bevy::prelude::*;

pub struct ModesPlugin;

impl Plugin for ModesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            draw::ModePlugin,
            prompt::ModePlugin,
            combine::ModePlugin,
            vote::ModePlugin,
            winner::ModePlugin,
        ));
    }
}
