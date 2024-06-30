pub mod combine;
pub mod draw;
pub mod prompt;
pub mod vote;
pub mod wait;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            draw::ModePlugin,
            prompt::ModePlugin,
            combine::ModePlugin,
            vote::ModePlugin,
        ));
    }
}
