pub mod combine;
pub mod draw;
pub mod prompt;
pub mod vote;
pub mod wait;

use bevy::prelude::*;
use combine::CombinePlugin;
use draw::DrawPlugin;
use prompt::PromptPlugin;
use vote::VotePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DrawPlugin, PromptPlugin, CombinePlugin, VotePlugin));
    }
}
